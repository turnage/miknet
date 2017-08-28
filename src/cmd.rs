//! Commands connection state machines can execute.

use MEvent;
use gram::Chunk;
use timers::Timer;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Cmd {
    Chunk(Chunk),
    Net(Vec<u8>),
    Timer(Timer),
    User(MEvent),
}
