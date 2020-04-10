use crate::*;

use async_std::net::SocketAddr;

use futures::prelude::*;

use structopt::StructOpt;

async fn run<C>(mut server: impl Server<C> + Unpin) -> Result<()>
where
    C: Connection + Unpin,
{
    let client = server.next().await.expect("client").expect("Ok(client)");
    let (mut client_sink, mut client_stream) = client.split();

    while let Some(Ok(wire_datagram)) = client_stream.next().await {
        let position = wire_datagram.stream_position.expect("position");
        let stream_id = position.stream_id;
        let benchmark_datagram: BenchmarkDatagram =
            bincode::deserialize(&wire_datagram.data).expect("valid datagram");
        if benchmark_datagram.id != ID_DO_NOT_RETURN {
            client_sink
                .send(SendCmd {
                    delivery_mode: DeliveryMode::ReliableOrdered(stream_id),
                    data: wire_datagram.data,
                    ..SendCmd::default()
                })
                .await?;
        }
    }

    Ok(())
}

#[derive(Debug, StructOpt)]
pub struct Options {
    /// Address to serve the benchmark on.
    #[structopt(short = "a", default_value = "127.0.0.1:33333")]
    pub address: SocketAddr,
    /// The protocol to benchmark.
    #[structopt(subcommand)]
    pub protocol: Protocol,
}

pub async fn server_main(options: Options) -> Result<()> {
    match options.protocol {
        Protocol::Tcp => match tcp::TcpServer::bind(options.address).await {
            Ok(server) => run(server).await,
            Err(e) => panic!("Failed to bind server: {:?}", e),
        },
        Protocol::Enet => {
            run(enet::EnetServer::bind(options.address).await).await
        }
        Protocol::Kcp => {
            run(kcp::KcpServer::bind(kcp::KcpMode::Normal, options.address)
                .await?)
            .await
        }
        Protocol::KcpTurbo => {
            run(kcp::KcpServer::bind(kcp::KcpMode::Turbo, options.address)
                .await?)
            .await
        }
    }
}
