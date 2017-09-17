//! Manages multiple connections for a node.

use {Error, Result};
use cmd::Cmd;
use conn::miklow::{Connection, Key, StateCookie};
use event::{Api, Event};
use futures::{Async, Poll, Stream};
use futures::future::ok;
use futures::stream::iter;
use futures::unsync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use itertools::Itertools;
use std::collections::hash_map::HashMap;
use std::net::SocketAddr;

pub struct ConnectionManager {
    key: Key,
    conns: HashMap<SocketAddr, Connection>,
}

impl ConnectionManager {
    pub fn pipe<'a, S>(src: S) -> Result<Box<Stream<Item = (SocketAddr, Cmd), Error = Error> + 'a>>
    where
        S: Stream<Item = (Option<SocketAddr>, Event), Error = Error> + 'a,
    {
        let mut cm = Self::new()?;
        Ok(Box::new(
            src.take_while(|&(_, ref event)| match *event {
                Event::Api(Api::Shutdown) => ok(false),
                _ => ok(true),
            }).map(|pair| pair)
                .filter_map(move |e| match e {
                    (Some(peer), event) => Some(cm.receive(peer, event)),
                    _ => None,
                })
                .map(|cmds| iter(cmds.into_iter().map(|cmd| Ok(cmd))))
                .flatten(),
        ))
    }

    fn new() -> Result<Self> { Ok(Self { key: Key::new()?, conns: HashMap::new() }) }

    fn receive(&mut self, peer: SocketAddr, event: Event) -> Vec<(SocketAddr, Cmd)> {
        let (conn, cmds) = self.conns
            .remove(&peer)
            .unwrap_or_else(|| Connection::new(self.key.clone()))
            .step(peer, event);
        if conn.should_persist() {
            self.conns.insert(peer, conn);
        }
        cmds
    }
}
