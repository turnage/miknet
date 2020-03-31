use async_std::net::*;
use bench::*;
use nix::{
    sys::signal::{kill, Signal},
    unistd::{fork, ForkResult, Pid},
};
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
struct NetworkConfig {
    /// Delay in milliseconds
    #[structopt(long, default_value = "0")]
    delay: u64,
    /// Jitter in milliseconds
    #[structopt(long, default_value = "0")]
    jitter: u64,
    /// Delay correlation percentage (range: [0.0-100.0])
    #[structopt(long, default_value = "0")]
    delay_correlation: f32,
    /// Independent chance of packet loss (range: [0.0-100.0])
    #[structopt(long, default_value = "0")]
    random_loss: f32,
    /// Random packet loss correlation (range: [0.0-100.0])
    #[structopt(long, default_value = "0")]
    random_loss_correlation: f32,
    /// Network loopback interface.
    #[structopt(long, default_value = "lo")]
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
        let output = Command::new("tc")
            .args(&[
                "qdisc",
                "add",
                "dev",
                &self.interface,
                "root",
                "netem",
                "delay",
                &format!("{}", self.delay / 2),
                &format!("{}", self.jitter / 2),
                &format!("{}", self.delay_correlation),
                "loss",
                "random",
                &format!("{}", self.random_loss / 2.),
                &format!("{}", self.random_loss_correlation),
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
struct Options {
    #[structopt(flatten)]
    network_config: NetworkConfig,
    #[structopt(subcommand)]
    protocol: Protocol,
    #[structopt(default_value = "200")]
    payload_size: usize,
}

fn server_address() -> SocketAddr {
    "127.0.0.1:33333".parse().expect("server address")
}

async fn launch_server(options: &Options) -> Option<Pid> {
    match fork().expect("forking server") {
        ForkResult::Parent { child, .. } => return Some(child),
        ForkResult::Child => {
            server::server_main(server::Options {
                address: server_address(),
                protocol: options.protocol,
            })
            .await;
            None
        }
    }
}

async fn run_client(options: &Options) {
    loop {
        let result = client::client_main(client::Options {
            address: server_address(),
            csv: false,
            protocol: options.protocol,
            payload_size: options.payload_size,
        })
        .await;

        return;
    }
}

async fn run(options: &Options) {
    let pid = launch_server(&options).await.expect("server pid");
    run_client(&options).await;
    kill(pid, Some(Signal::SIGKILL))
        .expect(&format!("killing server with pid {:?}", pid));
}

#[async_std::main]
async fn main() {
    let mut options = Options::from_args();

    options.network_config.reset();
    options.network_config.apply();

    let protocols = options.protocol.into_iter();
    for protocol in protocols {
        options.protocol = protocol;
        run(&options).await;
    }

    options.network_config.reset();
}
