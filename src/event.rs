use peer;

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[derive(Debug)]
pub enum Event<P: Debug + Serialize + DeserializeOwned> {
    Connect(peer::ID),
    Message { peer_id: peer::ID, payload: P },
    Disconnect(peer::ID),
}
