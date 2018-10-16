//! Connection manages the line beneath the miknet upstream.

use bincode::serialize;
use itertools::{Either, Itertools};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;
use std::net::SocketAddr;
use std::rc::Rc;
use std::time::Duration;

use crate::api;
use crate::protocol::validation::{Key, StateCookie, Tcb};
use crate::random::random;

/// Chunks are control and data messages that can be packed in a gram.
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
    Data(Vec<u8>),
}

/// Gram is the atomic unit of the miknet upstream. All transmissions are represented as a gram
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
pub enum WireEvent {
    Api(ApiEvent),
    Gram(Gram),
    Chunk(Chunk),
    Timer(Timer),
    InvalidGram,
}

impl From<Chunk> for WireEvent {
    fn from(chunk: Chunk) -> WireEvent { WireEvent::Chunk(chunk) }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApiCmd {
    NotifyDisconnect,
    NotifyConnectionAttemptTimedOut,
    NotifyConnectionEstablished,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WireCmd {
    Net(Gram),
    Timer(Timer),
    Api(ApiCmd),
    Chunk(Chunk),
    Upstream(Vec<u8>),
}

/// Connection is a relationship between two miknet nodes. Handshake and teardown based roughly on
/// SCTP.
#[derive(Clone, Debug, PartialEq)]
pub enum Connection {
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
        tcb: Tcb,
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

impl Connection {
    /// Returns a connection using the given key to sign and verify state cookies.
    pub fn new(key: Key) -> Self { Connection::Listen { key } }

    /// Processes an event and returns the next state of the connection and any commands that
    /// should be executed as part of the transition.
    pub fn step(self, event: WireEvent) -> (Self, Vec<WireCmd>) {
        let (next, cmds) = self.handle_event(event);
        let (chunks, mut cmds): (Vec<Chunk>, Vec<WireCmd>) =
            cmds.into_iter().partition_map(|cmd| match cmd {
                WireCmd::Chunk(chunk) => Either::Left(chunk),
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
    fn build_grams(&self, chunks: Vec<Chunk>) -> Vec<WireCmd> {
        let token = self.token();
        let grams = match *self {
            _ => vec![Gram { token, chunks }],
        };
        grams.into_iter().map(|gram| WireCmd::Net(gram)).collect()
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

    fn handle_events(self, events: Vec<WireEvent>) -> (Self, Vec<WireCmd>) {
        events.into_iter().fold(
            (self, Vec::new()),
            |(conn, mut cmds), event| {
                let (next_conn, more_cmds) = conn.handle_event(event);
                cmds.extend(more_cmds);
                (next_conn, cmds)
            },
        )
    }

    fn handle_event(self, event: WireEvent) -> (Self, Vec<WireCmd>) {
        match (self, event) {
            (conn, WireEvent::Gram(gram)) => match conn.expected_token() {
                Some(expected_token) if gram.token != expected_token => {
                    conn.handle_event(WireEvent::InvalidGram)
                }
                _ => conn.handle_events(
                    gram.chunks.into_iter().map(Chunk::into).collect(),
                ),
            },
            (
                Connection::Listen { key, .. },
                WireEvent::Chunk(Chunk::Init {
                    token: their_token,
                    tsn: their_tsn,
                }),
            ) => {
                let (our_tsn, our_token) = (random(), random());
                (
                    Connection::InitAckSent { their_token },
                    vec![WireCmd::Chunk(Chunk::InitAck {
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
            (Connection::Listen { .. }, WireEvent::Api(ApiEvent::Conn)) => {
                let (token, tsn) = (random(), random());
                (
                    Connection::InitSent {
                        token,
                        tsn,
                        queue: Vec::new(),
                    },
                    vec![
                        WireCmd::Chunk(Chunk::Init { token, tsn }),
                        WireCmd::Timer(Timer::InitTimer),
                    ],
                )
            }
            (
                Connection::Listen { key },
                WireEvent::Chunk(Chunk::CookieEcho(state_cookie)),
            )
                if state_cookie.signed_by(&key) =>
            {
                (
                    Connection::Established {
                        tcb: state_cookie.tcb.clone(),
                    },
                    vec![
                        WireCmd::Chunk(Chunk::CookieAck),
                        WireCmd::Api(ApiCmd::NotifyConnectionEstablished),
                    ],
                )
            }
            (
                Connection::InitSent { .. },
                WireEvent::Timer(Timer::InitTimer),
            ) => (
                Connection::Failed,
                vec![WireCmd::Api(ApiCmd::NotifyConnectionAttemptTimedOut)],
            ),
            (
                Connection::InitSent {
                    token: our_token,
                    tsn: our_tsn,
                    queue,
                },
                WireEvent::Chunk(Chunk::InitAck {
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
                    WireCmd::Chunk(Chunk::CookieEcho(state_cookie)),
                    WireCmd::Timer(Timer::CookieSentTimer),
                ],
            ),
            (
                Connection::InitSent {
                    token,
                    tsn,
                    mut queue,
                },
                WireEvent::Api(ae),
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
            (Connection::CookieEchoed { tcb, queue }, WireEvent::Api(ae)) => (
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
                WireEvent::Chunk(Chunk::CookieAck),
            ) => {
                let (conn, mut cmds) = queue.into_iter().fold(
                    (Connection::Established { tcb }, Vec::new()),
                    |(conn, mut cmds), ae| {
                        let (next_conn, more_cmds) =
                            conn.handle_event(WireEvent::Api(ae));
                        cmds.extend(more_cmds);
                        (next_conn, cmds)
                    },
                );
                cmds.push(WireCmd::Api(ApiCmd::NotifyConnectionEstablished));
                (conn, cmds)
            }
            (
                Connection::Established { tcb, .. },
                WireEvent::Api(ApiEvent::Disc),
            ) => (
                Connection::ShutdownSent {
                    token:       tcb.our_token,
                    their_token: tcb.their_token,
                },
                vec![WireCmd::Chunk(Chunk::Shutdown)],
            ),
            (
                Connection::Established { tcb, .. },
                WireEvent::Chunk(Chunk::Shutdown),
            ) => (
                Connection::ShutdownAckSent {
                    token:       tcb.our_token,
                    their_token: tcb.their_token,
                },
                vec![WireCmd::Chunk(Chunk::ShutdownAck)],
            ),
            (
                Connection::Established { tcb },
                WireEvent::Chunk(Chunk::Data(payload)),
            ) => (
                Connection::Established { tcb },
                vec![WireCmd::Upstream(payload)],
            ),
            (
                Connection::ShutdownSent { token, their_token },
                WireEvent::Chunk(Chunk::Shutdown),
            ) => (
                Connection::ShutdownAckSent { token, their_token },
                vec![WireCmd::Chunk(Chunk::ShutdownAck)],
            ),
            (
                Connection::ShutdownSent { their_token, .. },
                WireEvent::Chunk(Chunk::ShutdownAck),
            ) => (
                Connection::Shutdown { their_token },
                vec![
                    WireCmd::Api(ApiCmd::NotifyDisconnect),
                    WireCmd::Chunk(Chunk::ShutdownComplete),
                ],
            ),
            (
                Connection::ShutdownAckSent { their_token, .. },
                WireEvent::Chunk(Chunk::ShutdownComplete),
            ) => (
                Connection::Shutdown { their_token },
                vec![WireCmd::Api(ApiCmd::NotifyDisconnect)],
            ),
            (
                Connection::ShutdownAckSent { their_token, .. },
                WireEvent::Chunk(Chunk::ShutdownAck),
            ) => (
                Connection::Shutdown { their_token },
                vec![
                    WireCmd::Api(ApiCmd::NotifyDisconnect),
                    WireCmd::Chunk(Chunk::ShutdownComplete),
                ],
            ),
            (conn, _) => (conn, Vec::new()),
        }
    }
}
