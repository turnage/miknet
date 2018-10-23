use crate::host::HostError;
use failure_derive::Fail;
use std;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Error in host.")]
    Host(HostError),
}
