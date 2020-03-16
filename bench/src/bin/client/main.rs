#![recursion_limit = "256"]

use anyhow::{anyhow, bail};
use async_std::prelude::*;
use bench::*;
use bincode::deserialize;
use futures::{
    self,
    future::FusedFuture,
    prelude::*,
    stream::{self, FuturesUnordered},
};
use futures_timer::Delay;
use nhanh::*;
use std::time::{Duration, Instant};

pub struct Report {
    trip_times: Vec<TripTimes>,
}

struct TripTimes {
    sent: Instant,
    returned: Option<Instant>,
}

async fn run(mut client: impl Connection + Unpin) -> Result<Report> {
    let bench_seconds = 10;

    let tick_ps = 60;
    let tick_rate = Duration::from_secs(1) / tick_ps;
    let mut ticker = stream::repeat(0u8).then(|_| Delay::new(tick_rate));

    let payload_size = 200;
    let total_datagrams = tick_ps * bench_seconds;
    let mut remaining = total_datagrams;
    let mut trip_times: Vec<TripTimes> = vec![];

    while remaining > 0 {
        futures::select! {
            returned_datagram = client.select_next_some() => {
                let returned_datagram: Datagram = returned_datagram?;
                let benchmark_datagram = bincode::deserialize::<BenchmarkDatagram>(returned_datagram.data.as_slice())?;
                let mut report = trip_times.get_mut(benchmark_datagram.id as usize).ok_or(anyhow!("Server returned datagram with id {:?}, but we did not send a datagram with that id", benchmark_datagram.id))?;
                if let Some(return_timestamp) = report.returned {
                    Err(anyhow!("Server returned datagram {:?}, but it already returned a datagram with the same id at {:?}", benchmark_datagram.id, return_timestamp))?;
                }

                remaining -= 1;

            }
            _ = ticker.select_next_some() => {
                let delivery_mode = DeliveryMode::ReliableOrdered(StreamId(0));
                let data = vec![0; payload_size];
                let benchmark_datagram = BenchmarkDatagram {
                    delivery_mode,
                    id: trip_times.len() as u64,
                    data,
                };

                trip_times.push(TripTimes {
                    sent: Instant::now(),
                    returned: None,
                });
                client.send(&std::io::Cursor::new(bincode::serialize(&benchmark_datagram)?), delivery_mode);
            }
        }
    }

    Ok(Report { trip_times })
}

#[async_std::main]
async fn main() {}
