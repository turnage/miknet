use async_std::net::*;
use futures::channel::mpsc;
use futures::prelude::*;
use futures::stream::FusedStream;
use nhanh::*;
use std::collections::HashMap;
use std::ffi::c_void;
use std::pin::Pin;
use std::task::{Context, Poll};

pub const MAX_CHANNELS: u64 = 256;

#[allow(warnings)]
mod enet {
    include!(concat!(env!("OUT_DIR"), "/enet.rs"));
}

#[derive(Debug)]
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
    new_peer_stream: mpsc::UnboundedReceiver<NewPeer>,
    command_sink: mpsc::Sender<EnetCmd>,
}

async fn socket_addr_to_enet_addr(
    addr: impl ToSocketAddrs,
) -> enet::ENetAddress {
    let socket_address = addr
        .to_socket_addrs()
        .await
        .expect("addresses")
        .next()
        .expect("address");

    unsafe {
        let mut address: enet::ENetAddress = std::mem::uninitialized();
        address.host = match socket_address.ip() {
            IpAddr::V4(v4) => u32::from_le_bytes(v4.octets()),
            _ => panic!("Enet does not support ipv6"),
        };
        address.port = socket_address.port();
        address
    }
}

impl EnetServer {
    pub async fn bind(address: impl ToSocketAddrs) -> Self {
        let address = socket_addr_to_enet_addr(address).await;

        let (command_sink, command_stream) = mpsc::channel(0);
        let (new_peer_sink, new_peer_stream) = mpsc::unbounded();

        std::thread::spawn(enet_service_loop(
            command_stream,
            new_peer_sink,
            HostType::Server,
            address,
        ));

        Self {
            new_peer_stream,
            command_sink,
        }
    }
}

impl Server<EnetConnection> for EnetServer {}

impl FusedStream for EnetServer {
    fn is_terminated(&self) -> bool {
        self.new_peer_stream.is_terminated()
    }
}

impl Stream for EnetServer {
    type Item = Result<EnetConnection>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut Context,
    ) -> Poll<Option<Self::Item>> {
        let new_peer = match Pin::new(&mut self.new_peer_stream).poll_next(ctx)
        {
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
    command_sink: mpsc::Sender<EnetCmd>,
    peer_event_stream: mpsc::UnboundedReceiver<Datagram>,
}

impl EnetConnection {
    pub async fn connect(server_addr: impl ToSocketAddrs) -> Self {
        let address = socket_addr_to_enet_addr(server_addr).await;

        let (command_sink, command_stream) = mpsc::channel(0);
        let (new_peer_sink, mut new_peer_stream) = mpsc::unbounded();

        std::thread::spawn(enet_service_loop(
            command_stream,
            new_peer_sink,
            HostType::Client,
            address,
        ));

        let peer = new_peer_stream.next().await.expect("connection to server");
        Self {
            peer: peer.peer,
            command_sink,
            peer_event_stream: peer.peer_event_stream,
        }
    }
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

enum HostType {
    Server,
    Client,
}

impl HostType {
    fn create(&self, server_addr: enet::ENetAddress) -> *mut enet::ENetHost {
        unsafe {
            let host = match self {
                HostType::Server => {
                    enet::enet_host_create(&server_addr, 32, MAX_CHANNELS, 0, 0)
                }
                HostType::Client => {
                    let client = enet::enet_host_create(
                        0 as *const enet::ENetAddress,
                        32,
                        2,
                        0,
                        0,
                    );
                    assert!(
                        enet::enet_host_connect(client, &server_addr, MAX_CHANNELS, 0,)
                            != 0 as *mut enet::ENetPeer
                    );
                    client
                }
            };
            assert!(host != 0 as *mut enet::ENetHost);
            host
        }
    }
}

fn enet_service_command(command: EnetCmd) {
    let packet = unsafe {
        enet::enet_packet_create(
            command.data.as_ptr() as *const c_void,
            command.data.len() as u64,
            enet::_ENetPacketFlag_ENET_PACKET_FLAG_RELIABLE,
        )
    };

    unsafe {
        let peer = command.peer as *mut enet::ENetPeer;
        enet::enet_peer_send(peer, command.channel, packet)
    };
}

fn enet_service_loop(
    mut command_stream: mpsc::Receiver<EnetCmd>,
    mut new_peer_sink: mpsc::UnboundedSender<NewPeer>,
    host_type: HostType,
    server_addr: enet::ENetAddress,
) -> impl FnOnce() {
    move || {
        assert_eq!(unsafe { enet::enet_initialize() }, 0);

        let host = host_type.create(server_addr);
        let mut total_sent = 0;
        let mut peers = HashMap::new();
        loop {
            match command_stream.try_next().transpose() {
                Some(command) => command,
                None => return,
            }
            .ok()
            .into_iter()
            .for_each(enet_service_command);

            let mut event: enet::ENetEvent =
                unsafe { std::mem::uninitialized() };
            while unsafe {
                enet::enet_host_service(
                    host,
                    { &mut event as *mut enet::ENetEvent },
                    /*ms=*/ 1,
                )
            } > 0
            {
                match event.type_ {
                    enet::_ENetEventType_ENET_EVENT_TYPE_CONNECT => {
                        let (peer_event_sink, peer_event_stream) =
                            mpsc::unbounded();
                        assert!(
                            peers.insert(event.peer, peer_event_sink).is_none(),
                            "Peer already connected."
                        );
                        new_peer_sink
                            .unbounded_send(NewPeer {
                                peer: event.peer as u64,
                                peer_event_stream,
                            })
                            .expect("sending new peer event");
                    }
                    enet::_ENetEventType_ENET_EVENT_TYPE_DISCONNECT => {}
                    enet::_ENetEventType_ENET_EVENT_TYPE_RECEIVE => {
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
                    e => println!("other event type: {:?}", e),
                }
            }
        }
    }
}
