//! wire defines on-the-wire representations of data used in miknet protocol.

use crate::protocol::validation::StateCookie;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Channel(u8);

/// A type implementing the Message trait may be transmitted over the wire.
pub trait Message: 'static + Deserialize<'static> + Serialize {}

/// Chunks are control and data messages that can be packed in a gram.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Chunk {
    Init {
        token: u32,
        tsn:   u32,
    },
    InitAck {
        token:        u32,
        tsn:          u32,
        state_cookie: StateCookie,
    },
    CookieEcho(StateCookie),
    CookieAck,
    Shutdown,
    ShutdownAck,
    ShutdownComplete,
    CfgMismatch,
    Data(Vec<u8>),
}

/// Gram is the datagram of the miknet protocol. All transmissions are
/// represented as a gram before they are written on the network.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Gram {
    pub token:  u32,
    pub chunks: Vec<Chunk>,
}
