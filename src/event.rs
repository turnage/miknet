//! event defines events, the atomic temporal unit of the miknet protocol.

use gram;
use peer::Dest;
use std::net::SocketAddr;

#[derive(Debug, PartialEq)]
pub enum Api {
    Tx(Vec<u8>),
    Disc,
    Conn,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Api(Api),
    Ctrl(gram::Ctrl),
    Frag(gram::Frag),
    InvalidGram,
}
