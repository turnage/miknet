//! User api.

use crate::protocol::wire::Channel;
use std::net::SocketAddr;

#[derive(Clone, Debug, PartialEq)]
pub enum ApiCall {
    Tx { payload: Vec<u8>, channel: Channel },
    Disc,
    Conn,
    Shutdown,
}

#[derive(Eq, Clone, Debug, PartialEq)]
pub enum Event {
    ConnectionAttemptTimedOut(SocketAddr),
    ConnectionCfgMismatch(SocketAddr),
    ConnectionEstablished(SocketAddr),
    Disconnect(SocketAddr),
    Error(String),
    Shutdown,
}
