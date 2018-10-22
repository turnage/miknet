use crate::protocol::connection::ConnectionAction;

pub struct Protocol;

impl Protocol {
    pub fn step(self) -> (Self, Vec<ConnectionAction>) { (self, vec![]) }
}

pub struct ProtocolBuilder {}

impl ProtocolBuilder {
    pub fn new() -> Self { Self {} }

    pub fn build(&self) -> Protocol { Protocol {} }
}
