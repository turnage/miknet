use super::Protocol;

#[derive(Debug)]
pub enum Command {
    Connect(Protocol),
    Disconnect,
}


#[derive(Debug)]
pub enum Packet {
    Command(Command),
    Message(Vec<u8>),
}
