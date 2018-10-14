//! gram defines the atomic unit of the miknet protocol.

use bincode::serialize;
use itertools::{Either, Itertools};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;
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
pub enum ApiEvent {
    Tx(Vec<u8>),
    Disc,
    Conn,
    Shutdown,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Api(ApiEvent),
    Gram(Gram),
    Chunk(Chunk),
    Timer(Timer),
    InvalidGram,
}

impl From<Chunk> for Event {
    fn from(chunk: Chunk) -> Event { Event::Chunk(chunk) }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApiCmd {
    Disconnect,
    ConnectionAttemptTimedOut,
    ConnectionEstablished,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Cmd {
    Chunk(Chunk),
    Net(Gram),
    Timer(Timer),
    Api(ApiCmd),
}

pub trait Protocol: Sized {
    fn establish() -> Self;
    fn step(self, event: Event) -> (Self, Vec<Cmd>);
}

/// Connection is a relationship between two miknet nodes. Handshake and teardown based roughly on
/// SCTP.
#[derive(Clone, Debug, PartialEq)]
pub enum Connection<P: Protocol + Debug> {
    Listen {
        key: Key,
    },
    InitSent {
        token: u32,
        tsn:   u32,
        queue: Vec<ApiEvent>,
    },
    InitAckSent {
        their_token: u32,
    },
    CookieEchoed {
        tcb:   Tcb,
        queue: Vec<ApiEvent>,
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

impl<P: Protocol + Debug> Connection<P> {
    /// Returns a connection using the given key to sign and verify state cookies.
    pub fn new(key: Key) -> Self { Connection::Listen { key } }

    /// Processes an event and returns the next state of the connection and any commands that
    /// should be executed as part of the transition.
    pub fn step(self, event: Event) -> (Self, Vec<Cmd>) {
        let (next, cmds) = self.handle_event(event);
        let (chunks, mut cmds): (Vec<Chunk>, Vec<Cmd>) =
            cmds.into_iter().partition_map(|cmd| match cmd {
                Cmd::Chunk(chunk) => Either::Left(chunk),
                cmd => Either::Right(cmd),
            });
        if !chunks.is_empty() {
            cmds.extend(next.build_grams(chunks));
        }
        (next, cmds)
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
        grams.into_iter().map(|gram| Cmd::Net(gram)).collect()
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

    fn handle_events(self, events: Vec<Event>) -> (Self, Vec<Cmd>) {
        events.into_iter().fold(
            (self, Vec::new()),
            |(conn, mut cmds), event| {
                let (next_conn, more_cmds) = conn.handle_event(event);
                cmds.extend(more_cmds);
                (next_conn, cmds)
            },
        )
    }

    fn handle_event(self, event: Event) -> (Self, Vec<Cmd>) {
        match (self, event) {
            (conn, Event::Gram(gram)) => match conn.expected_token() {
                Some(expected_token) if gram.token != expected_token => {
                    conn.handle_event(Event::InvalidGram)
                }
                _ => conn.handle_events(
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
            (Connection::Listen { .. }, Event::Api(ApiEvent::Conn)) => {
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
                        Cmd::Api(ApiCmd::ConnectionEstablished),
                    ],
                )
            }
            (Connection::InitSent { .. }, Event::Timer(Timer::InitTimer)) => (
                Connection::Failed,
                vec![Cmd::Api(ApiCmd::ConnectionAttemptTimedOut)],
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
                    tcb: Tcb {
                        our_tsn,
                        our_token,
                        their_tsn,
                        their_token,
                    },
                    queue,
                },
                vec![
                    Cmd::Chunk(Chunk::CookieEcho(state_cookie)),
                    Cmd::Timer(Timer::CookieSentTimer),
                ],
            ),
            (
                Connection::InitSent {
                    token,
                    tsn,
                    mut queue,
                },
                Event::Api(ae),
            ) => (
                Connection::InitSent {
                    token,
                    tsn,
                    queue: {
                        queue.push(ae);
                        queue
                    },
                },
                Vec::new(),
            ),
            (Connection::CookieEchoed { tcb, queue }, Event::Api(ae)) => (
                Connection::CookieEchoed {
                    tcb,
                    queue: queue
                        .into_iter()
                        .chain(vec![ae].into_iter())
                        .collect(),
                },
                Vec::new(),
            ),
            (
                Connection::CookieEchoed { tcb, queue },
                Event::Chunk(Chunk::CookieAck),
            ) => {
                let (conn, mut cmds) = queue.into_iter().fold(
                    (
                        Connection::Established {
                            tcb,
                            protocol: P::establish(),
                        },
                        Vec::new(),
                    ),
                    |(conn, mut cmds), ae| {
                        let (next_conn, more_cmds) =
                            conn.handle_event(Event::Api(ae));
                        cmds.extend(more_cmds);
                        (next_conn, cmds)
                    },
                );
                cmds.push(Cmd::Api(ApiCmd::ConnectionEstablished));
                (conn, cmds)
            }
            (
                Connection::Established { tcb, .. },
                Event::Api(ApiEvent::Disc),
            ) => (
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
                    Cmd::Api(ApiCmd::Disconnect),
                    Cmd::Chunk(Chunk::ShutdownComplete),
                ],
            ),
            (
                Connection::ShutdownAckSent { their_token, .. },
                Event::Chunk(Chunk::ShutdownComplete),
            ) => (
                Connection::Shutdown { their_token },
                vec![Cmd::Api(ApiCmd::Disconnect)],
            ),
            (
                Connection::ShutdownAckSent { their_token, .. },
                Event::Chunk(Chunk::ShutdownAck),
            ) => (
                Connection::Shutdown { their_token },
                vec![
                    Cmd::Api(ApiCmd::Disconnect),
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
    use failure::Error;
    use std::str::FromStr;

    #[derive(Debug)]
    struct TumblerProtocol;

    impl Protocol for TumblerProtocol {
        fn establish() -> Self { Self {} }

        fn step(self, event: Event) -> (Self, Vec<Cmd>) { (self, Vec::new()) }
    }

    #[derive(Debug)]
    struct Tumbler {
        key1:    Key,
        key2:    Key,
        c1:      Connection<TumblerProtocol>,
        c2:      Connection<TumblerProtocol>,
        c1_cmds: Vec<Cmd>,
    }

    impl Tumbler {
        fn new() -> Result<Self, Error> {
            let key1 = Key::new()?;
            let key2 = Key::new()?;
            Ok(Self {
                c1: Connection::new(key1.clone()),
                c2: Connection::new(key2.clone()),
                key1,
                key2,
                c1_cmds: Vec::new(),
            })
        }

        fn start(self) -> Self {
            let (c1, c1_cmds) = self.c1.step(Event::Api(ApiEvent::Conn));
            Self {
                c1,
                c1_cmds,
                ..self
            }
        }

        fn established(&self) -> bool {
            match (&self.c1, &self.c2) {
                (
                    Connection::Established { .. },
                    Connection::Established { .. },
                ) => true,
                _ => false,
            }
        }

        fn tumble(self) -> Self {
            let (c2, c2_cmds) =
                Tumbler::tumble_connection(self.c2, self.c1_cmds);
            println!("c2 cmds: {:?}", c2_cmds);
            let (c1, c1_cmds) = Tumbler::tumble_connection(self.c1, c2_cmds);
            Self {
                c1: if c1.should_persist() {
                    c1
                } else {
                    Connection::new(self.key1.clone())
                },
                c2: if c2.should_persist() {
                    c2
                } else {
                    Connection::new(self.key2.clone())
                },
                c1_cmds,
                ..self
            }
        }

        fn tumble_connection(
            mut c: Connection<TumblerProtocol>,
            cmds: Vec<Cmd>,
        ) -> (Connection<TumblerProtocol>, Vec<Cmd>) {
            let mut out_cmds = Vec::new();
            for event in Tumbler::convert_cmds(cmds) {
                let (next_c, mut next_cmds) = c.step(event);
                c = next_c;
                out_cmds.append(&mut next_cmds);
            }
            (c, out_cmds)
        }

        fn convert_cmds(cmds: Vec<Cmd>) -> Vec<Event> {
            cmds.into_iter()
                .filter_map(|cmd| match cmd {
                    Cmd::Net(gram) => Some(Event::Gram(gram)),
                    _ => None,
                })
                .collect()
        }
    }

    #[test]
    fn handshake() -> Result<(), Error> {
        let mut tumbler = Tumbler::new()?;
        tumbler = tumbler.start();
        for i in 0..2 {
            tumbler = tumbler.tumble();
        }
        if !tumbler.established() {
            println!("Connection not established; states:");
            println!("{:?}", tumbler);
            panic!();
        }
        Ok(())
    }
}
