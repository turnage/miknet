//! Commands connection state machines can execute.

use MEvent;
use gram::Chunk;
use std::time::Duration;

pub enum Cmd {
    Net(Chunk),
    Timer(Duration),
    User(MEvent),
}
