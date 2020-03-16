use anyhow::anyhow;
use async_std::prelude::*;
use bench::*;
use bincode::deserialize;
use futures::prelude::*;
use nhanh::*;

async fn run<C>(mut server: impl Server<C> + Unpin) -> Result<()>
where
    C: Connection + Unpin,
{
    let mut client = server.try_next().await?.ok_or(anyhow!(
        "Server socket closed without receiving a connection"
    ))?;

    while let Some(datagram) = client.try_next().await? {
        let benchmark_datagram =
            deserialize::<BenchmarkDatagram>(datagram.data.as_slice())?;
        client.send(
            &std::io::Cursor::new(benchmark_datagram.data),
            benchmark_datagram.delivery_mode,
        );
    }

    Ok(())
}

#[async_std::main]
async fn main() {}
