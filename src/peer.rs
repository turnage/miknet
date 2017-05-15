use std::fmt::Debug;
use std::net::SocketAddr;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ID(usize);

impl Default for ID {
    fn default() -> Self { ID(0) }
}

pub fn next(id: ID) -> ID {
    let ID(val) = id;
    ID(val + 1)
}

#[derive(Debug)]
pub enum State {
    Connecting,
    Connected,
    Disconnecting,
}

#[derive(Debug)]
pub struct Peer {
    state: State,
    addr: SocketAddr,
}

impl Peer {
    pub fn new(addr: SocketAddr) -> Self {
        Peer {
            state: State::Connecting,
            addr: addr,
        }
    }
}
