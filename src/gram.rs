//! gram defines the atomic unit of the miknet protocol.

use bincode::deserialize;
use conn::StateCookie;
use event::Event;
use std::io;
use std::net::SocketAddr;
use tokio_core::net::UdpCodec;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Chunk {
    Init { token: u32, tsn: u32 },
    InitAck { token: u32, tsn: u32, state_cookie: StateCookie },
    CookieEcho(StateCookie),
    CookieAck,
    Shutdown,
    ShutdownAck,
    ShutdownComplete,
}

impl Into<Event> for Chunk {
    fn into(self) -> Event { Event::Chunk(self) }
}

/// Gram is the atomic unit of the miknet protocol. All transmissions are represented as a gram
/// before they are written on the network.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Gram {
    pub token: u32,
    pub chunks: Vec<Chunk>,
}

impl Gram {
    pub fn events(self, expected_token: Option<u32>) -> Vec<Event> {
        match expected_token {
            Some(expectation) if self.token != expectation => vec![Event::InvalidGram],
            _ => self.chunks.into_iter().map(Chunk::into).collect(),
        }
    }
}

/// GramCodec defines the protocol rules for sending grams over udp.
pub struct GramCodec;

impl UdpCodec for GramCodec {
    type In = Option<(SocketAddr, Event)>;
    type Out = (SocketAddr, Vec<u8>);

    fn decode(&mut self, src: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        match deserialize::<Gram>(buf) {
            Ok(gram) => Ok(Some((*src, Event::Gram(gram)))),
            Err(_) => Ok(None),
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
    use bincode::{Infinite, serialize};
    use futures::Stream;
    use std::net::{self, SocketAddr};
    use std::str::FromStr;
    use tokio_core::net::UdpSocket;
    use tokio_core::reactor::Core;

    #[test]
    fn runner() {
        let gram = Gram { token: 0, chunks: vec![Chunk::CookieAck] };
        assert_eq!(
            events(serialize(&gram, Infinite).expect("serialized_gram"))
                .expect("to generate events"),
            Event::Gram(gram)
        );
    }

    fn events(payload: Vec<u8>) -> Result<Event> {
        let mut core = Core::new()?;
        let handle = core.handle();
        let (sender, receiver) = (
            net::UdpSocket::bind("127.0.0.1:0")?,
            UdpSocket::bind(&SocketAddr::from_str("127.0.0.1:0")?, &handle)?,
        );
        let test_addr = receiver.local_addr()?;

        sender.send_to(&payload, &test_addr)?;
        let product = match core.run(receiver.framed(GramCodec {}).into_future()) {
            Ok((product, _)) => Ok(product),
            Err((e, _)) => Err(e),
        }?;

        match product {
            Some(Some((sender_addr, event))) => {
                assert_eq!(sender_addr, sender.local_addr()?);
                Ok(event)
            }
            _ => panic!("no events in the stream!"),
        }
    }
}
