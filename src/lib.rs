extern crate bincode;
#[macro_use]
extern crate error_chain;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod channel;
pub mod peer;
pub mod host;
pub mod event;

use event::Event;
use host::{Host, Target};

error_chain! {
    foreign_links {
        Io(std::io::Error);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Protocol {
    channels: Vec<channel::Config>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_works() {
        let mut host: Host<usize> = Host::new(Protocol {
            channels: vec![channel::Config::new(channel::Sequence::Sequenced,
                                                channel::Reliability::Reliable)],
        });
        host.send(Target::All, 8);
        while let Some(result) = host.service() {
            match result {
                Ok(event) => {
                    match event {
                        Event::Connect(peer_id) => println!("{:?} connected", peer_id),
                        Event::Message { peer_id: peer_id, payload: payload } => {
                            println!("{:?} sent {:?}", peer_id, payload)
                        }
                        Event::Disconnect(peer_id) => println!("{:?} disconnected", peer_id),
                    }
                }
                Err(error) => println!("Error: {:?}", error),
            }
        }
        if let Err(_) = host.disconnect(host::Target::All) {
            println!("error while disconnecting");
        }
    }
}
