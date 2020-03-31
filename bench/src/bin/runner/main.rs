use bench::*;
use runner::*;
use structopt::StructOpt;

#[async_std::main]
async fn main() {
    let options = Options::from_args();
    runner_main(options).await;
}
