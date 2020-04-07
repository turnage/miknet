use crate::{Result, *};
use async_std::net::*;
use bincode::*;
use futures::{
    channel::mpsc,
    prelude::*,
    stream::{self, LocalBoxStream, StreamExt},
};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

#[allow(warnings)]
mod kcp {
    include!(concat!(env!("OUT_DIR"), "/kcp.rs"));
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct Conv(u32);

struct NewPeer {
    conv: Conv,
    address: SocketAddr,
}

struct KcpServer {
    new_peers: LocalBoxStream<'static, Result<NewPeer>>,
}

impl KcpServer {
    pub async fn bind(address: impl ToSocketAddrs) -> Result<Self> {
        let socket = UdpSocket::bind(address).await?;

        let (mut sink, mut stream) = mpsc::channel(0);
        let mut watcher = (move || async move {
            let mut buffer = [0; 1024];
            loop {
                let (_, sender) = socket.recv_from(&mut buffer).await?;
                let conv: Conv = deserialize(&buffer)?;
                sink.send(NewPeer {
                    conv,
                    address: sender,
                })
                .await?;
            }

            Ok(())
        })()
        .boxed_local();

        let mut terminated = false;
        let new_peers = stream::poll_fn(
            move |ctx: &mut Context| -> Poll<Option<Result<NewPeer>>> {
                if terminated {
                    return Poll::Ready(None);
                }

                if let Poll::Ready(Err(e)) = watcher.as_mut().poll(ctx) {
                    terminated = true;
                    return Poll::Ready(Some(Err(e)));
                }

                Pin::new(&mut stream)
                    .poll_next(ctx)
                    .map(|peer| Ok(peer).transpose())
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
        Poll::Pending
    }
}

struct KcpConnection {
    control_block: *mut kcp::ikcpcb,
}
