#[macro_use]
extern crate rental;

use nhanh::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

mod tcp;

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkDatagram {
    pub delivery_mode: DeliveryMode,
    pub id: u64,
    pub data: Vec<u8>,
}
