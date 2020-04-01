use async_std::{future::timeout, prelude::*};
use bench::*;
use csv::Writer;
use itertools::iproduct;
use nhanh::*;
use rand::random;
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Serialize)]
struct Result {
    rate_limit_kbps: usize,
    bulk_transfer_size: usize,
    mean_ping: Duration,
    ping_deviation: Duration,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct CsvResult {
    rate_limit_kbps: usize,
    bulk_transfer_size: usize,
    mean_ping_ms: f32,
    ping_deviation_ms: f32,
}

impl From<Result> for CsvResult {
    fn from(src: Result) -> Self {
        Self {
            rate_limit_kbps: src.rate_limit_kbps,
            bulk_transfer_size: src.bulk_transfer_size,
            mean_ping_ms: src.mean_ping.as_nanos() as f32 / 1e6,
            ping_deviation_ms: src.ping_deviation.as_nanos() as f32 / 1e6,
        }
    }
}

async fn run_protocol(
    protocol: Protocol,
    rate_limit_kbps: usize,
) -> Vec<Result> {
    const KB: usize = 1024;

    let mut results = vec![];

    for bulk_transfer_size in vec![1, 4, 16].into_iter().map(|b| b * KB) {
        println!(
            "\tRunning with rate limit {}kbit, 5% drop rate, {}KiB bulk transfer",
            rate_limit_kbps,
            bulk_transfer_size / 1024,
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
                streams: 1,
                protocol,
                payload_size: 200,
                payload_count: 200,
                stream_burst_width: 10,
                bulk_transfers: vec![client::BulkTransfer {
                    stream_id: StreamId(2),
                    size: bulk_transfer_size,
                }],
            },
        };

        let run_result = match timeout(
            Duration::from_secs(60),
            runner::runner_main(options),
        )
        .await
        {
            Ok(Ok(run_result)) => {
                println!("\t\tResult: {:?}", run_result);
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
            bulk_transfer_size,
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
        vec![4096, 134217728].into_iter()
    );
    for (protocol, rate_limit_kbps) in configurations {
        eprintln!(
            "Running {:?} with {:?}kbps bandwidth...",
            protocol, rate_limit_kbps
        );
        let results = run_protocol(protocol, rate_limit_kbps).await;
        let path = format!("{:?}_{:?}_kbps.csv", protocol, rate_limit_kbps);
        let mut writer = Writer::from_path(path).expect("Opening output file");
        results.into_iter().map(CsvResult::from).for_each(|result| {
            writer.serialize(result).expect("Writing result to file");
        });
    }
}
