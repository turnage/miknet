//! TCP implementation of the nhanh API

use crate::*;
use async_std::{
    io::ReadExt,
    net::*,
    task::{Context, Poll},
};
use futures::{
    future::{FutureExt, TryFutureExt},
    stream::{FusedStream, StreamExt, TryStreamExt},
    Stream,
};
use nhanh::*;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use thiserror::Error;

rental! {
    pub mod rentals {
        use futures::{future::LocalBoxFuture, stream::{LocalBoxStream}};

        #[rental_mut]
        pub struct Incoming {
            listener: Box<super::TcpListener>,
            incoming: LocalBoxStream<'listener, super::Result<super::TcpConnection>>
        }

        #[rental]
        pub struct StreamRead {
            stream: Box<super::TcpStream>,
            read: Option<LocalBoxFuture<'stream, super::Result<Vec<u8>>>>
        }
    }
}

#[derive(Debug, Error)]
pub enum TcpError {
    #[error("Rental construction: {:?}", .0)]
    Rental(()),
}

struct TcpServer {
    incoming: rentals::Incoming,
}

impl TcpServer {
    pub async fn new(addrs: impl ToSocketAddrs) -> Result<TcpServer> {
        Ok(Self {
            incoming: rentals::Incoming::new(
                Box::new(TcpListener::bind(addrs).await?),
                |listener| {
                    listener
                        .incoming()
                        .err_into()
                        .map_ok(TcpConnection::from)
                        .boxed_local()
                },
            ),
        })
    }
}

impl FusedStream for TcpServer {
    fn is_terminated(&self) -> bool {
        false
    }
}

impl Server<TcpConnection> for TcpServer {}

impl Stream for TcpServer {
    type Item = Result<TcpConnection>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Option<Self::Item>> {
        self.incoming
            .rent_mut(|server| Pin::new(server).poll_next(ctx))
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Delimeter(u64);

impl Delimeter {
    fn coded_size() -> usize {
        bincode::serialize(&Delimeter(0))
            .expect("Serialize constant")
            .len()
    }
}

struct WriteJob {
    payload: Vec<u8>,
    position: usize,
}

pub struct TcpConnection {
    stream_read: rentals::StreamRead,
    // write_jobs: Vec<WriteJob>,
}

impl From<TcpStream> for TcpConnection {
    fn from(stream: TcpStream) -> Self {
        Self {
            stream_read: rentals::StreamRead::new(
                Box::new(stream),
                |mut stream| {
                    let mut buffer = vec![0; Delimeter::coded_size()];
                    Some(
                        async move {
                            stream
                                .read_exact(buffer.as_mut_slice())
                                .err_into()
                                .await
                                .map(|_| buffer)
                        }
                        .boxed_local(),
                    )
                },
            ),
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
