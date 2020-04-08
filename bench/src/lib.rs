#![recursion_limit = "512"]

use async_std::net::SocketAddr;
use nhanh::*;
use serde::{Deserialize, Serialize};

use structopt::StructOpt;

pub mod enet;
pub mod kcp;
pub mod tcp;

pub mod client;
pub mod runner;
pub mod server;

pub const ALL_PROTOCOLS: [Protocol; 3] =
    [Protocol::Tcp, Protocol::Enet, Protocol::Kcp];

pub const ID_DO_NOT_RETURN: u64 = u64::max_value();

pub fn default_server_address() -> SocketAddr {
    "127.0.0.1:33333".parse().unwrap()
}

/// Returns a stream that yields `()` `hertz` times per second.
pub fn ticker(hertz: u32) -> impl futures::stream::Stream<Item = ()> {
    use futures::stream::StreamExt;

    let tick_rate = std::time::Duration::from_secs(1) / hertz;
    futures::stream::repeat(0u8)
        .then(move |_| futures_timer::Delay::new(tick_rate))
}

#[derive(Serialize, Eq, PartialEq, Hash, Copy, Clone, Debug, StructOpt)]
pub enum Protocol {
    Tcp,
    Enet,
    Kcp,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkDatagram {
    pub delivery_mode: DeliveryMode,
    pub id: u64,
    pub data: Vec<u8>,
}
