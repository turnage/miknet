//! peer defines a miknet host's view of other hosts

use std::net::SocketAddr;

#[derive(Debug, PartialEq)]
pub struct Peer {
    addr: SocketAddr,
}

#[derive(Debug, PartialEq)]
pub enum Dest {
    Peer(Peer),
    All,
}
