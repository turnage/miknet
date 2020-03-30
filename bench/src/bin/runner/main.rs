use bench::*;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
struct NetworkConfig {
    /// Delay in milliseconds
    #[structopt(default_value = "0")]
    delay: u64,
    /// Jitter in milliseconds
    #[structopt(default_value = "0")]
    jitter: u64,
    /// Delay correlation percentage (range: [0.0-100.0])
    #[structopt(default_value = "0")]
    delay_correlation: f32,
    /// Independent chance of packet loss (range: [0.0-100.0])
    #[structopt(default_value = "0")]
    random_loss: f32,
    /// Random packet loss correlation (range: [0.0-100.0])
    random_loss_correlation: f32,
    /// Network loopback interface.
    #[structopt(default_value = "lo")]
    interface: String,
}

impl NetworkConfig {
    fn reset(&self) {
        Command::new("tc")
            .args(&["qdisc", "del", "dev", self.interface.as_str(), "root"])
            .output()
            .expect("resetting network loopback interface");
    }

    fn apply(&self) {
        self.apply_delay();
        self.apply_loss();
    }

    fn apply_delay(&self) {
        assert_eq!(
            Command::new("tc")
                .args(&[
                    "qdisc",
                    "add",
                    "dev",
                    &self.interface,
                    "delay",
                    &format!("{}", self.delay / 2),
                    &format!("{}", self.jitter / 2),
                    &format!("{}", self.delay_correlation / 2.),
                    "distribution",
                    "normal",
                ])
                .output()
                .expect("applying delay")
                .status
                .code(),
            Some(0)
        )
    }

    fn apply_loss(&self) {
        assert_eq!(
            Command::new("tc")
                .args(&[
                    "qdisc",
                    "add",
                    "dev",
                    &self.interface,
                    "loss",
                    "random",
                    &format!("{}", self.random_loss),
                    &format!("{}", self.random_loss_correlation),
                ])
                .output()
                .expect("applying loss")
                .status
                .code(),
            Some(0)
        );
    }
}

#[derive(StructOpt)]
struct Options {
    #[structopt(flatten)]
    network_config: NetworkConfig,
    #[structopt(subcommand)]
    protocol: Protocol,
}

fn main() {
    let options = Options::from_args();

    options.network_config.reset();
    options.network_config.apply();
}
