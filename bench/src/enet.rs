use async_std::net::*;
use futures::channel::mpsc;
use futures::prelude::*;
#[deny(all)]
use futures::stream::FusedStream;
use nhanh::*;
use std::collections::HashMap;
use std::ffi::c_void;
use std::pin::Pin;
use std::task::{Context, Poll};

include!(concat!(env!("OUT_DIR"), "/enet.rs"));

struct NewPeer {
    peer: u64,
    peer_event_stream: mpsc::UnboundedReceiver<Datagram>,
}

struct EnetCmd {
    peer: u64,
    channel: u8,
    data: Vec<u8>,
}

pub struct EnetServer {
    event_stream: mpsc::UnboundedReceiver<NewPeer>,
    command_sink: mpsc::UnboundedSender<EnetCmd>,
}

async fn socket_addr_to_enet_addr(addr: impl ToSocketAddrs) -> ENetAddress {
    let socket_address = addr
        .to_socket_addrs()
        .await
        .expect("addresses")
        .next()
        .expect("address");

    unsafe {
        let mut address: ENetAddress = std::mem::uninitialized();
        address.host = ENET_HOST_ANY;
        address.port = socket_address.port();
        address
    }
}

impl EnetServer {
    pub async fn bind(address: impl ToSocketAddrs) -> Self {
        let address = socket_addr_to_enet_addr(address).await;

        let (command_sink, mut command_stream) = mpsc::unbounded();
        let (mut event_sink, event_stream) = mpsc::unbounded();

        std::thread::spawn(move || {
            let host = unsafe { enet_host_create(&address, 30, 2, 0, 0) };
            loop {
                let command = command_stream.try_next().transpose();
                let command = match command {
                    Some(command) => command,
                    None => return,
                };
                if let Ok(command) = command {
                    let command: EnetCmd = command;
                    let packet = unsafe {
                        enet_packet_create(command.data.as_ptr() as *const c_void, command.data.len() as u64,_ENetPacketFlag_ENET_PACKET_FLAG_UNRELIABLE_FRAGMENT)
                    };

                    unsafe {
                        enet_peer_send(
                            command.peer as *mut ENetPeer,
                            command.channel,
                            packet,
                        )
                    };
                }

                let mut total_sent = 0;
                let mut peers = HashMap::new();
                let mut event: ENetEvent = unsafe { std::mem::uninitialized() };
                while unsafe {
                    enet_host_service(
                        host,
                        { &mut event as *mut ENetEvent },
                        /*ms=*/ 100,
                    )
                } > 0
                {
                    match event.type_ {
                        ENET_EVENT_TYPE_CONNECT => {
                            let (peer_event_sink, peer_event_stream) =
                                mpsc::unbounded();
                            peers.insert(event.peer, peer_event_sink);
                            event_sink
                                .unbounded_send(NewPeer {
                                    peer: event.peer as u64,
                                    peer_event_stream,
                                })
                                .expect("sending new peer event");
                        }
                        ENET_EVENT_TYPE_RECEIVE => {
                            let sink =
                                peers.get_mut(&event.peer).expect("peer sink");
                            let data: &'static [u8] = unsafe {
                                let packet = &mut *event.packet;
                                std::slice::from_raw_parts(
                                    packet.data,
                                    packet.dataLength as usize,
                                )
                            };
                            sink.unbounded_send(Datagram {
                                data: data.to_vec(),
                                stream_position: Some(StreamPosition {
                                    stream_id: StreamId(event.channelID),
                                    index: StreamIndex::Ordinal(total_sent),
                                }),
                            })
                            .expect("sending peer the event");

                            total_sent += 1;
                        }
                    }
                }
            }
        });

        Self {
            event_stream,
            command_sink,
        }
    }
}

impl Server<EnetConnection> for EnetServer {}

impl FusedStream for EnetServer {
    fn is_terminated(&self) -> bool {
        self.event_stream.is_terminated()
    }
}

impl Stream for EnetServer {
    type Item = Result<EnetConnection>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Option<Self::Item>> {
        let new_peer = match Pin::new(&mut self.event_stream).poll_next(ctx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Some(event)) => event,
            Poll::Ready(None) => return Poll::Ready(None),
        };

        Poll::Ready(Some(Ok(EnetConnection {
            peer: new_peer.peer,
            command_sink: self.command_sink.clone(),
            peer_event_stream: new_peer.peer_event_stream,
        })))
    }
}

pub struct EnetConnection {
    peer: u64,
    command_sink: mpsc::UnboundedSender<EnetCmd>,
    peer_event_stream: mpsc::UnboundedReceiver<Datagram>,
}

impl Connection for EnetConnection {}

impl FusedStream for EnetConnection {
    fn is_terminated(&self) -> bool {
        self.peer_event_stream.is_terminated()
    }
}

impl Stream for EnetConnection {
    type Item = Result<Datagram>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.peer_event_stream)
            .poll_next(ctx)
            .map(|d| d.map(Ok))
    }
}

impl Sink<SendCmd> for EnetConnection {
    type Error = Box<dyn std::error::Error>;
    fn poll_ready(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.command_sink)
            .poll_ready(ctx)
            .map_err(Into::into)
    }
    fn start_send(mut self: Pin<&mut Self>, item: SendCmd) -> Result<()> {
        let channel = match item.delivery_mode {
            DeliveryMode::ReliableOrdered(StreamId(channel)) => channel as u8,
            _ => panic!("benchmark only supports reliable ordered datagrams"),
        };

        let peer = self.peer;
        Pin::new(&mut self.command_sink)
            .start_send(EnetCmd {
                peer,
                channel,
                data: item.data,
            })
            .map_err(Into::into)
    }
    fn poll_flush(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.command_sink)
            .poll_flush(ctx)
            .map_err(Into::into)
    }
    fn poll_close(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.command_sink)
            .poll_close(ctx)
            .map_err(Into::into)
    }
}
