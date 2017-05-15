use super::Protocol;
use super::Result;

use event::Event;
use packet::{Command, Packet};
use peer::{self, Peer};

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::net::{SocketAddr, UdpSocket};

#[derive(Debug)]
pub enum Target {
    All,
    Peer(peer::ID),
}

pub struct Host<P: Debug + Serialize + DeserializeOwned> {
    peer_id_gen: peer::ID,
    queue: VecDeque<(peer::ID, Packet)>,
    peers: HashMap<peer::ID, Peer>,
    socket: UdpSocket,
    protocol: Protocol,
    _payload: PhantomData<P>,
}

impl<P: Debug + Serialize + DeserializeOwned> Host<P> {
    pub fn new(protocol: Protocol, address: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(address)?;
        let _ = socket.set_nonblocking(true)?;
        let _ = socket.set_broadcast(true)?;
        Ok(Host {
            peer_id_gen: Default::default(),
            queue: VecDeque::new(),
            peers: HashMap::new(),
            socket: socket,
            protocol: protocol,
            _payload: PhantomData,
        })
    }

    /// Queues a connection to an address which will be attempted on the next
    /// host service. Returns the ID of the new peer.
    pub fn connect(&mut self, addr: SocketAddr) -> peer::ID {
        let peer = self.add_peer(addr);
        self.queue.push_back((peer, Packet::Command(Command::Connect(self.protocol.clone()))));
        peer
    }

    pub fn send(&mut self, target: Target, payload: P) -> Result<()> { Ok(()) }
    pub fn service(&mut self) -> Option<Result<Event<P>>> { None }
    pub fn disconnect(&mut self, target: Target) -> Result<()> { Ok(()) }

    /// Adds a peer in a connecting state, returning the new peer's ID.
    fn add_peer(&mut self, addr: SocketAddr) -> peer::ID {
        let peer = self.peer_id_gen;
        self.peers.insert(peer, Peer::new(addr));
        self.peer_id_gen = peer::next(self.peer_id_gen);
        peer
    }
}
