//! User api.

use std::net::SocketAddr;

#[derive(Eq, Clone, Debug, PartialEq)]
pub enum Event {
    ConnectionAttemptTimedOut(SocketAddr),
    ConnectionCfgMismatch(SocketAddr),
    ConnectionEstablished(SocketAddr),
    Disconnect(SocketAddr),
    Error(String),
    Shutdown,
}
