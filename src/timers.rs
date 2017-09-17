//! timers for timeouts.

use Error;
use futures::{Future, Stream};
use futures::sync::mpsc::{UnboundedSender, unbounded};
use std::net::SocketAddr;
use std::time::Duration;
use tokio_timer::wheel;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Timer {
    InitTimer,
    CookieSentTimer,
}

impl Timer {
    fn duration(&self) -> Duration {
        match *self {
            Timer::InitTimer => Duration::new(3, 0),
            Timer::CookieSentTimer => Duration::new(5, 0),
        }
    }
}

pub struct Wheel;

impl Wheel {
    pub fn pipe()
        -> (UnboundedSender<(SocketAddr, Timer)>,
            Box<Stream<Item = (SocketAddr, Timer), Error = Error>>)
    {
        let wheel = wheel().build();
        let (timer_sink, timer_stream) = unbounded();
        (
            timer_sink,
            Box::new(timer_stream.map_err(Error::from).and_then(
                move |(peer, timer)| {
                    wheel.sleep(timer.duration()).map_err(Error::from).then(
                        move |_| Ok((peer, timer)),
                    )
                },
            )),
        )

    }
}
