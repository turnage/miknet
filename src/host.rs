use bincode::{self, serialize};
use crate::{
    api::ApiCall,
    error::{Error, Result},
    protocol::{peer::Peer, protocol::Protocol},
};
use failure_derive::Fail;
use serde::Serialize;
use std::{
    collections::hash_map::HashMap,
    io,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
};

#[derive(Fail, Debug)]
pub enum HostError {
    #[fail(
        display = "Failed to bind socket for new host: {:?}",
        bind_error
    )]
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

#[derive(Copy, Clone, Debug, Hash, Eq, Ord, PartialOrd, PartialEq)]
pub struct PeerID(u32);

pub struct PeerHandle<'a> {
    peer: &'a mut Peer,
}

impl<'a> PeerHandle<'a> {
    pub fn new(peer: &'a mut Peer) -> PeerHandle<'a> { Self { peer } }

    pub fn send(self, data: impl Serialize) -> Result<PeerHandle<'a>> {
        self.peer
            .enqueue_api_call(ApiCall::Tx(bincode::serialize(&data).map_err(
                |serialization_error| {
                    Error::Host(HostError::PayloadSerialization {
                        serialization_error,
                    })
                },
            )?));
        Ok(self)
    }
}

pub struct Host {
    addr:  UdpSocket,
    peers: HashMap<PeerID, (SocketAddr, Peer)>,
}

impl Host {
    pub fn new(addr: impl ToSocketAddrs) -> Result<Self> {
        let addr = UdpSocket::bind(addr)
            .map_err(|bind_error| Error::Host(HostError::Creation { bind_error }))?;
        Ok(Self {
            addr,
            peers: HashMap::new(),
        })
    }

    fn peer<'a>(&'a mut self, peer_id: PeerID) -> Option<PeerHandle<'a>> {
        self.peers
            .get_mut(&peer_id)
            .map(|(_, peer)| PeerHandle::new(peer))
    }
}
