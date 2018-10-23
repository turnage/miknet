//! User api.

use std::net::SocketAddr;

#[derive(Clone, Debug, PartialEq)]
pub enum ApiCall {
    Tx(Vec<u8>),
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
