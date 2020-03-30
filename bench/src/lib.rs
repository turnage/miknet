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

#[derive(Debug, StructOpt)]
pub enum Protocol {
    Tcp,
    Enet,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkDatagram {
    pub delivery_mode: DeliveryMode,
    pub id: u64,
    pub data: Vec<u8>,
}
