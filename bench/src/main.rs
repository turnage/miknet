
use async_std::net::*;
use bench::*;

use itertools::iproduct;
use nhanh::*;

use serde::Serialize;


fn local_address(port: u16) -> SocketAddr {
    format!("127.0.0.1:{}", port)
        .parse()
        .expect("local address")
}

#[derive(Debug, Clone, Serialize)]
struct NetcodeScenario {
    scenario_name: &'static str,
    #[serde(skip_serializing)]
    transfers: Vec<client::Transfer>,
}

#[derive(Debug, Clone, Serialize)]
struct Scenario {
    #[serde(flatten)]
    netcode_scenario: NetcodeScenario,
    #[serde(flatten)]
    network_config: runner::NetworkConfig,
}

impl Scenario {
    async fn run(&self, port: u16, protocol: Protocol) -> Report {
        let server_address = local_address(port);
        let client_options = client::Options {
            address: server_address,
            protocol,
            transfers: self.netcode_scenario.transfers.clone(),
        };

        let runner_options = runner::Options {
            network_config: self.network_config.clone(),
            client_options,
            start_server: true,
        };

        let results =
            runner::runner_main(runner_options).await.expect(&format!(
                "running scenario {} against protocol {:?}",
                self.netcode_scenario.scenario_name, protocol
            ));

        Report {
            scenario_name: self.netcode_scenario.scenario_name,
            network_delay_ms: self.network_config.delay,
            network_jitter_ms: self.network_config.jitter,
            network_delay_correlation: self.network_config.delay_correlation,
            network_random_packet_loss: self.network_config.random_loss,
            network_random_packet_loss_correlation: self
                .network_config
                .random_loss_correlation,
            network_rate_limit_kilobits: self.network_config.rate_limit_kbps,
            protocol,
            mean_ping_ms: results.mean.as_secs_f64() * 1e3,
            ping_deviation_ms: results.deviation.as_secs_f64() * 1e3,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct Report {
    scenario_name: &'static str,
    /// Delay in milliseconds
    network_delay_ms: u64,
    /// Jitter in milliseconds
    network_jitter_ms: u64,
    /// Delay correlation percentage (range: [0.0-100.0])
    network_delay_correlation: f32,
    /// Independent chance of packet loss (range: [0.0-100.0])
    network_random_packet_loss: f32,
    /// Random packet loss correlation (range: [0.0-100.0])
    network_random_packet_loss_correlation: f32,
    /// Rate limit of simulated wire.
    network_rate_limit_kilobits: usize,
    protocol: Protocol,
    mean_ping_ms: f64,
    ping_deviation_ms: f64,
}

const DEFAULT_RETURN_COUNT: Option<usize> = Some(200);

fn scenarios() -> Vec<Scenario> {
    vec![Scenario {
        netcode_scenario: NetcodeScenario {
            scenario_name: "200B_60hz",
            transfers: vec![client::Transfer {
                stream_id: StreamId(0),
                size: 200,
                hertz: 60,
                return_count: DEFAULT_RETURN_COUNT,
            }],
        },
        network_config: runner::NetworkConfig::default(),
    }]
}

#[async_std::main]
async fn main() {
    let scenarios = scenarios();
    let scenarios = scenarios.iter();

    let protocols = ALL_PROTOCOLS.iter().copied();

    let configurations = iproduct!(scenarios, protocols);

    let output = std::io::stdout();
    let mut writer = csv::Writer::from_writer(output);

    let mut port = 1025;
    for (scenario, protocol) in configurations {
        writer
            .serialize(scenario.run(port, protocol).await)
            .expect("writing report to stdout");
        port += 1;
    }
}
