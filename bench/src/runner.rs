use crate::*;

use futures::future::join;
use std::process::Command;

use structopt::StructOpt;

#[derive(StructOpt)]
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
    pub interface: String,
    #[structopt(long, default_value = "20000")]
    pub rate_limit_kbps: usize,
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
                &format!("{}", self.random_loss_correlation),
                "rate",
                &format!("{}kbit", self.rate_limit_kbps),
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
}

async fn run_client(options: &Options) -> client::Results {
    client::client_main(options.client_options.clone()).await
}

async fn run(options: &Options) -> Result<client::Results> {
    let server_options = server::Options {
        address: options.client_options.address,
        protocol: options.client_options.protocol,
    };

    let (results, server_result) =
        join(run_client(&options), server::server_main(server_options)).await;

    server_result.map(|_| results)
}

pub async fn runner_main(options: Options) -> Result<client::Results> {
    options.network_config.reset();
    options.network_config.apply();

    let results = run(&options).await;

    options.network_config.reset();

    results
}
