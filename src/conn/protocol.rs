//! gram defines the atomic unit of the miknet protocol.

use bincode::serialize;
use itertools::{Either, Itertools};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;

use crate::api;
use crate::conn::handshake::{Key, StateCookie, Tcb};
use crate::conn::sequence::Segment;
use crate::random::random;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Chunk {
    Init {
        token: u32,
        tsn:   u32,
    },
    InitAck {
        token:        u32,
        tsn:          u32,
        state_cookie: StateCookie,
    },
    CookieEcho(StateCookie),
    CookieAck,
    Shutdown,
    ShutdownAck,
    ShutdownComplete,
    CfgMismatch,
    Data {
        channel_id: u32,
        seg:        Segment,
    },
    DataAck {
        channel_id: u32,
        seq:        u32,
    },
}

/// Gram is the atomic unit of the miknet protocol. All transmissions are represented as a gram
/// before they are written on the network.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Gram {
    pub token:  u32,
    pub chunks: Vec<Chunk>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Timer {
    InitTimer,
    CookieSentTimer,
}

impl Timer {
    fn duration(&self) -> Duration {
        match *self {
            Timer::InitTimer => Duration::new(3, 0),
            Timer::CookieSentTimer => Duration::new(5, 0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Api {
    Tx(Vec<u8>),
    Disc,
    Conn,
    Shutdown,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Api(Api),
    Gram(Gram),
    Chunk(Chunk),
    Timer(Timer),
    InvalidGram,
}

impl From<Chunk> for Event {
    fn from(chunk: Chunk) -> Event { Event::Chunk(chunk) }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Cmd {
    Chunk(Chunk),
    Net(Vec<u8>),
    Timer(Timer),
    User(api::Event),
}

pub trait Protocol: Sized {
    fn establish() -> Self;
    fn step(self, event: Event) -> (Self, Vec<Cmd>);
}

/// Connection is a relationship between two miknet nodes. Handshake and teardown based roughly on
/// SCTP.
#[derive(Clone, Debug, PartialEq)]
pub enum Connection<P: Protocol> {
    Listen {
        key: Key,
    },
    InitSent {
        token: u32,
        tsn:   u32,
        queue: Vec<(SocketAddr, Api)>,
    },
    InitAckSent {
        their_token: u32,
    },
    CookieEchoed {
        tcb:   Tcb,
        queue: Vec<(SocketAddr, Api)>,
    },
    Established {
        tcb:      Tcb,
        protocol: P,
    },
    ShutdownSent {
        token:       u32,
        their_token: u32,
    },
    ShutdownAckSent {
        token:       u32,
        their_token: u32,
    },
    Shutdown {
        their_token: u32,
    },
    Failed,
}

impl<P: Protocol> Connection<P> {
    /// Returns a connection using the given key to sign and verify state cookies.
    pub fn new(key: Key) -> Self { Connection::Listen { key } }

    /// Processes an event and returns the next state of the connection and any commands that
    /// should be executed as part of the transition.
    pub fn step(
        self,
        peer: SocketAddr,
        event: Event,
    ) -> (Self, Vec<(SocketAddr, Cmd)>) {
        let (next, cmds) = self.handle_event(&peer, event);
        let (chunks, mut cmds): (Vec<Chunk>, Vec<Cmd>) =
            cmds.into_iter().partition_map(|cmd| match cmd {
                Cmd::Chunk(chunk) => Either::Left(chunk),
                cmd => Either::Right(cmd),
            });
        cmds.extend(next.build_grams(chunks));
        (
            next,
            cmds.into_iter().map(|cmd| (peer.clone(), cmd)).collect(),
        )
    }

    /// Returns whether this connection should be persisted between events. For example, we do not
    /// keep an InitAckSent state between events to prevent DOS attacks.
    pub fn should_persist(&self) -> bool {
        match *self {
            Connection::Listen { .. } => false,
            Connection::InitAckSent { .. } => false,
            Connection::Failed => false,
            _ => true,
        }
    }

    /// Prepares chunks as net commands to write out grams.
    fn build_grams(&self, chunks: Vec<Chunk>) -> Vec<Cmd> {
        let token = self.token();
        let grams = match *self {
            _ => vec![Gram { token, chunks }],
        };
        grams
            .into_iter()
            .map(|gram| Cmd::Net(serialize(&gram).unwrap()))
            .collect()
    }

    /// Returns the token this connection should embed in all its grams for association.
    fn token(&self) -> u32 {
        match *self {
            Connection::InitAckSent { their_token } => their_token,
            Connection::InitSent { token, .. } => token,
            Connection::CookieEchoed { ref tcb, .. } => tcb.their_token,
            Connection::Established { ref tcb, .. } => tcb.their_token,
            Connection::ShutdownSent { their_token, .. } => their_token,
            Connection::ShutdownAckSent { their_token, .. } => their_token,
            Connection::Shutdown { their_token } => their_token,
            _ => 0,
        }
    }

    /// Returns the token we expect in valid grams from our peer.
    fn expected_token(&self) -> Option<u32> {
        match *self {
            Connection::InitSent { token, .. } => Some(token),
            Connection::CookieEchoed { ref tcb, .. } => Some(tcb.our_token),
            Connection::Established { ref tcb, .. } => Some(tcb.our_token),
            Connection::ShutdownSent { token, .. } => Some(token),
            Connection::ShutdownAckSent { token, .. } => Some(token),
            _ => None,
        }
    }

    fn handle_events(
        self,
        peer: &SocketAddr,
        events: Vec<Event>,
    ) -> (Self, Vec<Cmd>) {
        events.into_iter().fold(
            (self, Vec::new()),
            |(conn, mut cmds), event| {
                let (next_conn, more_cmds) = conn.handle_event(peer, event);
                cmds.extend(more_cmds);
                (next_conn, cmds)
            },
        )
    }

    fn handle_event(self, peer: &SocketAddr, event: Event) -> (Self, Vec<Cmd>) {
        match (self, event) {
            (conn, Event::Gram(gram)) => match conn.expected_token() {
                Some(expected_token) if gram.token != expected_token => {
                    conn.handle_event(peer, Event::InvalidGram)
                }
                _ => conn.handle_events(
                    peer,
                    gram.chunks.into_iter().map(Chunk::into).collect(),
                ),
            },
            (
                Connection::Listen { key },
                Event::Chunk(Chunk::Init {
                    token: their_token,
                    tsn: their_tsn,
                }),
            ) => {
                let (our_tsn, our_token) = (random(), random());
                (
                    Connection::InitAckSent { their_token },
                    vec![Cmd::Chunk(Chunk::InitAck {
                        tsn:          our_tsn,
                        token:        our_token,
                        state_cookie: StateCookie::new(
                            Tcb {
                                our_tsn,
                                our_token,
                                their_token,
                                their_tsn,
                            },
                            &key,
                        ),
                    })],
                )
            }
            (Connection::Listen { .. }, Event::Api(Api::Conn)) => {
                let (token, tsn) = (random(), random());
                (
                    Connection::InitSent {
                        token,
                        tsn,
                        queue: Vec::new(),
                    },
                    vec![
                        Cmd::Chunk(Chunk::Init { token, tsn }),
                        Cmd::Timer(Timer::InitTimer),
                    ],
                )
            }
            (Connection::InitSent { .. }, Event::Timer(Timer::InitTimer)) => (
                Connection::Failed,
                vec![Cmd::User(api::Event::ConnectionAttemptTimedOut(*peer))],
            ),
            (
                Connection::InitSent {
                    token: our_token,
                    tsn: our_tsn,
                    queue,
                },
                Event::Chunk(Chunk::InitAck {
                    token: their_token,
                    tsn: their_tsn,
                    state_cookie,
                }),
            ) => (
                Connection::CookieEchoed {
                    tcb:   Tcb {
                        our_tsn,
                        our_token,
                        their_tsn,
                        their_token,
                    },
                    queue: queue,
                },
                vec![
                    Cmd::Chunk(Chunk::CookieEcho(state_cookie)),
                    Cmd::Timer(Timer::CookieSentTimer),
                ],
            ),
            (Connection::InitSent { token, tsn, queue }, Event::Api(ae)) => (
                Connection::InitSent {
                    token,
                    tsn,
                    queue: queue
                        .into_iter()
                        .chain(vec![(*peer, ae)].into_iter())
                        .collect(),
                },
                Vec::new(),
            ),
            (Connection::CookieEchoed { tcb, queue }, Event::Api(ae)) => (
                Connection::CookieEchoed {
                    tcb,
                    queue: queue
                        .into_iter()
                        .chain(vec![(*peer, ae)].into_iter())
                        .collect(),
                },
                Vec::new(),
            ),
            (
                Connection::CookieEchoed { tcb, queue },
                Event::Chunk(Chunk::CookieAck),
            ) => {
                let (conn, mut cmds) = queue
                    .into_iter()
                    .map(|(peer, ae)| (peer, Event::Api(ae)))
                    .fold(
                        (
                            Connection::Established {
                                tcb,
                                protocol: P::establish(),
                            },
                            Vec::new(),
                        ),
                        |(conn, mut cmds), (peer, ae)| {
                            let (next_conn, more_cmds) =
                                conn.handle_event(&peer, ae);
                            cmds.extend(more_cmds);
                            (next_conn, cmds)
                        },
                    );
                cmds.extend(vec![Cmd::User(
                    api::Event::ConnectionEstablished(*peer),
                )]);
                (conn, cmds)
            }
            (
                Connection::Listen { ref key, .. },
                Event::Chunk(Chunk::CookieEcho(ref state_cookie)),
            )
                if state_cookie.signed_by(&key) =>
            {
                (
                    Connection::Established {
                        tcb:      state_cookie.tcb.clone(),
                        protocol: P::establish(),
                    },
                    vec![
                        Cmd::Chunk(Chunk::CookieAck),
                        Cmd::User(api::Event::ConnectionEstablished(*peer)),
                    ],
                )
            }
            (Connection::Established { tcb, .. }, Event::Api(Api::Disc)) => (
                Connection::ShutdownSent {
                    token:       tcb.our_token,
                    their_token: tcb.their_token,
                },
                vec![Cmd::Chunk(Chunk::Shutdown)],
            ),
            (
                Connection::Established { tcb, .. },
                Event::Chunk(Chunk::Shutdown),
            ) => (
                Connection::ShutdownAckSent {
                    token:       tcb.our_token,
                    their_token: tcb.their_token,
                },
                vec![Cmd::Chunk(Chunk::ShutdownAck)],
            ),
            (
                Connection::ShutdownSent { token, their_token },
                Event::Chunk(Chunk::Shutdown),
            ) => (
                Connection::ShutdownAckSent { token, their_token },
                vec![Cmd::Chunk(Chunk::ShutdownAck)],
            ),
            (
                Connection::ShutdownSent { their_token, .. },
                Event::Chunk(Chunk::ShutdownAck),
            ) => (
                Connection::Shutdown { their_token },
                vec![
                    Cmd::User(api::Event::Disconnect(*peer)),
                    Cmd::Chunk(Chunk::ShutdownComplete),
                ],
            ),
            (
                Connection::ShutdownAckSent { their_token, .. },
                Event::Chunk(Chunk::ShutdownComplete),
            ) => (
                Connection::Shutdown { their_token },
                vec![Cmd::User(api::Event::Disconnect(*peer))],
            ),
            (
                Connection::ShutdownAckSent { their_token, .. },
                Event::Chunk(Chunk::ShutdownAck),
            ) => (
                Connection::Shutdown { their_token },
                vec![
                    Cmd::User(api::Event::Disconnect(*peer)),
                    Cmd::Chunk(Chunk::ShutdownComplete),
                ],
            ),
            (conn, _) => (conn, Vec::new()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::random;
    use std::str::FromStr;

    struct DummyProtocol;

    impl Protocol for DummyProtocol {
        fn establish() -> Self { DummyProtocol {} }

        fn step(self, event: Event) -> (Self, Vec<Cmd>) { (self, Vec::new()) }
    }

    fn expect(expectations: Vec<(Event, Vec<Cmd>)>) {
        let peer_addr =
            SocketAddr::from_str("127.0.0.1:0").expect("any address");
        let mut conn =
            Connection::<DummyProtocol>::new(Key::new().expect("key"));
        for (event, expected_cmds) in expectations {
            let (next_conn, cmds) = conn.handle_event(&peer_addr, event);
            assert_eq!(cmds, expected_cmds);
            conn = next_conn
        }
    }

    #[test]
    fn handshake() {
        expect(vec![(
            Event::Api(Api::Conn),
            vec![
                Cmd::Chunk(Chunk::Init {
                    token: random::RAND_TEST_CONST,
                    tsn:   random::RAND_TEST_CONST,
                }),
                Cmd::Timer(Timer::InitTimer),
            ],
        )])
    }
}
