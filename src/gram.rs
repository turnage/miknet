//! gram defines the atomic unit of the miknet protocol.

use bincode::{Bounded, deserialize, serialize};
use event::Event;

pub const MTU: Bounded = Bounded(1400);
pub const MTU_BYTES: usize = 1400;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Ctrl {
    Syn(usize),
    Ack(usize),
    Reset,
}

impl Into<Event> for Ctrl {
    fn into(self) -> Event { Event::Ctrl(self) }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Gram {
    pub cmds: Vec<Ctrl>,
    pub payload: Vec<u8>,
}

impl Into<Vec<Event>> for Gram {
    fn into(mut self) -> Vec<Event> {
        let mut events: Vec<Event> = self.cmds.drain(0..).map(|c| c.into()).collect();
        events.push(Event::Payload(self.payload));
        events
    }
}
