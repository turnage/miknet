#![recursion_limit = "256"]

use async_std::net::SocketAddr;
use nhanh::*;
use serde::{Deserialize, Serialize};

use structopt::StructOpt;

pub mod enet;
pub mod tcp;

pub mod client;
pub mod runner;
pub mod server;

pub const ALL_PROTOCOLS: [Protocol; 2] = [Protocol::Tcp, Protocol::Enet];

pub const ID_DO_NOT_RETURN: u64 = u64::max_value();

pub fn default_server_address() -> SocketAddr {
    "127.0.0.1:33333".parse().unwrap()
}

#[derive(Serialize, Eq, PartialEq, Hash, Copy, Clone, Debug, StructOpt)]
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
