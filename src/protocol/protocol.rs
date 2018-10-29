use crate::protocol::connection::ConnectionAction;

pub struct Protocol;

impl Protocol {
    pub fn step(self) -> (Self, Vec<ConnectionAction>) { (self, vec![]) }
}

impl From<ProtocolConfig> for Protocol {
    fn from(config: ProtocolConfig) -> Self { Self {} }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProtocolConfig;
