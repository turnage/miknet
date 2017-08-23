//! Connections

use MEvent;
use cmd::Cmd;
use event::Event;
use gram::{Ctrl, Gram};
use rand::random;

/// All connection states, based on TCP states. The three way handshake takes place on top of UDP.
pub enum Conn {
    Listen,
    SynSent(u32),
    SynRecvd { ours: u32, theirs: u32 },
    Established { ours: u32, theirs: u32 },
    CloseWait { ours: u32, theirs: u32 },
    LastAck(u32),
    FinWait1 { ours: u32, theirs: u32 },
    FinWait2(u32),
    Closing(u32),
    TimeWait,
    Closed,
}

impl Conn {
    fn transition(self, event: Event) -> (Self, Vec<Cmd>) {
        match (self, event) {
            (Conn::Listen, Event::Ctrl(Ctrl::Syn(theirs))) => {
                let ours = random();
                (Conn::SynRecvd { ours: ours, theirs: theirs },
                 vec![Cmd::Net(Gram {
                          cmds: vec![Ctrl::Syn(ours), Ctrl::Ack(theirs)],
                          frag: None,
                      })])
            }
            (Conn::SynSent(ours), Event::Ctrl(Ctrl::Syn(theirs))) => {
                (Conn::SynRecvd { ours: ours, theirs: theirs },
                 vec![Cmd::Net(Gram { cmds: vec![Ctrl::Ack(theirs)], frag: None })])
            }
            (Conn::SynRecvd { ours, theirs }, Event::Ctrl(Ctrl::Ack(ack))) if ours == ack => {
                (Conn::Established { ours: ours, theirs: theirs }, vec![])
            }
            (current, _) => (current, vec![Cmd::Net(Gram { cmds: vec![Ctrl::Reset], frag: None })]),
        }
    }
}
