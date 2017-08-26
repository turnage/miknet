extern crate bincode;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate futures;
extern crate tokio_core;
extern crate rand;

mod gram;
mod host;
mod event;
mod cmd;

#[allow(unused_doc_comment)]
error_chain! {
    foreign_links {
        Io(std::io::Error);
        Bincode(Box<bincode::ErrorKind>);
        Addr(std::net::AddrParseError);
    }
}


impl<T> std::convert::From<futures::unsync::mpsc::SendError<T>> for Error {
    fn from(_: futures::unsync::mpsc::SendError<T>) -> Error {
        "failed to send on closed channel".into()
    }
}

pub enum MEvent {
}
