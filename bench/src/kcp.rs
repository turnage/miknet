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

use crate::{Result, *};
use async_std::net::*;
use bincode::*;
use futures::{
    channel::mpsc,
    future::{Either, LocalBoxFuture},
    prelude::*,
    stream::{self, LocalBoxStream, StreamExt},
};
use serde::{Deserialize, Serialize};
use std::ffi::{c_void};
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::pin::Pin;
use std::sync::Arc;
use std::os::raw::c_int;
use std::task::{Context, Poll};
use std::time::*;

#[allow(warnings)]
mod kcp {
    include!(concat!(env!("OUT_DIR"), "/kcp.rs"));
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct Conv(u32);

struct NewPeer {
    conv: Conv,
    socket: UdpSocket,
    address: SocketAddr,
}

struct KcpServer {
    new_peers: LocalBoxStream<'static, Result<NewPeer>>,
}

impl KcpServer {
    pub async fn bind(address: impl ToSocketAddrs) -> Result<Self> {
        let socket = UdpSocket::bind(address).await?;

        Self::from_socket(socket)
    }

    fn from_socket(socket: UdpSocket) -> Result<Self> {
        let mut watcher = (move || async move {
            let mut buffer = [0; 1024];
            let (_, sender) = socket.recv_from(&mut buffer).await?;
            let conv: Conv = deserialize(&buffer)?;
            Ok(NewPeer {
                conv,
                socket,
                address: sender,
            })
        })()
        .boxed_local();

        let mut terminated = false;
        let new_peers = stream::poll_fn(
            move |ctx: &mut Context| -> Poll<Option<Result<NewPeer>>> {
                if terminated {
                    return Poll::Ready(None);
                }

                if let Poll::Ready(r) = watcher.as_mut().poll(ctx) {
                    terminated = true;
                    return Poll::Ready(Some(r));
                }

                Poll::Pending
            },
        );

        Ok(Self {
            new_peers: new_peers.boxed_local(),
        })
    }
}

impl Stream for KcpServer {
    type Item = Result<KcpConnection>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Option<Self::Item>> {
        let peer = match self.new_peers.as_mut().poll_next(ctx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(peer) => peer,
        };

        Poll::Pending
    }
}

struct KcpConnection {
    driver: LocalBoxFuture<'static, Result<()>>,
    control_block: *mut kcp::ikcpcb,
}

impl KcpConnection {
    async fn driver(
        Conv(conv): Conv,
        socket: UdpSocket,
        server: SocketAddr,
    ) -> Result<()> {
        socket.connect(server).await?;

        let epoch = Instant::now();
        let current_time_ms =
            || Instant::now().duration_since(epoch).as_millis() as u32;


        let (mut service_notify, service_requests) = mpsc::channel(1);
        let (mut wire_notify, wire_events) = mpsc::channel(1);

        enum Event {
            ServiceRequest,
        }

        let mut events = stream::select(service_requests, wire_events);

        let mut buffer = [0u8; 65535];

        let output_callback = |buf: *const i8, len: i32, _cb: *mut kcp::ikcpcb| -> c_int {
            let data: &[u8] = unsafe {std::slice::from_raw_parts(buf as *const u8, len as usize) };
            async_std::task::block_on(socket.send(data)).expect("sending data for kcp");
            0
        };
        let (callback, state) = unsafe { wrap_output_callback(&output_callback) };
        let mut cb = unsafe { kcp::ikcp_create(conv, state) };
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
                    }
                },
            }
        }
    }
}

type OutputCallback = unsafe extern "C" fn (*const i8, c_int, *mut kcp::ikcpcb, *mut c_void) -> c_int;

unsafe fn wrap_output_callback<F>(f: &F) -> (OutputCallback, *mut c_void) 
where
    F: Fn(*const i8, c_int, *mut kcp::ikcpcb) -> c_int
{
    unsafe extern "C" fn trampoline<F>(buf: *const i8, len: c_int, cb: *mut kcp::ikcpcb, callback: *mut c_void) -> c_int where F: Fn(*const i8, c_int, *mut kcp::ikcpcb) -> c_int
    {
        let callback: &F = &(*(callback as *const F));
        callback(buf, len, cb)
    }

    (trampoline::<F>, f as *const F as *mut c_void)
}
