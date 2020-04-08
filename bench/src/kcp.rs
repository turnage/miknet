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
    future::{AbortHandle, Abortable, Either, LocalBoxFuture},
    prelude::*,
    sink,
    stream::{self, Fuse, FusedStream, LocalBoxStream, StreamExt},
};
use rand::random;
use serde::{Deserialize, Serialize};
use std::ffi::c_void;
use std::os::raw::c_int;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::pin::Pin;
use std::sync::Arc;
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
    pub async fn bind(address: impl ToSocketAddrs) -> Result<Self> {
        let tcp = tcp::TcpServer::bind(address).await?;

        let peers = tcp.then(|tcp_connection| async {
            let mut tcp_connection = tcp_connection?;

            let udp =
                UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
                    .await?;
            let port = udp.local_addr()?.port();

            tcp_connection
                .send(SendCmd {
                    data: serialize(&port)?,
                    delivery_mode: DeliveryMode::ReliableOrdered(StreamId(0)),
                    ..SendCmd::default()
                })
                .await?;

            let client_port =
                tcp_connection.next().await.expect("Confirmation of port")?;
            let client_port: u16 = deserialize(client_port.data.as_slice())?;

            let mut client_addr = tcp_connection.peer_addr()?;
            client_addr.set_port(client_port);

            Ok(KcpConnection::from_socket(tcp_connection, udp, client_addr)
                .await)
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
    tcp_connection: tcp::TcpConnection,
    driver: LocalBoxFuture<'static, Result<()>>,
    control_block: *mut kcp::ikcpcb,
    receiver: LocalBoxStream<'static, Result<Datagram>>,
    sender:
        Pin<Box<dyn Sink<SendCmd, Error = Box<dyn std::error::Error>> + Unpin>>,
}

impl KcpConnection {
    pub async fn connect(mut server: SocketAddr) -> Result<Self> {
        let mut tcp_connection = tcp::TcpConnection::connect(server).await?;

        let port =
            tcp_connection.next().await.expect("udp port from server")?;
        let port: u16 = deserialize(port.data.as_slice())?;
        let udp_addr = server.set_port(port);

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
        let mut control_block = 0 as *mut kcp::ikcpcb;
        let control_block_out = &mut control_block as *mut *mut kcp::ikcpcb;
        let driver = Self::driver(socket, peer, control_block_out);
        let (cancel_handle, registration) = AbortHandle::new_pair();
        let driver = Abortable::new(driver, registration)
            .err_into()
            .map(|r| r.and_then(|r| Ok(r?)));

        Self {
            tcp_connection,
            driver: driver.boxed_local(),
            control_block,
            receiver: stream::empty().boxed_local(),
            sender: Pin::new(Box::new(sink::drain().sink_err_into())),
        }
    }

    async fn driver(
        socket: UdpSocket,
        peer: SocketAddr,
        control_block_out: *mut *mut kcp::ikcpcb,
    ) -> Result<()> {
        socket.connect(peer).await?;

        let epoch = Instant::now();
        let current_time_ms =
            || Instant::now().duration_since(epoch).as_millis() as u32;

        let (mut service_notify, service_requests) = mpsc::channel(1);
        let (mut wire_notify, wire_events) = mpsc::channel(1);

        enum Event {
            ServiceRequest,
            Outgoing(SendCmd),
        }

        let mut events = stream::select(service_requests, wire_events);

        let mut buffer = [0u8; 65535];

        let output_callback =
            |buf: *const i8, len: i32, _cb: *mut kcp::ikcpcb| -> c_int {
                let data: &[u8] = unsafe {
                    std::slice::from_raw_parts(buf as *const u8, len as usize)
                };
                async_std::task::block_on(socket.send(data))
                    .expect("sending data for kcp");
                0
            };
        let (callback, state) =
            unsafe { wrap_output_callback(&output_callback) };
        let mut cb = unsafe {
            kcp::ikcp_create(/*conv=*/ 0, state)
        };
        unsafe { *control_block_out = cb };
        unsafe { kcp::ikcp_setoutput(cb, Some(callback)) }

        loop {
            futures::select! {
                read_result = socket.recv(&mut buffer).fuse() => {
                    let len = read_result?;
                    let data = buffer.as_ptr() as *const i8;
                    let code = unsafe { kcp::ikcp_input(cb, data, len as i64) };
                    if code < 0 {
                        panic!("kcp panic; input error: {:?}", code);
                    }
                }
                event = events.select_next_some()  => {
                    match event {
                        Event::ServiceRequest => unsafe {
                            kcp::ikcp_update(cb, current_time_ms());

                            // Schedule next update
                            let delay = kcp::ikcp_check(cb, current_time_ms());
                            let delay = Duration::from_millis(delay as u64);
                            let timer = futures_timer::Delay::new(delay);

                            let mut notifier = service_notify.clone();
                            let notify = |_| async move {
                                notifier.send(Event::ServiceRequest).await
                            };
                            async_std::task::spawn(timer.then(notify));
                        }
                        Event::Outgoing(send_cmd) => unsafe {
                            match send_cmd.delivery_mode {
                                DeliveryMode::ReliableOrdered(StreamId(0)) => {},
                                _ => panic!("KCP only supports a single reliable channel"),
                            };

                            let data = send_cmd.data.as_ptr() as *const i8;
                            let code = kcp::ikcp_input(cb, data, send_cmd.data.len() as i64);
                            if code < 0 {
                                panic!("kcp input failed: {:?}", code);
                            }
                        }
                    }
                },
            }
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
        Pin::new(&mut self.receiver).poll_next(ctx)
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
