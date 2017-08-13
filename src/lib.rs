#![allow(unused)]
#![allow(unused_doc_comment)]

extern crate bincode;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serde_derive;
extern crate serde;

mod gram;
mod host;
mod event;
mod peer;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Bincode(Box<bincode::ErrorKind>);
    }
}
