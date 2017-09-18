//! mikhi is the high level component of the miknet protocol responsible for intraconnection logic.

#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub enum Channel {
    Reliable { sequenced: bool },
    Unreliable,
}

impl Default for Channel {
    fn default() -> Self { Channel::Reliable { sequenced: true } }
}

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Config {
    channels: Vec<Channel>,
}

impl Default for Config {
    fn default() -> Self { Self { channels: vec![Channel::default()] } }
}
