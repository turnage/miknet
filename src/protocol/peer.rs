use failure::Error;
use itertools::{Either, Itertools};
use std::{collections::hash_map::HashMap, net::SocketAddr};

use crate::{
    api::ApiCall,
    protocol::{
        connection::{Connection, ConnectionAction, ConnectionEvent},
        protocol::Protocol,
        transducer::Transducer,
        validation::Key,
    },
};

pub struct Peer {
    key:            Key,
    connection:     Option<Connection>,
    protocol:       Protocol,
    api_call_queue: Vec<ApiCall>,
}

impl Peer {
    pub fn new(protocol: Protocol) -> Result<Self, Error> {
        Ok(Self {
            key: Key::new()?,
            connection: None,
            protocol,
            api_call_queue: vec![],
        })
    }

    pub fn enqueue_api_call(&mut self, api_call: ApiCall) { self.api_call_queue.push(api_call); }

    pub fn step(self, events: Vec<ConnectionEvent>) -> (Self, Vec<ConnectionAction>) {
        let connection = self.connection.unwrap_or(Connection::new(self.key.clone()));
        let (connection, actions) = connection.transduce(events);
        let (actions, upstream_events): (Vec<ConnectionAction>, Vec<Vec<u8>>) =
            actions.into_iter().partition_map(|action| match action {
                ConnectionAction::Upstream(payload) => Either::Right(payload),
                action => Either::Left(action),
            });
        (Self { connection, ..self }, actions)
    }
}
