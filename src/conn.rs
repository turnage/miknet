//! Connections.

use {Error, MEvent, Result};
use bincode::{Infinite, serialize};
use cmd::Cmd;
use crypto::hmac::Hmac;
use crypto::mac::{Mac, MacResult};
use crypto::sha3::Sha3;
use event::{Api, Event};
use futures::{Async, Poll, Stream};
use gram::{Chunk, Gram};
use rand::{OsRng, Rng, random};
use std::collections::hash_map::HashMap;
use std::net::SocketAddr;
use timers::Timer;

pub trait EventPipe: Stream<Item = (Option<SocketAddr>, Event), Error = Error> {}

impl<P: Stream<Item = (Option<SocketAddr>, Event), Error = Error>> EventPipe for P {}

#[derive(Clone, Debug)]
pub struct ConnectionManager<P: EventPipe> {
    source: P,
    key: Key,
    connections: HashMap<SocketAddr, Connection>,
    outbuf: Vec<(SocketAddr, Cmd)>,
}

impl<P: EventPipe> ConnectionManager<P> {
    pub fn new(source: P) -> Result<Self> {
        Ok(Self { source, key: Key::new()?, connections: HashMap::new(), outbuf: Vec::new() })
    }

    fn receive(&mut self, sender: SocketAddr, event: Event) -> Vec<(SocketAddr, Cmd)> {
        let (conn, cmds) = if let Some(conn) = self.connections.remove(&sender) {
            conn.gen_cmds(event, sender)
        } else {
            Connection::Listen { key: self.key.clone() }.gen_cmds(event, sender)
        };
        if conn.should_persist() {
            self.connections.insert(sender, conn);
        }
        cmds
    }
}

impl<P: EventPipe> Stream for ConnectionManager<P> {
    type Item = (SocketAddr, Cmd);
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Some(_) = self.outbuf.first() {
            Ok(Async::Ready(Some(self.outbuf.remove(0))))
        } else {
            match self.source.poll() {
                Ok(Async::Ready(Some((Some(addr), event)))) => {
                    let mut commands = self.receive(addr, event);
                    self.outbuf.append(&mut commands);
                    self.poll()
                }
                Ok(Async::Ready(Some((None, event)))) => {
                    match event {
                        Event::Api(Api::Shutdown) => Ok(Async::Ready(None)),
                        _ => Err(
                            format!("Unknown event without address in pipline: {:?}", event)
                                .into(),
                        ),
                    }
                }

                Ok(Async::Ready(None)) => Ok(Async::Ready(None)),
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(e) => Err(e),
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Key {
    bytes: [u8; Key::BYTES],
}

impl Key {
    const BYTES: usize = 32;

    fn new() -> Result<Self> {
        let mut rng = OsRng::new()?;
        let mut bytes = [0; Key::BYTES];
        rng.fill_bytes(&mut bytes);
        Ok(Key { bytes })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StateCookie {
    tcb: Tcb,
    hmac: [u8; Key::BYTES],
}

impl StateCookie {
    fn new(tcb: Tcb, key: &Key) -> Self { Self { tcb, hmac: tcb.hmac(key) } }

    fn valid(&self, key: &Key) -> bool {
        MacResult::new(&self.hmac) == MacResult::new(&self.tcb.hmac(key))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct Tcb {
    our_tsn: u32,
    our_token: u32,
    their_tsn: u32,
    their_token: u32,
}

impl Tcb {
    fn hmac(&self, key: &Key) -> [u8; Key::BYTES] {
        let mut hmac_gen = Hmac::new(Sha3::sha3_256(), &key.bytes);
        hmac_gen.input(&serialize(self, Infinite).unwrap());

        let mut hmac = [0; Key::BYTES];
        hmac.copy_from_slice(&hmac_gen.result().code());

        hmac
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Connection {
    Listen { key: Key },
    InitSent { token: u32, tsn: u32 },
    InitAckSent { their_token: u32 },
    CookieEchoed(Tcb),
    Established(Tcb),
    ShutdownSent { token: u32, their_token: u32 },
    ShutdownAckSent { token: u32, their_token: u32 },
    Failed,
}

impl Connection {
    fn new(key: Key) -> Self { Connection::Listen { key: key } }

    fn gen_cmds(self, event: Event, peer: SocketAddr) -> (Self, Vec<(SocketAddr, Cmd)>) {
        let (next, cmds) = self.step(event, peer);
        (
            next.clone(),
            cmds.into_iter()
                .map(|cmd| {
                    (
                        peer.clone(),
                        match cmd {
                            Cmd::Chunk(chunk) => {
                                next.token().map(|token| {
                                    Cmd::Net(
                                        serialize(&Gram { token, chunks: vec![chunk] }, Infinite)
                                            .unwrap(),
                                    )
                                })
                            }
                            cmd => Some(cmd),
                        },
                    )
                })
                .filter_map(|(peer, maybe_cmd)| maybe_cmd.map(move |cmd| (peer, cmd)))
                .collect(),
        )
    }

    fn token(&self) -> Option<u32> {
        match *self {
            Connection::Listen { .. } => Some(0),
            Connection::InitAckSent { their_token } => Some(their_token),
            Connection::InitSent { token, .. } => Some(token),
            Connection::CookieEchoed(tcb) => Some(tcb.their_token),
            Connection::Established(tcb) => Some(tcb.their_token),
            Connection::ShutdownSent { their_token, .. } => Some(their_token),
            Connection::ShutdownAckSent { their_token, .. } => Some(their_token),
            _ => None,
        }
    }

    fn step(self, event: Event, peer: SocketAddr) -> (Self, Vec<Cmd>) {
        match (self, event) {
            (conn, Event::Gram(gram)) => {
                let events = conn.events_from(gram);
                conn.steps(events, peer)
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
                    Connection::Established(state_cookie.tcb),
                    vec![
                        Cmd::Chunk(Chunk::CookieAck),
                        Cmd::User(MEvent::ConnectionEstablished(peer)),
                    ],
                )
            }
            (conn, _) => (conn, Vec::new()),
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

    fn events_from(&self, gram: Gram) -> Vec<Event> {
        match *self {
            Connection::Listen { .. } => gram.events(None),
            Connection::InitSent { token, .. } => gram.events(Some(token)),
            Connection::CookieEchoed(tcb) => gram.events(Some(tcb.our_token)),
            Connection::Established(tcb) => gram.events(Some(tcb.our_token)),
            Connection::ShutdownSent { token, .. } => gram.events(Some(token)),
            Connection::ShutdownAckSent { token, .. } => gram.events(Some(token)),
            _ => Vec::new(),
        }
    }

    fn should_persist(&self) -> bool {
        match *self {
            Connection::Listen { .. } => false,
            Connection::InitAckSent { .. } => false,
            Connection::Failed => false,
            _ => true,
        }
    }
}
