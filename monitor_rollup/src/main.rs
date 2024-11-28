use clap::Parser;
use mystiko_monitor_rollup::start_monitor_rollup;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct MonitorArgs {
    config_path: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = MonitorArgs::parse();
    match start_monitor_rollup(args.config_path).await {
        Ok(scheduler) => {
            log::info!("monitor_rollup has been started successfully!");
            let result = scheduler.wait_shutdown().await;
            log::info!("monitor_rollup has been stopped with result: {:?}", result);
        }
        Err(err) => {
            eprintln!("monitor_rollup start meet error: {}", err);
        }
    }
}
