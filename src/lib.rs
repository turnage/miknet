extern crate bincode;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate futures;
extern crate tokio_core;
extern crate rand;
extern crate crypto;

mod gram;
mod host;
mod event;
mod cmd;
mod conn;
mod timers;
mod test_util;

pub use host::Host;

use std::convert::From;
use std::fmt::{self, Display, Formatter};
use std::net::SocketAddr;

#[allow(unused_doc_comment)]
error_chain! {
    foreign_links {
        Io(std::io::Error);
        Bincode(Box<bincode::ErrorKind>);
        Addr(std::net::AddrParseError);
    }
}


impl<T> From<futures::sync::mpsc::SendError<T>> for Error {
    fn from(_: futures::sync::mpsc::SendError<T>) -> Error {
        "failed to send on closed channel".into()
    }
}

impl From<()> for Error {
    fn from(_: ()) -> Error { "Something happenend that ought not have.".into() }
}

#[derive(Eq, Clone, Debug, PartialEq)]
pub enum MEvent {
    ConnectionAttemptTimedOut(SocketAddr),
    ConnectionEstablished(SocketAddr),
    Error(String),
    Shutdown,
}

impl Display for MEvent {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            MEvent::ConnectionAttemptTimedOut(addr) => {
                write!(f, "Connecting to {} timed out.", addr)
            }
            MEvent::ConnectionEstablished(addr) => write!(f, "Connected to {}!", addr),
            MEvent::Error(ref e) => write!(f, "Miknet failed due to error: {}", e),
            MEvent::Shutdown => write!(f, "Miknet host shutdown."),
        }
    }
}
