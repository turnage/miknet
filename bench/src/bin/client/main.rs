#![recursion_limit = "256"]

use anyhow::{anyhow, bail};
use async_std::{net::SocketAddr, prelude::*};
use bench::*;
use bincode::deserialize;
use futures::{
    self,
    future::FusedFuture,
    sink::SinkExt,
    stream::{self, select, FuturesUnordered, StreamExt },
};
use futures_timer::Delay;
use nhanh::*;
use std::time::{Duration, Instant};
use structopt::StructOpt;

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

    enum Input {
        Tick,
        Wire(Result<Datagram>)
    }

    let (mut client_sink, client_stream) = client.split();    
    let returned_datagrams = client_stream.map(Input::Wire);
    let ticks = ticker.map(|_| Input::Tick);
    let mut input_stream = select(returned_datagrams, ticks);

    while let Some(input) = input_stream.next().await.filter(|_| remaining  > 0) {
        println!("{:?} datagrams remain", remaining);
        match input {
            Input::Wire(returned_datagram) => {
                let returned_datagram: Datagram = returned_datagram?;
                let benchmark_datagram = bincode::deserialize::<BenchmarkDatagram>(returned_datagram.data.as_slice())?;
                let mut report = trip_times.get_mut(benchmark_datagram.id as usize).ok_or(anyhow!("Server returned datagram with id {:?}, but we did not send a datagram with that id", benchmark_datagram.id))?;
                if let Some(return_timestamp) = report.returned {
                    Err(anyhow!("Server returned datagram {:?}, but it already returned a datagram with the same id at {:?}", benchmark_datagram.id, return_timestamp))?;
                }

                remaining -= 1;

            }
            Input::Tick => {
                println!("Tick; sending packet");
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
                client_sink.send(SendCmd {
                    data: bincode::serialize(&benchmark_datagram)?,
                    delivery_mode, ..SendCmd::default()});
            }
        }
    }

    Ok(Report { trip_times })
}

#[derive(Debug, StructOpt)]
struct Options {
    /// Address of the server to run the benchmark against;
    #[structopt(short = "a")]
    address: SocketAddr,
}

#[async_std::main]
async fn main() {
    let options = Options::from_args();

    let connection = tcp::TcpConnection::connect(options.address).await.expect("Opening connection to benchmark server");     

    run(connection).await.expect("Running benchmark");
}
