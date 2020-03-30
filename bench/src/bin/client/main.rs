#![recursion_limit = "256"]

use anyhow::{anyhow, bail};
use async_std::{net::SocketAddr, prelude::*};
use bench::*;
use bincode::deserialize;
use futures::{
    self,
    future::FusedFuture,
    sink::SinkExt,
    stream::{self, select, FuturesUnordered, StreamExt},
};
use futures_timer::Delay;
use nhanh::*;
use serde::Serialize;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use structopt::StructOpt;

#[derive(Serialize)]
struct TripReport {
    index: u64,
    round_trip: u128,
}

struct Config {
    payload_size: usize,
}

async fn run(
    config: Config,
    mut client: impl Connection + Unpin,
) -> Vec<TripReport> {
    let bench_seconds = 10;

    let tick_ps = 60;
    let tick_rate = Duration::from_secs(1) / tick_ps;
    let mut ticker = stream::repeat(0u8).then(|_| Delay::new(tick_rate));

    let total_datagrams = tick_ps * bench_seconds;
    let mut remaining_to_send = total_datagrams;
    let mut live = HashMap::new();
    let mut trip_reports = vec![];

    enum Input {
        Tick,
        Wire(Result<Datagram>),
    }

    let (mut client_sink, client_stream) = client.split();
    let returned_datagrams = client_stream.map(Input::Wire);
    let ticks = ticker.map(|_| Input::Tick);
    let mut input_stream = select(returned_datagrams, ticks);

    loop {
        let input = input_stream.next().await.unwrap();
        match input {
            Input::Wire(returned_datagram) => {
                let returned_datagram: Datagram =
                    returned_datagram.expect("datagram");
                let benchmark_datagram =
                    bincode::deserialize::<BenchmarkDatagram>(
                        returned_datagram.data.as_slice(),
                    )
                    .expect("deserializing");

                let return_time = Instant::now();
                let send_time = live.remove(&benchmark_datagram.id).unwrap();
                let round_trip = return_time.duration_since(send_time);
                trip_reports.push(TripReport {
                    index: benchmark_datagram.id,
                    round_trip: round_trip.as_nanos(),
                });

                if remaining_to_send == 0 && live.is_empty() {
                    return trip_reports;
                }
            }
            Input::Tick => {
                let delivery_mode = DeliveryMode::ReliableOrdered(StreamId(0));
                let data = vec![0; config.payload_size];
                let id = (total_datagrams - remaining_to_send) as u64;
                let benchmark_datagram = BenchmarkDatagram {
                    delivery_mode,
                    id,
                    data,
                };

                client_sink
                    .send(SendCmd {
                        data: bincode::serialize(&benchmark_datagram)
                            .expect("serializing"),
                        delivery_mode,
                        ..SendCmd::default()
                    })
                    .await;

                let old = remaining_to_send;
                remaining_to_send = remaining_to_send.saturating_sub(1);
                if old != remaining_to_send {
                    live.insert(id, Instant::now());
                }
            }
        }
    }
}

#[derive(Debug, StructOpt)]
enum Protocol {
    Tcp,
    Enet,
}

#[derive(Debug, StructOpt)]
struct Options {
    /// Address of the server to run the benchmark against;
    #[structopt(short = "a", default_value = "127.0.0.1:33333")]
    address: SocketAddr,
    /// Path to write the report csv to.
    #[structopt(short = "o")]
    output: Option<String>,
    /// The protocol to benchmark.
    #[structopt(subcommand)]
    protocol: Protocol,
    #[structopt(short = "d", default_value = "200")]
    payload_size: usize,
}

#[async_std::main]
async fn main() {
    let options = Options::from_args();

    let config = Config {
        payload_size: options.payload_size,
    };

    let report = match options.protocol {
        Protocol::Tcp => {
            run(
                config,
                tcp::TcpConnection::connect(options.address)
                    .await
                    .expect("Opening connection to benchmark server"),
            )
            .await
        }
        Protocol::Enet => {
            run(config, enet::EnetConnection::connect(options.address).await)
                .await
        }
    };

    let mut writer: csv::Writer<Box<dyn std::io::Write>> = match options.output
    {
        Some(output) => {
            let file = std::fs::File::create(output)
                .expect("Creating csv output file");
            csv::Writer::from_writer(Box::new(file))
        }
        None => csv::Writer::from_writer(Box::new(std::io::stdout())),
    };
    report.into_iter().for_each(|trip_report| {
        writer
            .serialize(trip_report)
            .expect("Writing record to csv");
    });
}
