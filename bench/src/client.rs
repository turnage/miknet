#![recursion_limit = "256"]

use crate::*;
use anyhow::{anyhow, bail};
use async_std::{net::SocketAddr, prelude::*};
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
    fs::File,
    time::{Duration, Instant},
};
use structopt::StructOpt;

#[derive(Serialize)]
struct TripReport {
    index: u64,
    round_trip: u128,
}

#[derive(Debug)]
struct Results {
    stream_reports: HashMap<StreamId, StreamReport>,
}

struct StreamReport {
    mean: Duration,
    deviation: Duration,
    trip_reports: Vec<TripReport>,
}

impl std::fmt::Debug for StreamReport {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("StreamReport")
            .field("Mean", &self.mean)
            .field("Deviation", &self.deviation)
            .finish()
    }
}

impl From<Vec<TripReport>> for StreamReport {
    fn from(src: Vec<TripReport>) -> Self {
        let sum: u128 = src.iter().map(|r| r.round_trip).sum();
        let n = src.len() as u128;
        let mean = sum / n;

        let square_difference = |r: &TripReport| (r.round_trip - mean).pow(2);
        let sum_of_squares: u128 = src.iter().map(square_difference).sum();
        let variance = sum_of_squares / (n - 1);
        let deviation = (variance as f64).sqrt();

        let mean = Duration::from_nanos(mean as u64);
        let deviation = Duration::from_nanos(deviation as u64);

        StreamReport {
            mean,
            deviation,
            trip_reports: src,
        }
    }
}

struct Config {
    payload_size: usize,
}

async fn run(config: Config, mut client: impl Connection + Unpin) -> Results {
    let bench_seconds = 10;

    let tick_ps = 60;
    let tick_rate = Duration::from_secs(1) / tick_ps;
    let mut ticker = stream::repeat(0u8).then(|_| Delay::new(tick_rate));

    let total_datagrams = tick_ps * bench_seconds;
    let mut remaining_to_send = total_datagrams;
    let mut live = HashMap::new();
    let mut stream_reports = HashMap::new();
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
                    return Results {
                        stream_reports: {
                            stream_reports.insert(
                                StreamId(0),
                                StreamReport::from(trip_reports),
                            );
                            stream_reports
                        },
                    };
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
pub struct Options {
    /// Address of the server to run the benchmark against;
    #[structopt(short = "a", default_value = "127.0.0.1:33333")]
    pub address: SocketAddr,
    /// Path to write the report csv to.
    #[structopt(short = "csv")]
    pub csv: bool,
    /// The protocol to benchmark.
    #[structopt(subcommand)]
    pub protocol: Protocol,
    #[structopt(short = "d", default_value = "200")]
    pub payload_size: usize,
}

pub async fn client_main(options: Options) {
    let config = Config {
        payload_size: options.payload_size,
    };

    let results = match options.protocol {
        Protocol::Tcp => {
            run(
                config,
                loop {
                    let result =
                        tcp::TcpConnection::connect(options.address).await;

                    let error = match result {
                        Ok(results) => break results,
                        Err(e) => e,
                    };

                    // The server port is not yet open; give it time.
                    if error.is::<std::io::Error>()
                        && error
                            .downcast_ref::<std::io::Error>()
                            .map(std::io::Error::kind)
                            == Some(std::io::ErrorKind::ConnectionRefused)
                    {
                        continue;
                    }

                    panic!(
                        "Failed to connect to benchmark server: {:?}",
                        error
                    );
                },
            )
            .await
        }
        Protocol::Enet => {
            run(config, enet::EnetConnection::connect(options.address).await)
                .await
        }
        p => panic!("Unsupported protocol for client: {:?}", p),
    };

    println!("{:?}: {:#?}", options.protocol, results);
}
