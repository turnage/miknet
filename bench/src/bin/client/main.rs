use bench::client::*;
use structopt::StructOpt;

#[async_std::main]
async fn main() {
    let options = Options::from_args();
    client_main(options).await;
}
