use crate::*;

use async_std::{net::SocketAddr, prelude::*};

use futures::{
    self,
    sink::SinkExt,
    stream::{self, select, SelectAll, StreamExt},
};
use futures_timer::Delay;

use serde::Serialize;
use std::str::FromStr;
use std::{
    collections::HashMap,
    iter::FromIterator,
    time::{Duration, Instant},
};
use structopt::StructOpt;

#[derive(Debug, Serialize)]
struct TripReport {
    index: u64,
    round_trip: u128,
}

pub struct Results {
    pub mean: Duration,
    pub deviation: Duration,
}

impl FromIterator<Results> for Results {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Results>,
    {
        let (count, mean_sum, deviation_sum) = iter.into_iter().fold(
            (0, Duration::from_secs(0), Duration::from_secs(0)),
            |(mut count, mut mean_sum, mut deviation_sum), result| {
                mean_sum += result.mean;
                deviation_sum += result.deviation;
                count += 1;
                (count, mean_sum, deviation_sum)
            },
        );
        Results {
            mean: mean_sum / count,
            deviation: deviation_sum / count,
        }
    }
}

impl std::fmt::Debug for Results {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Results")
            .field("Mean", &self.mean)
            .field("Deviation", &self.deviation)
            .finish()
    }
}

impl From<Vec<TripReport>> for Results {
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

        Results { mean, deviation }
    }
}

/// Returns a stream that yields `()` `hertz` times per second.
fn ticker(hertz: u32) -> impl Stream<Item = ()> {
    let tick_rate = Duration::from_secs(1) / hertz;
    stream::repeat(0u8).then(move |_| Delay::new(tick_rate))
}

#[derive(Default, Debug)]
struct TransferTracker {
    total_expected: usize,
    live: HashMap<u64, Instant>,
    returned: Vec<TripReport>,
}

impl TransferTracker {
    fn track_send(&mut self, id: u64) {
        self.live.insert(id, Instant::now());
    }

    fn track_return(&mut self, id: u64) {
        if let Some(sent_time) = self.live.remove(&id) {
            self.returned.push(TripReport {
                index: id,
                round_trip: Instant::now().duration_since(sent_time).as_nanos(),
            });
        }
    }

    fn done(&self) -> bool {
        self.returned.len() >= self.total_expected
    }
}

async fn run(
    options: Options,
    client: impl Connection + Unpin,
) -> Result<Results> {
    enum Input {
        Transfer(TransferCmd),
        Wire(Result<Datagram>),
    }

    let (mut client_sink, client_stream) = client.split();
    let returned_datagrams = client_stream.map(Input::Wire);

    let mut tracking = options
        .transfers
        .iter()
        .filter_map(|tx| {
            tx.return_count.map(|total_expected| {
                (
                    tx.stream_id,
                    TransferTracker {
                        total_expected,
                        ..TransferTracker::default()
                    },
                )
            })
        })
        .collect::<HashMap<StreamId, TransferTracker>>();
    let transfers: SelectAll<_> = options
        .transfers
        .into_iter()
        .map(Transfer::stream)
        .collect();

    let mut input_stream =
        select(transfers.map(Input::Transfer), returned_datagrams);

    loop {
        let input = input_stream.next().await.unwrap();
        match input {
            Input::Wire(returned_datagram) => {
                let returned_datagram: Datagram =
                    returned_datagram.expect("datagram");
                let stream = returned_datagram
                    .stream_position
                    .expect("stream position")
                    .stream_id;
                let benchmark_datagram =
                    bincode::deserialize::<BenchmarkDatagram>(
                        returned_datagram.data.as_slice(),
                    )?;

                if let Some(tracker) = tracking.get_mut(&stream) {
                    tracker.track_return(benchmark_datagram.id);
                }

                if tracking.values().all(TransferTracker::done) {
                    return Ok(tracking
                        .into_iter()
                        .map(|(_, tracker)| tracker.returned)
                        .map(Results::from)
                        .collect());
                }
            }
            Input::Transfer(transfer_cmd) => {
                client_sink.send(transfer_cmd.send_cmd).await?;
                if let Some((cumulative_tracking, cmd_tracking)) =
                    transfer_cmd.tracking.and_then(|cmd_tracking| {
                        let cumulative_tracking =
                            tracking.get_mut(&cmd_tracking.stream_id)?;
                        Some((cumulative_tracking, cmd_tracking))
                    })
                {
                    cumulative_tracking.track_send(cmd_tracking.id);
                }
            }
        }
    }
}

#[derive(Clone, Debug, StructOpt)]
pub struct Options {
    /// Address of the server to run the benchmark against;
    #[structopt(short = "a", default_value = "127.0.0.1:33333")]
    pub address: SocketAddr,
    /// Periodic transfers, specified in terms of
    /// `stream_id:size:hertz:[return_count]`.
    #[structopt(short = "b", long)]
    pub transfers: Vec<Transfer>,
    #[structopt(subcommand)]
    pub protocol: Protocol,
}

struct TransferCmd {
    send_cmd: SendCmd,
    tracking: Option<TransferMessageTracking>,
}

/// Tracking information for a transfer message on the wire.
struct TransferMessageTracking {
    stream_id: StreamId,
    id: u64,
}

#[derive(Clone, Debug, Copy)]
pub struct Transfer {
    pub stream_id: StreamId,
    pub size: usize,
    pub hertz: u32,
    pub return_count: Option<usize>,
}

impl Transfer {
    fn stream(self) -> impl Stream<Item = TransferCmd> {
        let ticker = ticker(self.hertz);
        let mut id = 0;
        ticker.map(move |_| {
            id += 1;
            match self.return_count {
                Some(_) => TransferCmd {
                    send_cmd: self.send_cmd(id),
                    tracking: Some(TransferMessageTracking {
                        stream_id: self.stream_id,
                        id,
                    }),
                },
                None => TransferCmd {
                    send_cmd: self.send_cmd(ID_DO_NOT_RETURN),
                    tracking: None,
                },
            }
        })
    }

    fn send_cmd(&self, id: u64) -> SendCmd {
        let delivery_mode = DeliveryMode::ReliableOrdered(self.stream_id);
        SendCmd {
            delivery_mode,
            data: bincode::serialize(&BenchmarkDatagram {
                id,
                delivery_mode,
                data: vec![0; self.size],
            })
            .expect("to serialize bulk transfer"),
            ..SendCmd::default()
        }
    }
}

impl FromStr for Transfer {
    type Err = std::num::ParseIntError;
    fn from_str(src: &str) -> std::result::Result<Self, Self::Err> {
        let args: Vec<&str> = src.split(":").collect();

        let stream_id = args[0].parse::<u8>()?;
        let size = args[1].parse::<usize>()?;
        let hertz = args[2].parse::<u32>()?;
        let return_count =
            args.get(3).map(|a| a.parse::<usize>()).transpose()?;

        Ok(Self {
            stream_id: StreamId(stream_id),
            size,
            hertz,
            return_count,
        })
    }
}

pub async fn client_main(options: Options) -> Result<Results> {
    let address = options.address;
    match options.protocol {
        Protocol::Tcp => {
            run(
                options,
                loop {
                    let result = tcp::TcpConnection::connect(address).await;

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
            run(options, enet::EnetConnection::connect(address).await).await
        }
    }
}
