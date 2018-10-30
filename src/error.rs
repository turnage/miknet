use crate::{host::HostError, protocol::validation::ValidationError};
use enum_derive::{enum_derive_util, EnumFromInner};
use failure_derive::Fail;
use macro_attr::{macro_attr, macro_attr_impl};
use std;

pub type Result<T> = std::result::Result<T, Error>;

macro_attr! {
    #[derive(Fail, Debug, EnumFromInner!)]
    pub enum Error {
        #[fail(display = "Error in api layer.")]
        Host(HostError),
        #[fail(display = "Error in validation layer.")]
        Validation(ValidationError),
    }
}
