use std::fmt::Debug;

pub type ID = usize;

#[derive(Debug)]
pub struct Peer<U: Debug> {
    id: ID,
    user_data: U,
}

impl<U: Debug> Peer<U> {
    pub fn new(id: ID, user_data: U) -> Self {
        Peer {
            id: id,
            user_data: user_data,
        }
    }
}
