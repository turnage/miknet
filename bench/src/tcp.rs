//! TCP implementation of the nhanh API

use crate::*;
use async_std::{
    io::ReadExt,
    net::*,
    task::{Context, Poll}
};
use futures::{
    future::{FutureExt, TryFutureExt, LocalBoxFuture},
    stream::{FusedStream, StreamExt, TryStreamExt, Fuse},
    Stream,
};
use nhanh::*;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error;
use bincode::{deserialize, serialize};
use tokio_util::{compat::*, codec::*};
use tokio_serde::{SymmetricallyFramed, formats::*};

struct TcpServer {
    incoming: Fuse<Incoming<'static>>
}

impl TcpServer {
    pub async fn new(addrs: impl ToSocketAddrs) -> Result<TcpServer> {
        let listener = TcpListener::bind(addrs).await?;
        let listener = Box::leak(Box::new(listener));

        Ok(Self {
            incoming: listener.incoming().fuse()
        })
    }
}

impl FusedStream for TcpServer {
    fn is_terminated(&self) -> bool {
        self.incoming.is_terminated()
    }
}

impl Server<TcpConnection> for TcpServer {}

impl Stream for TcpServer {
    type Item = Result<TcpConnection>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.incoming).poll_next(ctx) {
            Poll::Ready(Some(Ok(tcp_stream))) => {
                Poll::Ready(None)
            }
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Some(Err(e.into())))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending
        }
    }
}

pub struct TcpConnection {
    stream: SymmetricallyFramed<Framed<Compat<TcpStream>, LengthDelimitedCodec>, Datagram, SymmetricalBincode<Datagram>>,
}

impl From<TcpStream> for TcpConnection {
    fn from(stream: TcpStream) -> Self {
        let framer = LengthDelimitedCodec::new();
        let stream = Framed::new(stream.compat(), framer);
        let codec = SymmetricalBincode::default();
        let stream = SymmetricallyFramed::new(stream, codec);

        Self {
            stream
        }
    }
}

impl Connection for TcpConnection {
    fn send(&mut self, data: &dyn std::io::Read, delivery_mode: DeliveryMode) {
        //
    }
}

impl Stream for TcpConnection {
    type Item = Result<Datagram>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}

impl FusedStream for TcpConnection {
    fn is_terminated(&self) -> bool {
        false
    }
}
