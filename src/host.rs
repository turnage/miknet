//! A user handle to their own host in a miknet connection(s).

use Result;
use event::{Api, Event};
use futures::{Future, Sink};
use futures::unsync::mpsc::UnboundedSender;
use std::net::SocketAddr;

/// Defines user api calls for miknet connections.
pub struct Host {
    tx: UnboundedSender<(SocketAddr, Event)>,
}

impl Host {
    pub(crate) fn new(tx: UnboundedSender<(SocketAddr, Event)>) -> Self { Self { tx: tx } }

    pub fn connect(&self, addr: &SocketAddr) -> Result<()> {
        self.queue(*addr, Event::Api(Api::Conn))
    }

    pub fn disconnect(&self, addr: &SocketAddr) -> Result<()> {
        self.queue(*addr, Event::Api(Api::Disc))
    }

    fn queue(&self, addr: SocketAddr, event: Event) -> Result<()> {
        self.tx.clone().send((addr, event)).wait()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::Stream;
    use futures::unsync::mpsc::unbounded;
    use std::net::SocketAddr;
    use std::str::FromStr;

    #[test]
    fn it_works() {
        let (tx, rx) = unbounded();
        let host = Host::new(tx);
        let addr = SocketAddr::from_str("127.0.0.1:0").expect("loopback addr");
        host.connect(&addr);

        if let Ok((Some((dest_addr, event)), _)) = rx.into_future().wait() {
            assert_eq!(dest_addr, addr);
            assert_eq!(event, Event::Api(Api::Conn));
        } else {
            panic!("no api event!");
        }
    }
}
