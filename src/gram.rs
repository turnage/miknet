//! gram defines the atomic unit of the miknet protocol.

use bincode::{Bounded, deserialize, serialize_into};
use event::Event;
use std::io;
use std::net::SocketAddr;
use tokio_core::net::UdpCodec;

pub const MTU: Bounded = Bounded(1400);
pub const MTU_BYTES: usize = 1400;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Ctrl {
    Syn(usize),
    Ack(usize),
    Reset,
}

impl Into<Event> for Ctrl {
    fn into(self) -> Event { Event::Ctrl(self) }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Gram {
    pub cmds: Vec<Ctrl>,
    pub payload: Vec<u8>,
}

impl Into<Vec<Event>> for Gram {
    fn into(mut self) -> Vec<Event> {
        let mut events: Vec<Event> = self.cmds.drain(0..).map(|c| c.into()).collect();
        events.push(Event::Payload(self.payload));
        events
    }
}

/// GramCodec defines the protocol rules for sending grams over udp.
pub struct GramCodec;

impl UdpCodec for GramCodec {
    type In = (SocketAddr, Vec<Event>);
    type Out = (SocketAddr, Vec<u8>);

    fn decode(&mut self, src: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        match deserialize::<Gram>(buf) {
            Ok(gram) => Ok((*src, gram.into())),
            Err(_) => Ok((*src, vec![Event::InvalidGram])),
        }
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
        let (dest, mut payload) = msg;
        buf.append(&mut payload);
        dest
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Result;
    use bincode::serialize;
    use futures::Stream;
    use std::net::{self, SocketAddr};
    use std::str::FromStr;
    use tokio_core::net::UdpSocket;
    use tokio_core::reactor::Core;

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
                 (vec![0, 20, 3], vec![Event::InvalidGram])] {
            assert_eq!(events(gram).expect("to generate events"), expectation);
        }
    }

    fn events(payload: Vec<u8>) -> Result<Vec<Event>> {
        let mut core = Core::new()?;
        let handle = core.handle();
        let (sender, receiver) = (net::UdpSocket::bind("127.0.0.1:0")?,
                                  UdpSocket::bind(&SocketAddr::from_str("127.0.0.1:0")?, &handle)?);
        let test_addr = receiver.local_addr()?;

        sender.send_to(&payload, &test_addr)?;
        let product = match core.run(receiver.framed(GramCodec {}).into_future()) {
            Ok((product, _)) => Ok(product),
            Err((e, _)) => Err(e),
        }?;

        match product {
            Some((sender_addr, events)) => {
                assert_eq!(sender_addr, sender.local_addr()?);
                Ok(events)
            }
            None => panic!("no events in the stream!"),
        }
    }
}
