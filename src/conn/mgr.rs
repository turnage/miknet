//! Manages multiple connections for a node.

use {Error, Result};
use cmd::Cmd;
use conn::Config;
use conn::miklow::{Connection, Key};
use event::{Api, Event};
use futures::Stream;
use futures::future::ok;
use futures::stream::iter_ok;
use std::collections::hash_map::HashMap;
use std::net::SocketAddr;

pub struct ConnectionManager {
    key: Key,
    cfg: Config,
    conns: HashMap<SocketAddr, Connection>,
}

impl ConnectionManager {
    pub fn pipe<'a, S>(
        cfg: Config,
        src: S,
    ) -> Result<Box<Stream<Item = (SocketAddr, Cmd), Error = Error> + 'a>>
    where
        S: Stream<Item = (Option<SocketAddr>, Event), Error = Error> + 'a,
    {
        let mut cm = Self::new(cfg)?;
        Ok(Box::new(
            src.take_while(|&(_, ref event)| match *event {
                Event::Api(Api::Shutdown) => ok(false),
                _ => ok(true),
            }).map(|pair| pair)
                .filter_map(move |e| match e {
                    (Some(peer), event) => Some(cm.receive(peer, event)),
                    _ => None,
                })
                .map(|cmds| iter_ok(cmds.into_iter()))
                .flatten(),
        ))
    }

    fn new(cfg: Config) -> Result<Self> {
        Ok(Self { key: Key::new()?, cfg, conns: HashMap::new() })
    }

    fn receive(&mut self, peer: SocketAddr, event: Event) -> Vec<(SocketAddr, Cmd)> {
        let (conn, cmds) = self.conns
            .remove(&peer)
            .unwrap_or_else(|| Connection::new(self.key.clone(), self.cfg.clone()))
            .step(peer, event);
        if conn.should_persist() {
            self.conns.insert(peer, conn);
        }
        cmds
    }
}
