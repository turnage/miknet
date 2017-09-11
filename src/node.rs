//! A user handle to their own node in a miknet connection(s).

use {Error, MEvent, Result};
use cmd::Cmd;
use conn::ConnectionManager;
use event::{Api, Event};
use futures::{Future, Sink, Stream};
use futures::sync::mpsc::{UnboundedSender, unbounded};
use gram::GramCodec;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket as StdUdpSocket};
use std::thread::spawn;
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

/// Node is miknet's abstaction over a socket. This represents a single node which may be connected
/// to and communicate with other miknet Nodes.
pub struct Node {
    pub addr: SocketAddr,
    api_sink: UnboundedSender<(Option<SocketAddr>, Event)>,
}

impl Node {
    pub fn new<T: ToSocketAddrs>(addrs: T) -> Result<(Self, Box<Iterator<Item = MEvent>>)> {
        let (user_event_sink, user_event_stream) = unbounded();
        let (api_sink, api_stream) = unbounded();
        let socket = StdUdpSocket::bind(addrs)?;
        let addr = socket.local_addr()?;

        spawn(
            move || match Self::run(socket, api_stream.map_err(Error::from), user_event_sink.clone()) {
                Ok(()) => {}
                Err(e) => {
                    user_event_sink
                        .send(MEvent::Error(format!("{}", e)))
                        .wait()
                        .expect(&format!("Could not report error to user: {:?}", e));
                }
            },
        );

        Ok((
            Self { addr, api_sink },
            Box::new(user_event_stream.wait().filter_map(|item| match item {
                Ok(event) => Some(event),
                Err(_) => Some(MEvent::Shutdown),
            })),
        ))
    }

    pub fn connect(&self, addr: &SocketAddr) -> Result<()> {
        self.queue(Some(*addr), Event::Api(Api::Conn))
    }

    pub fn disconnect(&self, addr: &SocketAddr) -> Result<()> {
        self.queue(Some(*addr), Event::Api(Api::Disc))
    }

    pub fn shutdown(&self) -> Result<()> { self.queue(None, Event::Api(Api::Shutdown)) }

    fn queue(&self, addr: Option<SocketAddr>, event: Event) -> Result<()> {
        self.api_sink.clone().send((addr, event)).wait()?;
        Ok(())
    }

    fn run<AS, US>(socket: StdUdpSocket, api_stream: AS, user_event_sink: US) -> Result<()>
    where
        AS: Stream<Item = (Option<SocketAddr>, Event), Error = Error>,
        US: Sink<SinkItem = MEvent> + Clone + 'static,
    {
        let mut core = Core::new()?;
        let handle = core.handle();

        let net_error_reporter = user_event_sink.clone();
        let (net_sink, net_stream) = UdpSocket::from_socket(socket, &handle)?
            .framed(GramCodec {})
            .split();
        let (net_cmd_sink, net_cmd_stream) = unbounded();
        handle.spawn(
            net_sink
                .send_all(net_cmd_stream.map_err(|_| {
                    io::Error::new(io::ErrorKind::WriteZero, "Sender is corrupt.")
                }))
                .map(|_| ())
                .or_else(move |_| {
                    net_error_reporter
                        .clone()
                        .send(MEvent::Error("".to_string()))
                        .then(|_| Ok(()))
                }),
        );


        let sources = net_stream
            .filter_map(|e| e)
            .map(|(addr, event)| (Some(addr), event))
            .map_err(Error::from)
            .select(api_stream);
        let stream = ConnectionManager::new(sources)?.for_each(
            move |(peer, cmd)| {
                match (peer, cmd) {
                    (peer, Cmd::Net(bytes)) => {
                        handle.spawn(net_cmd_sink.clone().send((peer, bytes)).then(|_| Ok(())));
                    }
                    (_, Cmd::User(event)) => {
                        handle.spawn(user_event_sink.clone().send(event).then(|_| Ok(())));
                    }
                    _ => (),
                };
                Ok(())
            },
        );

        core.run(stream)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    use test_util::simulate;

    #[test]
    fn api_stream_works() {
        let (tx, rx) = unbounded();
        let local_addr = SocketAddr::from_str("127.0.0.1:1").expect("any addr");
        let node = Node { addr: local_addr, api_sink: tx };
        let addr = SocketAddr::from_str("127.0.0.1:0").expect("loopback addr");
        node.connect(&addr);

        if let Ok((Some((Some(dest_addr), event)), _)) = rx.into_future().wait() {
            assert_eq!(dest_addr, addr);
            assert_eq!(event, Event::Api(Api::Conn));
        } else {
            panic!("no api event!");
        }
    }

    #[test]
    fn connection() {
        simulate(
            |n1, n2| n1.connect(&n2.addr),
            &|event| match *event {
                MEvent::ConnectionEstablished(_) => true,
                _ => false,
            },
            |n1addr, n2addr| {
                vec![
                    MEvent::ConnectionEstablished(n2addr),
                    MEvent::ConnectionEstablished(n1addr),
                ]
            },
        );
    }
}
