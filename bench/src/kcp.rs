use crate::{Result, *};
use bincode::*;
use futures::{channel::mpsc, prelude::*};
use serde::{Deserialize, Serialize};
use std::net::*;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

#[allow(warnings)]
mod kcp {
    include!(concat!(env!("OUT_DIR"), "/kcp.rs"));
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct Conv(u32);

struct KcpServer {
    #[allow(unused)]
    marker: Arc<()>,
    new_peers: mpsc::UnboundedReceiver<(Conv, SocketAddr)>,
}

impl KcpServer {
    pub fn bind(address: impl ToSocketAddrs) -> Result<Self> {
        let socket = UdpSocket::bind(address)?;

        let (peer_sink, new_peers) = mpsc::unbounded();
        let marker = Arc::new(());

        let daemon_marker = marker.clone();
        std::thread::spawn(move || {
            Self::daemon(daemon_marker, peer_sink, socket)
                .expect("kcp listener daemon")
        });

        Ok(Self { marker, new_peers })
    }

    fn daemon(
        marker: Arc<()>,
        mut peer_sink: mpsc::UnboundedSender<(Conv, SocketAddr)>,
        socket: UdpSocket,
    ) -> Result<()> {
        loop {
            let mut buffer = [0; 1400];
            match socket.recv_from(&mut buffer) {
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => Err(e)?,
                Ok((_, sender)) => {
                    let conv = deserialize(&buffer)?;

                    peer_sink.unbounded_send((conv, sender))?;
                }
            }
        }
    }
}

impl Stream for KcpServer {
    type Item = Result<KcpConnection>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}

struct KcpConnection {
    control_block: *mut kcp::ikcpcb,
}
