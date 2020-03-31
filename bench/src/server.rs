use crate::*;
use anyhow::anyhow;
use async_std::net::SocketAddr;
use bincode::deserialize;
use futures::prelude::*;
use nhanh::*;
use structopt::StructOpt;

async fn run<C>(mut server: impl Server<C> + Unpin) -> Result<()>
where
    C: Connection + Unpin,
{
    let mut client = server.for_each_concurrent(/*limit=*/None, |client| async {
        let client = client.expect("Unwrapping client from listener");
        let (mut client_sink, mut client_stream) = client.split();
        while let Some(datagram) = client_stream.next().await {  
            let benchmark_datagram = datagram.expect("Unwrapping benchmark datagram");
            client_sink.send(SendCmd {
                delivery_mode: DeliveryMode::ReliableOrdered(benchmark_datagram.stream_position.expect("Unwrapping stream position of benchmark diagram").stream_id),
                data: benchmark_datagram.data,
                ..SendCmd::default()
            }).await.expect("Returning benchmark datagram to client");
        }
    }).await;

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

pub async fn server_main(options: Options) {
    let server = match options.protocol {
        Protocol::Tcp => {
            run(tcp::TcpServer::bind(options.address)
                .await
                .expect("binding to tcp server address"))
            .await
        }
        Protocol::Enet => {
            run(enet::EnetServer::bind(options.address).await).await
        }
        p => panic!("unsupported protocol for server: {:?}", p),
    }
    .expect("running benchmark server");
}
