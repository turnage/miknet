//! KCP adapter for benchmarking.
//!
//! KCP is just a protocol layer, and does not establish connections. Connection
//! establishment is not benchmarked, so this adapter rolls its own.
//!
//! A UDP socket for a server listens for messages with a new connection id,
//! what KCP calls a "conv". The server stream of connections then ends (only
//! one connection per server; that's all the benchmark needs).
//!
//! The server's socket sends the conv back to the client to establish
//! connection.

use crate::{tcp, Result, *};
use async_std::net::*;
use bincode::*;
use futures::{
    channel::mpsc,
    prelude::*,
    stream::{Fuse, FusedStream, LocalBoxStream, StreamExt},
};

use std::ffi::c_void;
use std::os::raw::c_int;

use std::pin::Pin;

use std::task::{Context, Poll};
use std::time::*;

#[allow(warnings)]
mod kcp {
    include!(concat!(env!("OUT_DIR"), "/kcp.rs"));
}

pub struct KcpServer {
    peers: Fuse<LocalBoxStream<'static, Result<KcpConnection>>>,
}

impl Server<KcpConnection> for KcpServer {}

impl KcpServer {
    pub async fn bind(
        address: impl ToSocketAddrs + Clone + 'static,
    ) -> Result<Self> {
        let tcp = tcp::TcpServer::bind(address.clone()).await?;

        let peers = tcp.then(move |tcp_connection| {
            let address = address.clone();
            async move {
                let mut tcp_connection = tcp_connection?;

                let udp = UdpSocket::bind(address.clone()).await?;
                let port = udp.local_addr()?.port();

                tcp_connection
                    .send(SendCmd {
                        data: serialize(&port)?,
                        delivery_mode: DeliveryMode::ReliableOrdered(StreamId(
                            0,
                        )),
                        ..SendCmd::default()
                    })
                    .await?;

                let client_port = tcp_connection
                    .next()
                    .await
                    .expect("Confirmation of port")?;
                let client_port: u16 =
                    deserialize(client_port.data.as_slice())?;

                let mut client_addr = tcp_connection.peer_addr();
                client_addr.set_port(client_port);

                Ok(KcpConnection::from_socket(tcp_connection, udp, client_addr)
                    .await)
            }
        });

        Ok(Self {
            peers: peers.boxed_local().fuse(),
        })
    }
}

impl Stream for KcpServer {
    type Item = Result<KcpConnection>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.peers).poll_next(ctx)
    }
}

impl FusedStream for KcpServer {
    fn is_terminated(&self) -> bool {
        self.peers.is_terminated()
    }
}

pub struct KcpConnection {
    #[allow(unused)]
    tcp_connection: tcp::TcpConnection,
    receiver: mpsc::Receiver<Datagram>,
    sender:
        Pin<Box<dyn Sink<SendCmd, Error = Box<dyn std::error::Error>> + Unpin>>,
}

impl KcpConnection {
    pub async fn connect(mut server: SocketAddr) -> Result<Self> {
        let mut tcp_connection = tcp::TcpConnection::connect(server).await?;

        let port =
            tcp_connection.next().await.expect("udp port from server")?;
        let port: u16 = deserialize(port.data.as_slice())?;
        let _udp_addr = server.set_port(port);

        let udp = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
            .await?;
        let our_port = udp.local_addr()?.port();
        tcp_connection
            .send(SendCmd {
                data: serialize(&our_port)?,
                delivery_mode: DeliveryMode::ReliableOrdered(StreamId(0)),
                ..SendCmd::default()
            })
            .await?;

        Ok(Self::from_socket(tcp_connection, udp, server).await)
    }

    async fn from_socket(
        tcp_connection: tcp::TcpConnection,
        socket: UdpSocket,
        peer: SocketAddr,
    ) -> Self {
        let (command_sink, command_stream) = mpsc::channel(100);
        let (datagram_sink, datagram_stream) = mpsc::channel(100);

        async_std::task::spawn(
            Self::driver(socket, peer, command_stream, datagram_sink).map(drop),
        );

        Self {
            tcp_connection,
            receiver: datagram_stream,
            sender: Pin::new(Box::new(command_sink.sink_err_into())),
        }
    }

    async fn driver(
        socket: UdpSocket,
        peer: SocketAddr,
        command_stream: mpsc::Receiver<SendCmd>,
        datagram_sink: mpsc::Sender<Datagram>,
    ) -> Result<()> {
        socket.connect(peer).await?;

        enum Event {
            Outgoing(SendCmd),
        }

        let mut events = command_stream.map(Event::Outgoing);

        let epoch = Instant::now();
        let current_time_ms =
            || Instant::now().duration_since(epoch).as_millis() as u32;

        let cb = {
            let output_callback = |buf: *const i8,
                                   len: i32,
                                   _cb: *mut kcp::ikcpcb|
             -> c_int {
                let data: &[u8] = unsafe {
                    std::slice::from_raw_parts(buf as *const u8, len as usize)
                };
                eprintln!("output callback at {}", current_time_ms());
                let _ = async_std::task::block_on(socket.send(data));
                0
            };
            let (callback, state) =
                unsafe { wrap_output_callback(&output_callback) };
            let cb = unsafe {
                kcp::ikcp_create(/*conv=*/ 0, state)
            };
            unsafe { kcp::ikcp_setoutput(cb, Some(callback)) }
            Cb(cb)
        };

        let _service_ticks = ticker(1000).fuse();

        let mut servicer = KcpServicer {
            epoch: Instant::now(),
            datagram_sink,
            cb,
            sequence_number: 0,
        };

        let mut buffer = [0u8; 65535];
        let mut service_ticker = ticker(1000).fuse();
        loop {
            futures::select! {
                read_result = socket.recv(&mut buffer).fuse() => {
                    {
                        let len = read_result?;
                        let data = buffer.as_ptr() as *const i8;
                        let code = unsafe {
                            kcp::ikcp_input(cb.0, data, len as i64)
                        };
                        if code < 0 {
                            panic!("kcp panic; input error: {:?}", code);
                        }
                    }

                    servicer.service().await;
                }
                event = events.select_next_some()  => {
                    match event {
                        Event::Outgoing(send_cmd) => unsafe {
                            match send_cmd.delivery_mode {
                                DeliveryMode::ReliableOrdered(StreamId(0)) => {},
                                _ => panic!("KCP only supports a single reliable channel"),
                            };

                            {
                                let data = send_cmd.data.as_ptr() as *const i8;
                                let code = kcp::ikcp_send(
                                   cb.0,
                                   data,
                                   send_cmd.data.len() as i32
                                );
                                if code < 0 {
                                    panic!("kcp input failed: {:?}", code);
                                }
                            }

                            servicer.service().await;

                            kcp::ikcp_flush(cb.0);
                        }
                    }
                }
                _ = service_ticker.select_next_some() => servicer.service().await,
            }
        }
    }
}

#[derive(Copy, Clone)]
struct Cb(*mut kcp::ikcpcb);

unsafe impl Send for Cb {}

struct KcpServicer {
    epoch: Instant,
    datagram_sink: mpsc::Sender<Datagram>,
    cb: Cb,
    sequence_number: u32,
}

impl KcpServicer {
    fn current_time_ms(&self) -> u32 {
        Instant::now().duration_since(self.epoch).as_millis() as u32
    }

    async fn service(&mut self) {
        unsafe { kcp::ikcp_update(self.cb.0, self.current_time_ms()) };

        let mut buffer = [0; 65535];
        #[allow(unused_assignments)]
        let mut len = i32::default();

        while {
            let buffer_ptr = buffer.as_mut_ptr() as *mut i8;
            len = unsafe {
                kcp::ikcp_recv(self.cb.0, buffer_ptr, buffer.len() as i32)
            };
            len > 0
        } {
            self.datagram_sink
                .send(Datagram {
                    data: buffer[0..(len as usize)].to_vec(),
                    stream_position: Some(StreamPosition {
                        stream_id: StreamId(0),
                        index: StreamIndex::Ordinal(self.sequence_number),
                    }),
                })
                .await
                .expect("Sending datagram to user");
            self.sequence_number += 1;
        }
    }
}

impl Connection for KcpConnection {}

impl Sink<SendCmd> for KcpConnection {
    type Error = Box<dyn std::error::Error>;
    fn poll_ready(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.sender)
            .poll_ready(ctx)
            .map_err(Into::into)
    }
    fn start_send(mut self: Pin<&mut Self>, item: SendCmd) -> Result<()> {
        Pin::new(&mut self.sender)
            .start_send(item)
            .map_err(Into::into)
    }
    fn poll_flush(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.sender)
            .poll_flush(ctx)
            .map_err(Into::into)
    }
    fn poll_close(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.sender)
            .poll_close(ctx)
            .map_err(Into::into)
    }
}

impl Stream for KcpConnection {
    type Item = Result<Datagram>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.receiver)
            .poll_next(ctx)
            .map(|d| d.map(Ok))
    }
}

impl FusedStream for KcpConnection {
    fn is_terminated(&self) -> bool {
        false
    }
}

type OutputCallback = unsafe extern "C" fn(
    *const i8,
    c_int,
    *mut kcp::ikcpcb,
    *mut c_void,
) -> c_int;

unsafe fn wrap_output_callback<F>(f: &F) -> (OutputCallback, *mut c_void)
where
    F: Fn(*const i8, c_int, *mut kcp::ikcpcb) -> c_int,
{
    unsafe extern "C" fn trampoline<F>(
        buf: *const i8,
        len: c_int,
        cb: *mut kcp::ikcpcb,
        callback: *mut c_void,
    ) -> c_int
    where
        F: Fn(*const i8, c_int, *mut kcp::ikcpcb) -> c_int,
    {
        let callback: &F = &(*(callback as *const F));
        callback(buf, len, cb)
    }

    (trampoline::<F>, f as *const F as *mut c_void)
}
