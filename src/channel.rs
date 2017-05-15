#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Sequence {
    Sequenced,
    Unsequenced,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Reliability {
    Reliable,
    Unreliable,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    sequence: Sequence,
    reliability: Reliability,
}

impl Config {
    pub fn new(sequence: Sequence, reliability: Reliability) -> Self {
        Config {
            sequence: sequence,
            reliability: reliability,
        }
    }
}
