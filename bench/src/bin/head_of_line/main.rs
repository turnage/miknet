use async_std::{future::timeout, prelude::*};
use bench::*;
use csv::Writer;
use itertools::iproduct;
use rand::random;
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Serialize)]
struct Result {
    rate_limit_kbps: usize,
    mean_ping: Duration,
    ping_deviation: Duration,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct CsvResult {
    rate_limit_kbps: usize,
    mean_ping_ms: f32,
    ping_deviation_ms: f32,
}

impl From<Result> for CsvResult {
    fn from(src: Result) -> Self {
        Self {
            rate_limit_kbps: src.rate_limit_kbps,
            mean_ping_ms: src.mean_ping.as_nanos() as f32 / 1e6,
            ping_deviation_ms: src.ping_deviation.as_nanos() as f32 / 1e6,
        }
    }
}

async fn run_protocol(protocol: Protocol, streams: u8) -> Vec<Result> {
    let mut results = vec![];

    for rate_limit_kbps in (8..11).map(|base: u32| 2usize.pow(base)) {
        println!(
            "\tRunning with rate limit {}kbit, 5% drop rate",
            rate_limit_kbps
        );
        let options = runner::Options {
            start_server: true,
            network_config: runner::NetworkConfig {
                delay: 20,
                jitter: 0,
                delay_correlation: 0.0,
                random_loss: 5.0,
                random_loss_correlation: 0.0,
                interface: String::from("lo"),
                rate_limit_kbps,
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
                payload_count: 200,
                stream_burst_width: 10,
            },
        };

        let run_result = match timeout(
            Duration::from_secs(60),
            runner::runner_main(options),
        )
        .await
        {
            Ok(Ok(run_result)) => {
                println!("\tResult: {:?}", run_result);
                run_result
            }
            Ok(Err(e)) => {
                eprintln!("Run error {:?}; omitting result", e);
                continue;
            }
            Err(_) => {
                eprintln!("Run time out; omitting result",);
                continue;
            }
        };

        results.push(Result {
            rate_limit_kbps,
            mean_ping: run_result.mean,
            ping_deviation: run_result.deviation,
        });
    }

    results
}

#[async_std::main]
async fn main() {
    let configurations = iproduct!(
        ALL_PROTOCOLS.iter().copied(),
        (1..8).step_by(2).map(|b: u32| 2u8.pow(b))
    );
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
