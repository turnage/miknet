#![feature(bind_by_move_pattern_guards)]
#![allow(unused)]
#![feature(custom_attribute)]

//! miknet is a library that implements the mik networking protocol, which is designed to minimize
//! buffering and jitter on logical connections which have many reliable and unreliable streams.
//!
//! Why miknet?
//!
//! Using multiple TCP streams for this will result in a higher retry rate, and drop rate for any
//! coexistent UDP activity, as their isolated congestion control will compete to consume all the
//! bandwidth available. In miknet all streams cooperate to respect the performance priorities set
//! by the user.
//!
//! Using a single TCP stream for multiple channels is simply a nonstarter as each stream will block
//! the others.

mod api;
//mod connection_manager;
mod error;
pub mod host;
mod protocol;
mod random;
