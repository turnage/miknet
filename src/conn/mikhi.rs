//! mikhi is the high level component of the miknet protocol responsible for intraconnection logic.

use conn::miklow::Tcb;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Config {
    max_buffer: usize,
}

impl Default for Config {
    fn default() -> Self { Self { max_buffer: 1024 * 1024 } }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Mikhi {
    pub tcb: Tcb,

    cfg: Config,
}

impl Mikhi {
    pub fn new(tcb: Tcb) -> Self { Self { tcb: tcb, cfg: Config::default() } }
}
