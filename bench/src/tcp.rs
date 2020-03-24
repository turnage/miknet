//! TCP implementation of the nhanh API

use crate::*;
use async_std::{
    io::ReadExt,
    net::*,
    task::{Context, Poll},
};
use bincode::{deserialize, serialize};
use futures::channel::mpsc;
use futures::{
    future::{self, FutureExt, LocalBoxFuture, TryFutureExt},
    sink::SinkExt,
    stream::{select, Fuse, FusedStream, StreamExt, TryStreamExt},
    Sink, Stream,
};
use nhanh::*;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error;
use tokio_serde::{formats::*, SymmetricallyFramed};
use tokio_util::{codec::*, compat::*};

type FramedTcpStream = Framed<Compat<TcpStream>, LengthDelimitedCodec>;
type DatagramStream = SymmetricallyFramed<
    FramedTcpStream,
    Datagram,
    SymmetricalBincode<Datagram>,
>;

type Task = LocalBoxFuture<'static, Result<()>>;

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
    task: Task,
    receiver: mpsc::Receiver<Result<Datagram>>,
    sender: mpsc::Sender<SendCmd>,
}

impl TcpConnection {
    pub async fn connect(address: impl ToSocketAddrs) -> Result<Self> {
        let tcp_stream = TcpStream::connect(address).await?;
        Ok(TcpConnection::from(tcp_stream))
    }

    async fn new_task(
        wire: DatagramStream,
        mut user_sink: mpsc::Sender<Result<Datagram>>,
        user_stream: mpsc::Receiver<SendCmd>,
    ) -> Result<()> {
        let box_error =
            |e: std::io::Error| -> Box<dyn std::error::Error> { Box::new(e) };

        let (mut wire_sink, wire_stream) =
            wire.sink_map_err(&box_error).map_err(&box_error).split();

        enum Input {
            Wire(Result<Datagram>),
            User(SendCmd),
        }

        let mut wire_stream = wire_stream.map(Input::Wire);
        let mut user_stream = user_stream.map(Input::User);
        let mut stream = select(wire_stream, user_stream);

        let mut total_sent = 0;

        while let Some(input) = stream.next().await {
            match input {
                Input::Wire(result) => {
                    user_sink.send(Ok(result?)).await;
                }
                Input::User(SendCmd {
                    data,
                    delivery_mode,
                    ..
                }) => match delivery_mode {
                    DeliveryMode::ReliableOrdered(stream_id) => {
                        let datagram = Datagram {
                            data,
                            stream_position: Some(StreamPosition {
                                stream_id,
                                index: StreamIndex::Ordinal(total_sent),
                            }),
                        };

                        wire_sink.send(datagram).await;
                        total_sent += 1;
                    }
                    unsupported => {
                        eprintln!("Dropping payload with send mode {:?}; TCP benchmark impl does not support it.", unsupported);
                    }
                },
            }
        }

        Ok(())
    }
}

impl From<TcpStream> for TcpConnection {
    fn from(stream: TcpStream) -> Self {
        let framer = LengthDelimitedCodec::new();
        let stream = Framed::new(stream.compat(), framer);
        let codec = SymmetricalBincode::default();
        let wire = SymmetricallyFramed::new(stream, codec);

        let (user_sink, receiver) = mpsc::channel(0);
        let (sender, user_stream) = mpsc::channel(0);

        let task = Self::new_task(wire, user_sink, user_stream).boxed_local();

        Self {
            task,
            receiver,
            sender,
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
        if let Poll::Ready(result) = self.task.as_mut().poll(ctx) {
            return Poll::Ready(None);
        }

        Pin::new(&mut self.receiver).poll_next(ctx)
    }
}

impl FusedStream for TcpConnection {
    fn is_terminated(&self) -> bool {
        false
    }
}
