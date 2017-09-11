//! miklow is the lower level component of the miknet protocol responsible for establishing and
//! tearing down connections.

use {MEvent, Result};
use bincode::{Infinite, serialize};
use cmd::Cmd;
use crypto::hmac::Hmac;
use crypto::mac::{Mac, MacResult};
use crypto::sha3::Sha3;
use event::{Api, Event};
use gram::{Chunk, Gram};
use rand::{OsRng, Rng};
use rand::random;
use std::net::SocketAddr;
use timers::Timer;

/// Key is a crytographic key used to authenticate state cookies.
#[derive(Clone, Debug, PartialEq)]
pub struct Key {
    bytes: [u8; Key::BYTES],
}

impl Key {
    const BYTES: usize = 32;

    /// Returns a new key using random bytes from the OS Rng.
    pub fn new() -> Result<Self> {
        let mut rng = OsRng::new()?;
        let mut bytes = [0; Key::BYTES];
        rng.fill_bytes(&mut bytes);
        Ok(Key { bytes })
    }
}

/// State cookies are used in the four way connection handshake. Usage is based on SCTP; look there
/// for further information.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateCookie {
    tcb: Tcb,
    hmac: [u8; Key::BYTES],
}

impl StateCookie {
    /// Creates a new state cookie signed by the given key.
    pub fn new(tcb: Tcb, key: &Key) -> Self { Self { tcb, hmac: tcb.hmac(key) } }

    /// Returns true if the state cookie was signed using the given key. Uses invariable time
    /// comparison.
    pub fn valid(&self, key: &Key) -> bool {
        MacResult::new(&self.hmac) == MacResult::new(&self.tcb.hmac(key))
    }

    /// Consumes the state cookie and yield the Tcb.
    pub fn tcb(self) -> Tcb { self.tcb }
}

/// Tcb contains all the information needed to manage an established connection.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Tcb {
    pub our_tsn: u32,
    pub our_token: u32,
    pub their_tsn: u32,
    pub their_token: u32,
}

impl Tcb {
    /// Returns an HMAC for the tcb content using the key.
    fn hmac(&self, key: &Key) -> [u8; Key::BYTES] {
        let mut hmac_gen = Hmac::new(Sha3::sha3_256(), &key.bytes);
        hmac_gen.input(&serialize(self, Infinite).unwrap());

        let mut hmac = [0; Key::BYTES];
        hmac.copy_from_slice(&hmac_gen.result().code());

        hmac
    }
}

/// Connection is a relationship between two miknet nodes. Handshake and teardown based roughly on
/// SCTP.
#[derive(Clone, Debug, PartialEq)]
pub enum Connection {
    Listen { key: Key },
    InitSent { token: u32, tsn: u32 },
    InitAckSent { their_token: u32 },
    CookieEchoed(Tcb),
    Established(Tcb),
    ShutdownSent { token: u32, their_token: u32 },
    ShutdownAckSent { token: u32, their_token: u32 },
    Shutdown,
    Failed,
}

impl Connection {
    /// Returns a connection using the given key to sign and verify state cookies.
    pub fn new(key: Key) -> Self { Connection::Listen { key: key } }

    /// ready_for returns whether this connection is capable of processing this event (yet). If not
    /// the client may choose to buffer the event and deliver it when this connection can process
    /// it.
    pub fn ready_for(&self, event: &Event) -> bool {
        match *event {
            Event::Api(Api::Tx(_)) |
            Event::Api(Api::Disc) => {
                match *self {
                    Connection::Established(_) => true,
                    _ => false,
                }
            }
            _ => true,
        }
    }

    /// Processes an event and returns the next state of the connection and any commands that
    /// should be executed as part of the transition.
    pub fn gen_cmds(self, peer: SocketAddr, event: Event) -> (Self, Vec<(SocketAddr, Cmd)>) {
        let (next, cmds) = self.step(event, peer);
        let token = next.token();
        (
            next,
            cmds.into_iter()
                .map(|cmd| {
                    (
                        peer.clone(),
                        match cmd {
                            Cmd::Chunk(chunk) => {
                                Cmd::Net(
                                    serialize(&Gram { token, chunks: vec![chunk] }, Infinite)
                                        .unwrap(),
                                )
                            }
                            _ => cmd,
                        },
                    )
                })
                .collect(),
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

    /// Returns the token this connection should embed in all its grams for association.
    fn token(&self) -> u32 {
        match *self {
            Connection::InitAckSent { their_token } => their_token,
            Connection::InitSent { token, .. } => token,
            Connection::CookieEchoed(tcb) => tcb.their_token,
            Connection::Established(tcb) => tcb.their_token,
            Connection::ShutdownSent { their_token, .. } => their_token,
            Connection::ShutdownAckSent { their_token, .. } => their_token,
            _ => 0,
        }
    }

    fn steps(self, events: Vec<Event>, peer: SocketAddr) -> (Self, Vec<Cmd>) {
        events.into_iter().fold(
            (self, Vec::new()),
            |(conn, mut cmds), event| {
                let (next_conn, more_cmds) = conn.step(event, peer.clone());
                cmds.extend(more_cmds);
                (next_conn, cmds)
            },
        )
    }

    fn step(self, event: Event, peer: SocketAddr) -> (Self, Vec<Cmd>) {
        match (self, event) {
            (conn, Event::Gram(gram)) => {
                let expected_token = conn.expected_token();
                conn.steps(gram.events(expected_token), peer)
            } 
            (Connection::Listen { key },
             Event::Chunk(Chunk::Init { token: their_token, tsn: their_tsn })) => {
                let (our_tsn, our_token) = (random(), random());
                (
                    Connection::InitAckSent { their_token },
                    vec![
                        Cmd::Chunk(Chunk::InitAck {
                            tsn: our_tsn,
                            token: our_token,
                            state_cookie: StateCookie::new(
                                Tcb { our_tsn, our_token, their_token, their_tsn },
                                &key,
                            ),
                        }),
                    ],
                )
            }
            (Connection::Listen { .. }, Event::Api(Api::Conn)) => {
                let (token, tsn) = (random(), random());
                (
                    Connection::InitSent { token, tsn },
                    vec![
                        Cmd::Chunk(Chunk::Init { token, tsn }),
                        Cmd::Timer(Timer::InitTimer),
                    ],
                )
            }
            (Connection::InitSent { .. }, Event::Timer(Timer::InitTimer)) => {
                (Connection::Failed, vec![Cmd::User(MEvent::ConnectionAttemptTimedOut(peer))])
            }
            (Connection::InitSent { token: our_token, tsn: our_tsn },
             Event::Chunk(Chunk::InitAck { token: their_token, tsn: their_tsn, state_cookie })) => {
                (
                    Connection::CookieEchoed(Tcb { our_tsn, our_token, their_tsn, their_token }),
                    vec![
                        Cmd::Chunk(Chunk::CookieEcho(state_cookie)),
                        Cmd::Timer(Timer::CookieSentTimer),
                    ],
                )
            }
            (Connection::CookieEchoed(tcb), Event::Chunk(Chunk::CookieAck)) => {
                (Connection::Established(tcb), vec![Cmd::User(MEvent::ConnectionEstablished(peer))])
            }
            (Connection::Listen { ref key }, Event::Chunk(Chunk::CookieEcho(state_cookie)))
                if state_cookie.valid(&key) => {
                (
                    Connection::Established(state_cookie.tcb()),
                    vec![
                        Cmd::Chunk(Chunk::CookieAck),
                        Cmd::User(MEvent::ConnectionEstablished(peer)),
                    ],
                )
            }
            (Connection::Established(tcb), Event::Api(Api::Disc)) => {
                (
                    Connection::ShutdownSent { token: tcb.our_token, their_token: tcb.their_token },
                    vec![Cmd::Chunk(Chunk::Shutdown)],
                )
            }
            (Connection::Established(tcb), Event::Chunk(Chunk::Shutdown)) => {
                (
                    Connection::ShutdownAckSent { token: tcb.our_token, their_token: tcb.their_token },
                    vec![Cmd::Chunk(Chunk::ShutdownAck)],
                )
            }
            (Connection::ShutdownSent { token, their_token }, Event::Chunk(Chunk::Shutdown)) => {
                (
                    Connection::ShutdownAckSent { token, their_token },
                    vec![Cmd::Chunk(Chunk::ShutdownAck)],
                )
            }
            (Connection::ShutdownSent { .. }, Event::Chunk(Chunk::ShutdownAck)) => {
                (Connection::Shutdown, vec![Cmd::Chunk(Chunk::ShutdownComplete)])
            }
            (Connection::ShutdownAckSent { .. }, Event::Chunk(Chunk::ShutdownComplete)) => {
                (Connection::Shutdown, Vec::new())
            }
            (Connection::ShutdownAckSent { .. }, Event::Chunk(Chunk::ShutdownAck)) => {
                (Connection::Shutdown, vec![Cmd::Chunk(Chunk::ShutdownComplete)])
            }
            (conn, _) => (conn, Vec::new()),
        }
    }

    /// Returns the token we expect in valid grams from our peer.
    fn expected_token(&self) -> Option<u32> {
        match *self {
            Connection::InitSent { token, .. } => Some(token),
            Connection::CookieEchoed(tcb) => Some(tcb.our_token),
            Connection::Established(tcb) => Some(tcb.our_token),
            Connection::ShutdownSent { token, .. } => Some(token),
            Connection::ShutdownAckSent { token, .. } => Some(token),
            _ => None,
        }
    }
}
