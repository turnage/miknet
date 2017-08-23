//! Commands connection state machines can execute.

use MEvent;
use gram::Gram;
use std::time::Duration;

pub enum Cmd {
    Net(Gram),
    Timer(Duration),
    User(MEvent),
}
