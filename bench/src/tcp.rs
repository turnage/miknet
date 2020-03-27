//! TCP implementation of the nhanh API

use anyhow::anyhow;
use crate::*;
use async_std::{
    io::ReadExt,
    net::*,
    task::{Context, Poll},
};
use bincode::{deserialize, serialize};
use futures::channel::mpsc;
use futures::{
    future::{self, FutureExt, TryFutureExt},
    sink::SinkExt,
    stream::{self, LocalBoxStream, Fuse, FusedStream, StreamExt, TryStreamExt},
    Sink, Stream,
};
use nhanh::*;
use serde::{Deserialize, Serialize};
use std::{marker::Unpin, pin::Pin};
use thiserror::Error;
use tokio_serde::{formats::*, SymmetricallyFramed};
use tokio_util::{codec::*, compat::*};

type FramedTcpStream = Framed<Compat<TcpStream>, LengthDelimitedCodec>;
type DatagramStream = SymmetricallyFramed<
    FramedTcpStream,
    Datagram,
    SymmetricalBincode<Datagram>,
>;

pub struct TcpServer {
    incoming: Fuse<Incoming<'static>>,
}

impl TcpServer {
    pub async fn bind(addrs: impl ToSocketAddrs) -> Result<TcpServer> {
        let listener = TcpListener::bind(addrs).await?;
        let listener = Box::leak(Box::new(listener));

        Ok(Self {
            incoming: listener.incoming().fuse(),
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
            Poll::Ready(Some(Ok(tcp_stream))) => Poll::Ready(Some(Ok(TcpConnection::from(tcp_stream)))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e.into()))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct TcpConnection {
    receiver: LocalBoxStream<'static, Result<Datagram>>,
    sender: Pin<Box<dyn Sink<SendCmd, Error=Box<dyn std::error::Error>> + Unpin>>,
}

impl TcpConnection {
    pub async fn connect(address: impl ToSocketAddrs) -> Result<Self> {
        let tcp_stream = TcpStream::connect(address).await?;
        Ok(TcpConnection::from(tcp_stream))
    }

    fn send_gate() -> impl FnMut(SendCmd) -> stream::Iter<<Option<Result<Datagram>> as IntoIterator>::IntoIter> {

        let mut total_sent = 0;
        move |send_cmd: SendCmd| {
                stream::iter(
                    match send_cmd.delivery_mode {
                        DeliveryMode::ReliableOrdered(stream_id) => {
                            total_sent += 1;
                            Some(Ok(Datagram {
                                data: send_cmd.data,
                                stream_position: Some(StreamPosition {
                                    stream_id, 
                                    index: StreamIndex::Ordinal(total_sent),
                                })
                            }))
                        }
                        _ => None,
                    }
                )
    }}

}

impl From<TcpStream> for TcpConnection {
    fn from(stream: TcpStream) -> Self {
        let framer = LengthDelimitedCodec::new();
        let stream = Framed::new(stream.compat(), framer);
        let codec = SymmetricalBincode::default();

        let wire = SymmetricallyFramed::new(stream, codec);
        let wire = wire.sink_map_err(Into::into);
        let wire = wire.map_err(Into::into);
        let (wire_sink, wire_stream) = wire.split();

        let wire_sink = wire_sink.with_flat_map(Box::new(Self::send_gate()));

        Self {
            receiver: wire_stream.boxed_local(),
            sender: Pin::new(Box::new(wire_sink)),
        }
    }
}



impl Connection for TcpConnection {}

impl Sink<SendCmd> for TcpConnection {
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

impl Stream for TcpConnection {
    type Item = Result<Datagram>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.receiver).poll_next(ctx)
    }
}

impl FusedStream for TcpConnection {
    fn is_terminated(&self) -> bool {
        false
    }
}
