//! socket provides abstractions over UDP sockets.

use {Error, Result};
use event::Event;
use futures::{Future, Sink, Stream};
use futures::sync::mpsc::{UnboundedSender, unbounded};
use gram::GramCodec;
use std::io;
use std::net::{SocketAddr, UdpSocket as StdUdpSocket};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Handle;

/// Socket is a abstraction over a UDP socket providing a protocol where we can write in a Gram
/// struct and read out entirely deserialized Gram structs.
pub struct Socket;

impl Socket {
    /// pipe returns a sink to send network commands to and a stream over which to receive network
    /// events.
    pub fn pipe<'a>(
        socket: StdUdpSocket,
        handle: &'a Handle,
    ) -> Result<
        (UnboundedSender<(SocketAddr, Vec<u8>)>,
         Box<Stream<Item = (SocketAddr, Event), Error = Error> + 'a>),
    > {
        let (net_sink, net_stream) = UdpSocket::from_socket(socket, &handle)?
            .framed(GramCodec {})
            .split();
        let (net_cmd_sink, net_cmd_stream) = unbounded();
        handle.spawn(
            net_sink
                .sink_map_err(|_| ())
                .send_all(net_cmd_stream)
                .then(|_| Ok(())),
        );
        Ok((net_cmd_sink, Box::new(net_stream.filter_map(|e| e).map_err(Error::from))))
    }
}
