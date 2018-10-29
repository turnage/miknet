use bincode::{self, serialize};
use crate::{
    api::{ApiCall, Event},
    error::{Error, Result},
    protocol::{
        protocol::{Protocol, ProtocolConfig},
        wire::{Channel, Message},
    },
};
use failure_derive::Fail;
use rand::random;
use serde::Serialize;
use std::{
    collections::hash_map::HashMap,
    io,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    time::Instant,
};

#[derive(Fail, Debug)]
pub enum HostError {
    #[fail(display = "Failed to bind socket for new host: {:?}", bind_error)]
    Creation {
        #[cause]
        bind_error: io::Error,
    },
    #[fail(
        display = "Failed to serialize payload to send peer: {:?}",
        serialization_error
    )]
    PayloadSerialization {
        #[cause]
        serialization_error: bincode::Error,
    },
}

/// A Peer is a connected miknet host.
///
/// The lifetime of this type is not gauranteed to match the lifetime of the connection to the peer.
#[derive(Copy, Clone, Debug, Hash, Eq, Ord, PartialOrd, PartialEq)]
pub struct Peer(u32);

impl Peer {
    fn new() -> Self { Peer(random()) }
}

/// Host is the API handle for an endpoint of the miknet protocol.
pub struct Host {
    address:         UdpSocket,
    protocol_config: ProtocolConfig,
    peers:           HashMap<Peer, (SocketAddr, Protocol)>,
    command_queue:   HashMap<Peer, Vec<ApiCall>>,
}

impl Host {
    /// Binds a miknet host to a socket address.
    pub fn bind(protocol_config: ProtocolConfig, address: impl ToSocketAddrs) -> Result<Self> {
        let address = UdpSocket::bind(address)
            .map_err(|bind_error| Error::Host(HostError::Creation { bind_error }))?;
        Ok(Self {
            address,
            peers: HashMap::new(),
            command_queue: HashMap::new(),
            protocol_config,
        })
    }

    /// Polls underlying socket and protocol timers for events and dequeues all queued commands.
    pub fn service(&mut self, deadline: Instant) -> Vec<Event> { vec![] }

    /// Enqueues a connection attempt to the address.
    pub fn enqueue_connect(&mut self, address: SocketAddr) {
        let peer = Peer::new();
        self.peers
            .insert(peer, (address, Protocol::from(self.protocol_config)));
        self.command_queue
            .entry(peer)
            .or_insert(vec![])
            .push(ApiCall::Conn);
    }

    /// Enqueues a disconnection from `peer`.
    pub fn enqueue_disconnect(&mut self, peer: Peer) {
        if let Some(mut queue) = self.command_queue.get_mut(&peer) {
            queue.push(ApiCall::Disc)
        }
    }

    /// Enqueues a send of `message` over `channel` to `peer`.
    pub fn enqueue_send(
        &mut self,
        peer: Peer,
        channel: Channel,
        message: impl Message,
    ) -> Result<()> {
        if let Some(mut queue) = self.command_queue.get_mut(&peer) {
            queue.push(ApiCall::Tx {
                payload: bincode::serialize(&message).map_err(|serialization_error| {
                    Error::Host(HostError::PayloadSerialization {
                        serialization_error,
                    })
                })?,
                channel,
            });
        }
        Ok(())
    }

    /// Polls underlying socket and protocol timers for any events.
    fn poll(&mut self, deadline: Instant) -> Vec<Event> { vec![] }

    /// Executes all queued commands.
    fn deque_commands(&mut self) -> Vec<Event> { vec![] }
}
