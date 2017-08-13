//! event defines events, the atomic temporal unit of the miknet protocol.

use gram;
use std::net::SocketAddr;

#[derive(Debug, PartialEq)]
pub enum ProtoError {
    InvalidGram,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Ctrl(gram::Ctrl),
    Payload(Vec<u8>),
    ProtoError(ProtoError),
}
