//! host defines the traits and behaviors of hosts in the miknet protocol.

use Result;
use bincode::deserialize;
use event::{Event, ProtoError};
use gram::{Gram, MTU_BYTES};
use std::net::{SocketAddr, UdpSocket};

pub struct Host {
    socket: UdpSocket,
}

impl Host {
    fn new(socket: UdpSocket) -> Self { Self { socket: socket } }

    fn poll(&self) -> Result<(SocketAddr, Vec<Event>)> {
        self.socket.set_nonblocking(false)?;
        let mut buffer = [0; MTU_BYTES];
        let (_, sender) = self.socket.recv_from(&mut buffer)?;
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
        let host = Host::new(receiver);

        sender.send_to(&payload, test_addr)?;
        let (sender_addr, events) = host.poll()?;

        assert_eq!(sender_addr, sender.local_addr()?);
        Ok(events)
    }
}
