//! A node is the host unit of the miknet protocol.

use failure::Error;
use std::collections::hash_map::HashMap;
use std::net::SocketAddr;

use crate::connection::validation::Key;
use crate::connection::wire::{Cmd, Connection, Event, Protocol};

pub struct ConnectionManager<P: Protocol> {
    key:   Key,
    conns: HashMap<SocketAddr, Connection<P>>,
}

impl<P: Protocol> ConnectionManager<P> {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            key:   Key::new()?,
            conns: HashMap::new(),
        })
    }

    pub fn step(
        &mut self,
        peer: SocketAddr,
        event: Event,
    ) -> Vec<(SocketAddr, Cmd)> {
        let (conn, cmds) = self
            .conns
            .remove(&peer)
            .unwrap_or_else(|| Connection::new(self.key.clone()))
            .step(event);
        if conn.should_persist() {
            self.conns.insert(peer, conn);
        }
        cmds.into_iter().map(|cmd| (peer, cmd)).collect()
    }
}
