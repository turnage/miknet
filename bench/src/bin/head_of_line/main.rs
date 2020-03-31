use async_std::{future::timeout, prelude::*};
use bench::*;
use csv::Writer;
use itertools::iproduct;
use rand::random;
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Serialize)]
struct Result {
    random_loss: f32,
    mean_ping: Duration,
    ping_deviation: Duration,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct CsvResult {
    random_loss: f32,
    mean_ping_ms: f32,
    ping_deviation_ms: f32,
}

impl From<Result> for CsvResult {
    fn from(src: Result) -> Self {
        Self {
            random_loss: src.random_loss,
            mean_ping_ms: src.mean_ping.as_nanos() as f32 / 1e6,
            ping_deviation_ms: src.ping_deviation.as_nanos() as f32 / 1e6,
        }
    }
}

async fn run_protocol(protocol: Protocol, streams: u8) -> Vec<Result> {
    let mut results = vec![];

    for random_loss in (0..40).step_by(5).map(|p| p as f32) {
        println!("\tRunning with random loss {}%", random_loss);
        let options = runner::Options {
            start_server: true,
            network_config: runner::NetworkConfig {
                delay: 0,
                jitter: 0,
                delay_correlation: 0.0,
                random_loss,
                random_loss_correlation: 0.0,
                interface: String::from("lo"),
            },
            client_options: client::Options {
                address: format!(
                    "127.0.0.1:{}",
                    (random::<usize>() + 1024) % 65535
                )
                .parse()
                .unwrap(),
                streams,
                protocol,
                payload_size: 200,
                payload_count: 10,
                stream_burst_width: 10,
            },
        };

        let run_result = match timeout(
            Duration::from_secs(20),
            runner::runner_main(options),
        )
        .await
        {
            Ok(Ok(run_result)) => run_result,
            Ok(Err(e)) => {
                eprintln!(
                    "Run with random loss {:?} had error {:?}; omitting result",
                    random_loss, e
                );
                continue;
            }
            Err(_) => {
                eprintln!(
                    "Run with random loss {:?} time out; omitting result",
                    random_loss
                );
                continue;
            }
        };
        results.push(Result {
            random_loss,
            mean_ping: run_result.mean,
            ping_deviation: run_result.deviation,
        });
    }

    results
}

#[async_std::main]
async fn main() {
    let configurations = iproduct!(ALL_PROTOCOLS.iter().copied(), (1..4));
    for (protocol, stream_count) in configurations {
        eprintln!("Running {:?} with {:?} streams...", protocol, stream_count);
        let results = run_protocol(protocol, stream_count).await;
        let path = format!("{:?}_{:?}_streams.csv", protocol, stream_count);
        let mut writer = Writer::from_path(path).expect("Opening output file");
        results.into_iter().map(CsvResult::from).for_each(|result| {
            writer.serialize(result).expect("Writing result to file");
        });
    }
}
