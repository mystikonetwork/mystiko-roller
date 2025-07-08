use clap::Parser;
use mystiko_roller_monitor::start_monitor;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct MonitorArgs {
    config_path: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = MonitorArgs::parse();
    match start_monitor(args.config_path).await {
        Ok(scheduler) => {
            log::info!("roller_monitor has been started successfully!");
            let result = scheduler.wait_shutdown().await;
            log::info!("roller_monitor has been stopped with result: {:?}", result);
        }
        Err(err) => {
            eprintln!("roller_monitor start meet error: {err}");
        }
    }
}
