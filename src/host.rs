use super::Protocol;
use super::Result;

use event;
use peer;

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum Target {
    All,
    Peer(peer::ID),
}

pub struct Host<P: Debug + Serialize + DeserializeOwned> {
    _payload: PhantomData<P>,
}

impl<P: Debug + Serialize + DeserializeOwned> Host<P> {
    pub fn new(protocol: Protocol) -> Self { Host { _payload: PhantomData } }
    pub fn send(&mut self, target: Target, payload: P) -> Result<()> { Ok(()) }
    pub fn service(&mut self) -> Option<Result<event::Event<P>>> { None }
    pub fn disconnect(&mut self, target: Target) -> Result<()> { Ok(()) }
}
