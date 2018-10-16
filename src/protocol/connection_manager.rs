use failure::Error;
use itertools::{Either, Itertools};
use std::collections::hash_map::HashMap;
use std::net::SocketAddr;

use crate::protocol::connection::{Connection, WireCmd, WireEvent};
use crate::protocol::protocol::{Protocol, ProtocolBuilder};
use crate::protocol::validation::Key;

pub struct ConnectionManager {
    key:              Key,
    conns:            HashMap<SocketAddr, (Connection, Protocol)>,
    protocol_builder: ProtocolBuilder,
}

impl ConnectionManager {
    pub fn new(protocol_builder: ProtocolBuilder) -> Result<Self, Error> {
        Ok(Self {
            key: Key::new()?,
            conns: HashMap::new(),
            protocol_builder,
        })
    }

    pub fn step(
        &mut self,
        peer: SocketAddr,
        event: WireEvent,
    ) -> Vec<(SocketAddr, WireCmd)> {
        let (conn, protocol) = self.conns.remove(&peer).unwrap_or_else(|| {
            (
                Connection::new(self.key.clone()),
                self.protocol_builder.build(),
            )
        });
        let (conn, cmds) = conn.step(event);
        let (cmds, upstream_events): (Vec<WireCmd>, Vec<Vec<u8>>) =
            cmds.into_iter().partition_map(|cmd| match cmd {
                WireCmd::Upstream(payload) => Either::Right(payload),
                cmd => Either::Left(cmd),
            });
        if conn.should_persist() {
            self.conns.insert(peer, (conn, protocol));
        }
        cmds.into_iter().map(|cmd| (peer, cmd)).collect()
    }
}
