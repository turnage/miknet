//! event defines events, the atomic temporal unit of the miknet protocol.

use gram::Chunk;

#[derive(Debug, PartialEq)]
pub enum Api {
    Tx(Vec<u8>),
    Disc,
    Conn,
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Api(Api),
    Chunk(Chunk),
    InvalidGram,
}
