use failure::Error;
use itertools::{Either, Itertools};
use std::{collections::hash_map::HashMap, net::SocketAddr};

use crate::protocol::{
    connection::{Connection, ConnectionAction, ConnectionEvent},
    protocol::{Protocol, ProtocolBuilder},
    transducer::Transducer,
    validation::Key,
};

pub struct Host {
    key:              Key,
    conns:            HashMap<SocketAddr, (Connection, Protocol)>,
    protocol_builder: ProtocolBuilder,
}

impl Host {
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
        event: ConnectionEvent,
    ) -> Vec<(SocketAddr, ConnectionAction)> {
        let (conn, protocol) = self.conns.remove(&peer).unwrap_or_else(|| {
            (
                Connection::new(self.key.clone()),
                self.protocol_builder.build(),
            )
        });
        let (conn, actions) = conn.transduce(vec![event]);
        let (actions, upstream_events): (Vec<ConnectionAction>, Vec<Vec<u8>>) =
            actions.into_iter().partition_map(|action| match action {
                ConnectionAction::Upstream(payload) => Either::Right(payload),
                action => Either::Left(action),
            });
        if let Some(conn) = conn {
            self.conns.insert(peer, (conn, protocol));
        }
        actions.into_iter().map(|action| (peer, action)).collect()
    }
}
