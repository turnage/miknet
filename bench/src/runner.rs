use crate::*;
use futures::future::join;
use serde::Serialize;
use std::fs;
use std::process::Command;
use structopt::StructOpt;

#[derive(Serialize, Debug, Clone, StructOpt)]
pub struct NetworkConfig {
    /// Delay in milliseconds
    #[structopt(long, default_value = "0")]
    pub delay: u64,
    /// Jitter in milliseconds
    #[structopt(long, default_value = "0")]
    pub jitter: u64,
    /// Delay correlation percentage (range: [0.0-100.0])
    #[structopt(long, default_value = "0")]
    pub delay_correlation: f32,
    /// Independent chance of packet loss (range: [0.0-100.0])
    #[structopt(long, default_value = "0")]
    pub random_loss: f32,
    /// Random packet loss correlation (range: [0.0-100.0])
    #[structopt(long, default_value = "0")]
    pub random_loss_correlation: f32,
    /// Network loopback interface.
    #[structopt(long, default_value = "lo")]
    #[serde(skip)]
    pub interface: String,
    /// Rate limit of simulated wire. Defaults to 1Gigabit.
    #[structopt(long, default_value = "1073741824")]
    pub rate_limit_kbps: usize,
    #[structopt(long, default_value="1000")]
    pub packet_limit: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            delay: 0,
            jitter: 0,
            delay_correlation: 0.0,
            random_loss: 0.0,
            random_loss_correlation: 0.0,
            interface: String::from("lo"),
            rate_limit_kbps: 1073741824,
            packet_limit: 1000
        }
    }
}

impl NetworkConfig {
    fn reset(&self) {
        Command::new("tc")
            .args(&["qdisc", "del", "dev", self.interface.as_str(), "root"])
            .output()
            .expect("resetting network loopback interface");
    }

    fn apply(&self) {
        let output = Command::new("tc")
            .args(&[
                "qdisc",
                "add",
                "dev",
                &self.interface,
                "root",
                "netem",
                "delay",
                &format!("{}", self.delay / 2 * 1000),
                &format!("{}", self.jitter / 2 * 1000),
                &format!("{}", self.delay_correlation),
                "loss",
                "random",
                &format!("{}%", self.random_loss / 2.),
                &format!("{}%", self.random_loss_correlation),
                "rate",
                &format!("{}kbit", self.rate_limit_kbps),
                "limit",
                &format!("{}", self.packet_limit / 2)
            ])
            .output()
            .expect("applying delay");
        assert_eq!(
            output.status.code(),
            Some(0),
            "Failure apllying: {:#?}",
            output
        )
    }
}

#[derive(StructOpt)]
pub struct Options {
    #[structopt(flatten)]
    pub network_config: NetworkConfig,
    #[structopt(flatten)]
    pub client_options: client::Options,
    /// Whether to launch a server in this process at the client's expected
    /// server address.
    #[structopt(long)]
    pub start_server: bool,
    #[structopt(long, short = "o")]
    pub output: Option<String>,
}

async fn run_client(options: &Options) -> Result<client::Summary> {
    client::client_main(options.client_options.clone()).await
}

async fn run(options: &Options) -> Result<client::Summary> {
    let server_options = server::Options {
        address: options.client_options.address,
        protocol: options.client_options.protocol,
    };

    let (results, server_result) =
        join(run_client(&options), server::server_main(server_options)).await;

    server_result.and_then(|_| results)
}

pub async fn runner_main(options: Options) -> Result<client::Summary> {
    options.network_config.reset();
    options.network_config.apply();

    let results = run(&options).await;

    options.network_config.reset();

    let results = results?;

    if let Some(output) = options.output {
        let writer = fs::File::create(output)?;
        let mut writer = csv::Writer::from_writer(writer);

        for report in &results.trip_reports {
            writer.serialize(report)?;
        }
    }

    Ok(results)
}
