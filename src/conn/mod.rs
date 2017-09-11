//! Connections.

mod miklow;

use self::miklow::{Connection, Key};

pub use self::miklow::StateCookie;

use {Error, Result};
use cmd::Cmd;
use event::{Api, Event};
use futures::{Async, Poll, Stream};
use futures::stream::Select;
use futures::unsync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use itertools::Itertools;
use std::collections::hash_map::HashMap;
use std::net::SocketAddr;

pub struct ConnectionManager<'a> {
    source: Box<Stream<Item = (Option<SocketAddr>, Event), Error = Error> + 'a>,
    key: Key,
    connections: HashMap<SocketAddr, Connection>,
    outbuf: Vec<(SocketAddr, Cmd)>,
    requeue: UnboundedSender<(Option<SocketAddr>, Event)>,
}

impl<'a> ConnectionManager<'a> {
    pub fn new<P>(source: P) -> Result<Self>
    where
        P: Stream<Item = (Option<SocketAddr>, Event), Error = Error> + 'a,
    {
        let (requeue, requeue_stream) = unbounded();
        Ok(Self {
            source: Box::new(source.select(requeue_stream.map_err(Error::from))),
            key: Key::new()?,
            connections: HashMap::new(),
            outbuf: Vec::new(),
            requeue,
        })
    }

    fn receive(&mut self, peer: SocketAddr, event: Event) -> Result<Vec<(SocketAddr, Cmd)>> {
        let conn = if let Some(conn) = self.connections.remove(&peer) {
            conn
        } else {
            Connection::new(self.key.clone())
        };
        let (conn, cmds) = if conn.ready_for(&event) {
            conn.gen_cmds(peer, event)
        } else {
            self.requeue.unbounded_send((Some(peer), event))?;
            (conn, vec![])
        };
        if conn.should_persist() {
            self.connections.insert(peer, conn);
        }
        Ok(cmds)
    }
}

impl<'a> Stream for ConnectionManager<'a> {
    type Item = (SocketAddr, Cmd);
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Some(_) = self.outbuf.first() {
            Ok(Async::Ready(Some(self.outbuf.remove(0))))
        } else {
            match self.source.poll() {
                Ok(Async::Ready(Some((Some(addr), event)))) => {
                    let mut commands = self.receive(addr, event)?;
                    self.outbuf.append(&mut commands);
                    self.poll()
                }
                Ok(Async::Ready(Some((None, event)))) => {
                    match event {
                        Event::Api(Api::Shutdown) => Ok(Async::Ready(None)),
                        _ => Err(
                            format!("Unknown event without address in pipline: {:?}", event)
                                .into(),
                        ),
                    }
                }

                Ok(Async::Ready(None)) => Ok(Async::Ready(None)),
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(e) => Err(e),
            }
        }
    }
}
