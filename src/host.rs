use bincode::{self, deserialize, serialize};
use crate::{
    api::{ApiCall, Event},
    error::{Error, Result},
    protocol::{
        connection::ConnectionEvent,
        peer::Peer,
        protocol::{Protocol, ProtocolConfig},
        wire::{Channel, Gram, Message},
    },
};
use failure_derive::Fail;
use rand::random;
use serde::Serialize;
use std::{
    collections::hash_map::HashMap,
    io,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    time::{Duration, Instant},
};

pub const MAX_GRAM_LEN: usize = 1024 * 8;

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
    #[fail(
        display = "Failed to set the underlying socket read timeout: {:?}",
        set_timeout_error
    )]
    SetSocketTimeout {
        #[cause]
        set_timeout_error: io::Error,
    },
    #[fail(
        display = "Failed to receive on the underlying socket: {:?}",
        recv_error
    )]
    ReceiveFrom {
        #[cause]
        recv_error: io::Error,
    },
}

/// A Peer is a connected miknet host.
///
/// The lifetime of this type is not gauranteed to match the lifetime of the connection to the peer.
#[derive(Copy, Clone, Debug, Hash, Eq, Ord, PartialOrd, PartialEq)]
pub struct PeerID(u32);

impl PeerID {
    fn new() -> Self { PeerID(random()) }
}

/// Host is the API handle for an endpoint of the miknet protocol.
pub struct Host {
    socket:          UdpSocket,
    protocol_config: ProtocolConfig,
    peers:           HashMap<PeerID, (SocketAddr, Peer)>,
    command_queue:   HashMap<PeerID, Vec<ApiCall>>,
}

impl Host {
    /// Binds a miknet host to a socket address.
    pub fn bind(protocol_config: ProtocolConfig, address: impl ToSocketAddrs) -> Result<Self> {
        let socket = UdpSocket::bind(address)
            .map_err(|bind_error| Error::Host(HostError::Creation { bind_error }))?;
        Ok(Self {
            socket,
            peers: HashMap::new(),
            command_queue: HashMap::new(),
            protocol_config,
        })
    }

    /// Polls underlying socket and protocol timers for events and dequeues all queued commands.
    pub fn service(&mut self, budget: Duration) -> Result<Vec<Event>> {
        let start = Instant::now();
        while let Some(remaining_budget) = budget.checked_sub(start.elapsed()) {
            self.socket
                .set_read_timeout(Some(remaining_budget))
                .map_err(|set_timeout_error| {
                    Error::Host(HostError::SetSocketTimeout { set_timeout_error })
                })?;

            let mut buffer = vec![0; MAX_GRAM_LEN];
            let (len, sender) = self
                .socket
                .recv_from(&mut buffer)
                .map_err(|recv_error| Error::Host(HostError::ReceiveFrom { recv_error }))?;

            let connection_event = match bincode::deserialize::<Gram>(&buffer) {
                Ok(gram) => ConnectionEvent::Gram(gram),
                Err(_) => ConnectionEvent::InvalidGram,
            };

            // TODO(turnage): factor into poll and hook up runner.
        }

        Ok(vec![])
    }

    /// Enqueues a connection attempt to the address.
    pub fn enqueue_connect(&mut self, address: SocketAddr) -> Result<()> {
        let peer_id = PeerID::new();
        self.peers.insert(
            peer_id,
            (address, Peer::new(Protocol::from(self.protocol_config))?),
        );
        Ok(self
            .command_queue
            .entry(peer_id)
            .or_insert(vec![])
            .push(ApiCall::Conn))
    }

    /// Enqueues a disconnection from `peer`.
    pub fn enqueue_disconnect(&mut self, peer: PeerID) {
        if let Some(mut queue) = self.command_queue.get_mut(&peer) {
            queue.push(ApiCall::Disc)
        }
    }

    /// Enqueues a send of `message` over `channel` to `peer`.
    pub fn enqueue_send(
        &mut self,
        peer: PeerID,
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
