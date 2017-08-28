use {Error, MEvent, Result};
use cmd::Cmd;
use conn::ConnectionManager;
use event::Event;
use futures::{Future, Sink, Stream, stream};
use futures::sync::mpsc::{UnboundedReceiver, unbounded};
use gram::GramCodec;
use host::Host;
use std::io::{self, ErrorKind};
use std::net::SocketAddr;
use std::thread::spawn;
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

pub struct Node {
    host: Host,
    user_event_stream: UnboundedReceiver<MEvent>,
}

impl Node {
    pub fn new(addr: &SocketAddr) -> Self {
        let (user_event_sink, user_event_stream) = unbounded();
        let (api_sink, api_stream) = unbounded();
        let addr = *addr;

        spawn(
            move || match Node::run(addr, api_stream.map_err(Error::from), user_event_sink.clone()) {
                Ok(()) => {}
                Err(e) => {
                    user_event_sink.send(MEvent::Error(format!("{}", e))).wait();
                }
            },
        );

        Node { host: Host::new(api_sink), user_event_stream }
    }

    fn run<AS, US>(addr: SocketAddr, api_stream: AS, user_event_sink: US) -> Result<()>
    where
        AS: Stream<Item = (SocketAddr, Event), Error = Error>,
        US: Sink<SinkItem = MEvent> + Clone + 'static,
    {
        let mut core = Core::new()?;
        let handle = core.handle();

        let net_error_reporter = user_event_sink.clone();
        let (net_sink, net_stream) = UdpSocket::bind(&addr, &handle)?
            .framed(GramCodec {})
            .split();
        let (net_cmd_sink, net_cmd_stream) = unbounded();
        handle.spawn(
            net_sink
                .send_all(net_cmd_stream.map_err(|_| {
                    io::Error::new(io::ErrorKind::WriteZero, "Sender is corrupt.")
                }))
                .map(|_| ())
                .or_else(move |e| {
                    net_error_reporter
                        .clone()
                        .send(MEvent::Error("".to_string()))
                        .then(|_| Ok(()))
                }), /*                .map(|_| ())
                .map_err(|_| ()),*/
        );


        let mut cm = ConnectionManager::new()?;
        let stream = net_stream
            .filter_map(|e| e)
            .map_err(Error::from)
            .select(api_stream)
            .map(|(sender, event)| {
                stream::iter_ok::<_, Error>(cm.receive(sender, event).into_iter())
            })
            .flatten()
            .for_each(move |(peer, cmd): (SocketAddr, Cmd)| {
                match cmd {
                    Cmd::Net(bytes) => {
                        handle.spawn(
                            net_cmd_sink.clone().send((peer, bytes)).then(|_| Ok(())),
                        )
                    }
                    Cmd::User(event) => {
                        handle.spawn(user_event_sink.clone().send(event).then(|_| Ok(())))
                    }
                    _ => (),
                };
                Ok(())
            });

        core.run(stream)
    }
}
