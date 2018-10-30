pub mod connection;
pub mod peer;
pub mod protocol;
mod sequence;
mod transducer;
pub mod validation;
pub mod wire;

use self::validation::ValidationError;
use failure_derive::Fail;
use std::fmt::Debug;
