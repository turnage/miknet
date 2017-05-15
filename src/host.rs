use super::Protocol;
use super::Result;

use event;
use peer;

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::net::{SocketAddr, UdpSocket};

#[derive(Debug)]
pub enum Target {
    All,
    Peer(peer::ID),
}

pub struct Host<P: Debug + Serialize + DeserializeOwned> {
    socket: UdpSocket,
    _payload: PhantomData<P>,
}

impl<P: Debug + Serialize + DeserializeOwned> Host<P> {
    pub fn new(protocol: Protocol, address: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(address)?;
        let _ = socket.set_nonblocking(true)?;
        let _ = socket.set_broadcast(true)?;
        Ok(Host {
            socket: socket,
            _payload: PhantomData,
        })
    }
    pub fn send(&mut self, target: Target, payload: P) -> Result<()> { Ok(()) }
    pub fn service(&mut self) -> Option<Result<event::Event<P>>> { None }
    pub fn disconnect(&mut self, target: Target) -> Result<()> { Ok(()) }
}
