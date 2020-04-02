use bench::server::*;
use structopt::StructOpt;

#[async_std::main]
async fn main() {
    let options = Options::from_args();

    server_main(options).await.expect("running server");
}
