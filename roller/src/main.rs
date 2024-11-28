use clap::Parser;
use mystiko_roller::scheduler::schedule::run;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct RollerArgs {
    chain_id: Option<u64>,
}

#[tokio::main]
async fn main() {
    let args = RollerArgs::parse();
    let result = run(args.chain_id).await;
    if result.is_err() {
        println!("roller run meet critical error {:?}", result.err());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        main();
    }
}
