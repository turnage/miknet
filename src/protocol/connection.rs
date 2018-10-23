//! Connection manages the line beneath the miknet upstream.

use bincode::serialize;
use crate::api::ApiCall;
use itertools::{Either, Itertools};
use std::{fmt::Debug, net::SocketAddr, rc::Rc, time::Duration};

use crate::{
    api,
    protocol::{
        transducer::Transducer,
        validation::{Key, StateCookie, Tcb},
        wire::{Chunk, Gram},
    },
    random::random,
};

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

#[derive(Debug, PartialEq)]
pub enum ConnectionEvent {
    Api(ApiCall),
    Gram(Gram),
    Chunk(Chunk),
    Timer(Timer),
    InvalidGram,
}

impl From<Chunk> for ConnectionEvent {
    fn from(chunk: Chunk) -> ConnectionEvent { ConnectionEvent::Chunk(chunk) }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApiAction {
    NotifyDisconnect,
    NotifyConnectionAttemptTimedOut,
    NotifyConnectionEstablished,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectionAction {
    Timer(Timer),
    Api(ApiAction),
    Chunk { token: u32, chunk: Chunk },
    Upstream(Vec<u8>),
}

trait ConnectionState {
    fn expected_token(&self) -> Option<u32>;
    fn transition(
        self,
        event: ConnectionEvent,
    ) -> (
        Option<Connection>,
        Vec<ConnectionEvent>,
        Vec<ConnectionAction>,
    );
}

#[derive(Clone, Debug, PartialEq)]
struct Listen {
    key: Key,
}

impl ConnectionState for Listen {
    fn expected_token(&self) -> Option<u32> { None }

    fn transition(
        self,
        event: ConnectionEvent,
    ) -> (
        Option<Connection>,
        Vec<ConnectionEvent>,
        Vec<ConnectionAction>,
    ) {
        match event {
            ConnectionEvent::Chunk(Chunk::Init {
                token: their_token,
                tsn: their_tsn,
            }) => {
                let (our_tsn, our_token) = (random(), random());
                (
                    None,
                    vec![],
                    vec![ConnectionAction::Chunk {
                        token: their_token,
                        chunk: Chunk::InitAck {
                            tsn:          our_tsn,
                            token:        our_token,
                            state_cookie: StateCookie::new(
                                Tcb {
                                    our_tsn,
                                    our_token,
                                    their_token,
                                    their_tsn,
                                },
                                &self.key,
                            ),
                        },
                    }],
                )
            }
            ConnectionEvent::Api(ApiCall::Conn) => {
                let (our_token, tsn) = (random(), random());
                (
                    Some(Connection::InitSent(InitSent {
                        our_token,
                        tsn,
                        queue: Vec::new(),
                    })),
                    vec![],
                    vec![
                        ConnectionAction::Chunk {
                            token: 0,
                            chunk: Chunk::Init {
                                token: our_token,
                                tsn,
                            },
                        },
                        ConnectionAction::Timer(Timer::InitTimer),
                    ],
                )
            }
            ConnectionEvent::Chunk(Chunk::CookieEcho(state_cookie))
                if state_cookie.signed_by(&self.key) =>
            {
                (
                    Some(Connection::Established(Established {
                        tcb: state_cookie.tcb.clone(),
                    })),
                    vec![],
                    vec![
                        ConnectionAction::Chunk {
                            token: state_cookie.tcb.their_token,
                            chunk: Chunk::CookieAck,
                        },
                        ConnectionAction::Api(ApiAction::NotifyConnectionEstablished),
                    ],
                )
            }
            _ => (None, vec![], vec![]),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct InitSent {
    our_token: u32,
    tsn:       u32,
    queue:     Vec<ApiCall>,
}

impl ConnectionState for InitSent {
    fn expected_token(&self) -> Option<u32> { Some(self.our_token) }

    fn transition(
        mut self,
        event: ConnectionEvent,
    ) -> (
        Option<Connection>,
        Vec<ConnectionEvent>,
        Vec<ConnectionAction>,
    ) {
        match event {
            ConnectionEvent::Timer(Timer::InitTimer) => (
                None,
                vec![],
                vec![ConnectionAction::Api(
                    ApiAction::NotifyConnectionAttemptTimedOut,
                )],
            ),
            ConnectionEvent::Chunk(Chunk::InitAck {
                token: their_token,
                tsn: their_tsn,
                state_cookie,
            }) => (
                Some(Connection::CookieEchoed(CookieEchoed {
                    tcb:   Tcb {
                        our_tsn: self.tsn,
                        our_token: self.our_token,
                        their_tsn,
                        their_token,
                    },
                    queue: self.queue,
                })),
                vec![],
                vec![
                    ConnectionAction::Chunk {
                        token: their_token,
                        chunk: Chunk::CookieEcho(state_cookie),
                    },
                    ConnectionAction::Timer(Timer::CookieSentTimer),
                ],
            ),
            ConnectionEvent::Api(api_call) => (
                Some(Connection::InitSent(InitSent {
                    queue: {
                        self.queue.push(api_call);
                        self.queue
                    },
                    ..self
                })),
                vec![],
                vec![],
            ),
            _ => (Some(Connection::InitSent(self)), vec![], vec![]),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct CookieEchoed {
    tcb:   Tcb,
    queue: Vec<ApiCall>,
}

impl ConnectionState for CookieEchoed {
    fn expected_token(&self) -> Option<u32> { Some(self.tcb.our_token) }

    fn transition(
        mut self,
        event: ConnectionEvent,
    ) -> (
        Option<Connection>,
        Vec<ConnectionEvent>,
        Vec<ConnectionAction>,
    ) {
        match event {
            ConnectionEvent::Api(api_call) => (
                Some(Connection::CookieEchoed(CookieEchoed {
                    queue: {
                        self.queue.push(api_call);
                        self.queue
                    },
                    ..self
                })),
                vec![],
                vec![],
            ),
            ConnectionEvent::Chunk(Chunk::CookieAck) => (
                Some(Connection::Established(Established { tcb: self.tcb })),
                self.queue
                    .into_iter()
                    .map(|api_call| ConnectionEvent::Api(api_call))
                    .collect(),
                vec![ConnectionAction::Api(
                    ApiAction::NotifyConnectionEstablished,
                )],
            ),
            _ => (Some(Connection::CookieEchoed(self)), vec![], vec![]),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Established {
    tcb: Tcb,
}

impl ConnectionState for Established {
    fn expected_token(&self) -> Option<u32> { Some(self.tcb.our_token) }

    fn transition(
        self,
        event: ConnectionEvent,
    ) -> (
        Option<Connection>,
        Vec<ConnectionEvent>,
        Vec<ConnectionAction>,
    ) {
        match event {
            ConnectionEvent::Api(ApiCall::Disc) => (
                Some(Connection::ShutdownSent(ShutdownSent {
                    our_token:   self.tcb.our_token,
                    their_token: self.tcb.their_token,
                })),
                vec![],
                vec![ConnectionAction::Chunk {
                    token: self.tcb.their_token,
                    chunk: Chunk::Shutdown,
                }],
            ),
            ConnectionEvent::Chunk(Chunk::Shutdown) => (
                Some(Connection::ShutdownAckSent(ShutdownAckSent {
                    our_token:   self.tcb.our_token,
                    their_token: self.tcb.their_token,
                })),
                vec![],
                vec![ConnectionAction::Chunk {
                    token: self.tcb.their_token,
                    chunk: Chunk::ShutdownAck,
                }],
            ),
            ConnectionEvent::Chunk(Chunk::Data(payload)) => (
                Some(Connection::Established(self)),
                vec![],
                vec![ConnectionAction::Upstream(payload)],
            ),
            _ => (Some(Connection::Established(self)), vec![], vec![]),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ShutdownSent {
    our_token:   u32,
    their_token: u32,
}

impl ConnectionState for ShutdownSent {
    fn expected_token(&self) -> Option<u32> { Some(self.our_token) }

    fn transition(
        self,
        event: ConnectionEvent,
    ) -> (
        Option<Connection>,
        Vec<ConnectionEvent>,
        Vec<ConnectionAction>,
    ) {
        match event {
            ConnectionEvent::Chunk(Chunk::Shutdown) => (
                Some(Connection::ShutdownAckSent(ShutdownAckSent {
                    our_token:   self.our_token,
                    their_token: self.their_token,
                })),
                vec![],
                vec![ConnectionAction::Chunk {
                    token: self.their_token,
                    chunk: Chunk::ShutdownAck,
                }],
            ),
            ConnectionEvent::Chunk(Chunk::ShutdownAck) => (
                None,
                vec![],
                vec![
                    ConnectionAction::Api(ApiAction::NotifyDisconnect),
                    ConnectionAction::Chunk {
                        token: self.their_token,
                        chunk: Chunk::ShutdownComplete,
                    },
                ],
            ),
            _ => (Some(Connection::ShutdownSent(self)), vec![], vec![]),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ShutdownAckSent {
    our_token:   u32,
    their_token: u32,
}

impl ConnectionState for ShutdownAckSent {
    fn expected_token(&self) -> Option<u32> { Some(self.our_token) }

    fn transition(
        self,
        event: ConnectionEvent,
    ) -> (
        Option<Connection>,
        Vec<ConnectionEvent>,
        Vec<ConnectionAction>,
    ) {
        match event {
            ConnectionEvent::Chunk(Chunk::ShutdownComplete) => (
                None,
                vec![],
                vec![ConnectionAction::Api(ApiAction::NotifyDisconnect)],
            ),
            ConnectionEvent::Chunk(Chunk::ShutdownAck) => (
                None,
                vec![],
                vec![
                    ConnectionAction::Api(ApiAction::NotifyDisconnect),
                    ConnectionAction::Chunk {
                        token: self.their_token,
                        chunk: Chunk::ShutdownComplete,
                    },
                ],
            ),
            _ => (Some(Connection::ShutdownAckSent(self)), vec![], vec![]),
        }
    }
}

/// Connection is a relationship between two miknet nodes. Handshake and teardown based roughly on
/// SCTP.
#[derive(Clone, Debug, PartialEq)]
pub enum Connection {
    Listen(Listen),
    InitSent(InitSent),
    CookieEchoed(CookieEchoed),
    Established(Established),
    ShutdownSent(ShutdownSent),
    ShutdownAckSent(ShutdownAckSent),
}

impl Transducer for Connection {
    type Action = ConnectionAction;
    type Event = ConnectionEvent;

    fn transition(
        self,
        event: ConnectionEvent,
    ) -> (Option<Self>, Vec<ConnectionEvent>, Vec<ConnectionAction>) {
        match event {
            ConnectionEvent::Gram(gram) => {
                let expected_token = match self {
                    Connection::Listen(ref listen) => listen.expected_token(),
                    Connection::InitSent(ref init_sent) => init_sent.expected_token(),
                    Connection::CookieEchoed(ref cookie_echoed) => cookie_echoed.expected_token(),
                    Connection::Established(ref established) => established.expected_token(),
                    Connection::ShutdownSent(ref shutdown_sent) => shutdown_sent.expected_token(),
                    Connection::ShutdownAckSent(ref shutdown_ack_sent) => {
                        shutdown_ack_sent.expected_token()
                    }
                };
                (
                    Some(self),
                    Connection::events_from_gram(expected_token, gram),
                    vec![],
                )
            }
            event => match self {
                Connection::Listen(listen) => listen.transition(event),
                Connection::InitSent(init_sent) => init_sent.transition(event),
                Connection::CookieEchoed(cookie_echoed) => cookie_echoed.transition(event),
                Connection::Established(established) => established.transition(event),
                Connection::ShutdownSent(shutdown_sent) => shutdown_sent.transition(event),
                Connection::ShutdownAckSent(shutdown_ack_sent) => {
                    shutdown_ack_sent.transition(event)
                }
            },
        }
    }
}

impl Connection {
    /// Returns a connection using the given key to sign and verify state cookies.
    pub fn new(key: Key) -> Self { Connection::Listen(Listen { key }) }

    fn events_from_gram(expected_token: Option<u32>, gram: Gram) -> Vec<ConnectionEvent> {
        if !gram.chunks.is_empty() && expected_token.map(|t| t == gram.token).unwrap_or(true) {
            gram.chunks
                .into_iter()
                .map(|chunk| ConnectionEvent::Chunk(chunk))
                .collect()
        } else {
            vec![ConnectionEvent::InvalidGram]
        }
    }
}
