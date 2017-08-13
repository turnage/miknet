//! host defines the traits and behaviors of hosts in the miknet protocol.

use {Error, Result};
use bincode::{deserialize, serialize_into};
use event::{Event, ProtoError, Api};
use gram::{Gram, MTU, MTU_BYTES};
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Sender, channel};
use std::thread;
use serde::Serialize;
use peer::{Dest, Peer};

pub struct Host {
    event_sink: Sender<(SocketAddr, Vec<Event>)>,
    peers: Vec<Peer>,
}

impl Host {
    fn new(socket: UdpSocket) -> Result<Self> {
        let (event_sink, event_feed) = channel();
        let (error_sink, error_feed) = channel();
        let network_event_sink = event_sink.clone();
        let receiver = socket.try_clone()?;
        thread::spawn(|| Host::poll_proc(receiver, network_event_sink, error_sink));
        Ok(Self { event_sink, peers: Vec::new() })
    }
}

impl Host {
    pub fn connect(&self, addr: SocketAddr) -> Result<()>{
        self.event_sink.send((addr, vec![Event::Api(Api::Conn)])).map_err(|e| e.into())
    }

    pub fn disconnect(&self, dest: Dest) -> Result<()> {
        match dest {
            Dest::Peer(p) => self.event_sink.send((p.addr, vec![Event::Api(Api::Disc)])).map_err(Into::into),
            Dest::All => {
                for p in &self.peers {
                    self.event_sink.send((p.addr, vec![Event::Api(Api::Disc)]))?;
                }
                Ok(())
            }
        }
    }

    pub fn send<T: Serialize>(&self, dest: Dest, msg: T) -> Result<()> {
        let mut buf = Vec::new();
        serialize_into(&mut buf, &msg, MTU).map_err(|e| e.into())
    }
}

/// Poll thread implementation.
impl Host {
    fn poll_proc(receiver: UdpSocket,
                 events: Sender<(SocketAddr, Vec<Event>)>,
                 errors: Sender<Error>) {
        loop {
            match Host::poll(&receiver) {
                Err(e) => if let Err(_) = errors.send(e) {
                    return;
                }
                Ok(e) => if let Err(_) = events.send(e) {
                    return;
                }
            }
        }
    }

    fn poll(socket: &UdpSocket) -> Result<(SocketAddr, Vec<Event>)> {
        socket.set_nonblocking(false)?;
        let mut buffer = [0; MTU_BYTES];
        let (_, sender) = socket.recv_from(&mut buffer)?;
        let gram: Result<Gram> = deserialize(&buffer).map_err(|e| e.into());
        match gram {
            Ok(gram) => Ok((sender, gram.into())),
            Err(_) => Ok((sender, vec![Event::ProtoError(ProtoError::InvalidGram)])),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bincode::serialize;
    use gram::{Gram, MTU};
    use gram::Ctrl;

    #[test]
    fn runner() {
        for (gram, expectation) in
            vec![(serialize(&Gram {
                                cmds: vec![Ctrl::Syn(10)],
                                payload: vec![1, 0, 2],
                            },
                            MTU)
                      .expect("seriazed gram"),
                  vec![Event::Ctrl(Ctrl::Syn(10)), Event::Payload(vec![1, 0, 2])]),
                 (vec![0, 20, 3], vec![Event::ProtoError(ProtoError::InvalidGram)])] {
            assert_eq!(events(gram).expect("to generate events"), expectation);
        }
    }

    fn events(payload: Vec<u8>) -> Result<Vec<Event>> {
        let (sender, receiver) = (UdpSocket::bind("localhost:0")?, UdpSocket::bind("localhost:0")?);
        let test_addr = receiver.local_addr()?;

        sender.send_to(&payload, test_addr)?;
        let (sender_addr, events) = Host::poll(&receiver)?;

        assert_eq!(sender_addr, sender.local_addr()?);
        Ok(events)
    }
}
