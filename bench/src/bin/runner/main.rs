use bench::*;
use runner::*;
use structopt::StructOpt;

#[async_std::main]
async fn main() {
    let options = Options::from_args();
    println!("{:?}", runner_main(options).await);
}
