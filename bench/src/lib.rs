#[macro_use]
extern crate rental;

use nhanh::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use structopt::StructOpt;

pub mod enet;
pub mod tcp;

pub mod client;
pub mod server;

#[derive(Copy, Clone, Debug, StructOpt)]
pub enum Protocol {
    Tcp,
    Enet,
    All,
}

impl IntoIterator for Protocol {
    type Item = Self;
    type IntoIter = <Vec<Protocol> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Protocol::All => vec![Protocol::Tcp, Protocol::Enet],
            _ => vec![self],
        }
        .into_iter()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkDatagram {
    pub delivery_mode: DeliveryMode,
    pub id: u64,
    pub data: Vec<u8>,
}
