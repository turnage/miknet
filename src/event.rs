//! event defines events, the atomic temporal unit of the miknet protocol.

use gram;
use peer::Dest;
use std::net::SocketAddr;

#[derive(Debug, PartialEq)]
pub enum ProtoError {
    InvalidGram,
}

#[derive(Debug, PartialEq)]
pub enum Api {
    Tx(Dest, Vec<u8>),
    Disc(Dest),
    Conn(SocketAddr),
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Api(Api),
    Ctrl(gram::Ctrl),
    Payload(Vec<u8>),
    ProtoError(ProtoError),
}
