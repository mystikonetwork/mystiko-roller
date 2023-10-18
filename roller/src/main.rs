use mystiko_roller::scheduler::schedule::run;

#[tokio::main]
async fn main() {
    let result = run().await;
    if result.is_err() {
        println!("roller run meet error {:?}", result.err());
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
