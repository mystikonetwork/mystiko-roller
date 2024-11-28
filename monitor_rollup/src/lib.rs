mod common;
mod monitor;

pub use common::*;
pub use monitor::*;

use mystiko_config::MystikoConfig;
use mystiko_ethers::{ProviderPool, Providers};
use mystiko_scheduler::{Scheduler, SchedulerOptions, StartOptions};
use mystiko_utils::json::to_safe_json_string;
use std::path::PathBuf;
use std::sync::Arc;

pub async fn start_monitor_rollup(config_path: Option<String>) -> MonitorRollupResult<Scheduler<(), MonitorRollup>> {
    let config = Arc::new(MonitorRollupConfig::new(config_path.map(PathBuf::from))?);
    let mystiko_config = Arc::new(MystikoConfig::from_options(config.mystiko.clone()).await?);
    let providers = Arc::new(ProviderPool::new(mystiko_config.clone()));
    start_monitor_with_config(config, mystiko_config, providers).await
}

pub async fn start_monitor_with_config<P>(
    config: Arc<MonitorRollupConfig>,
    mystiko_config: Arc<MystikoConfig>,
    providers: Arc<P>,
) -> MonitorRollupResult<Scheduler<(), MonitorRollup<P>>>
where
    P: Providers + 'static,
{
    monitor_rollup_trace_init(&config)?;
    log::info!(
        "starting monitor_rollup with config: {}",
        to_safe_json_string(&config, false)?
    );
    let monitor = MonitorRollup::from_config(mystiko_config, providers).await?;
    let scheduler = Scheduler::new(
        SchedulerOptions::<(), MonitorRollup<P>>::builder()
            .task(Arc::new(monitor))
            .status_server_bind_address(config.scheduler.status_server_bind_address.to_string())
            .status_server_port(config.scheduler.status_server_port)
            .build(),
    );
    let start_options = StartOptions::<MonitorRollupError>::from(config.scheduler.clone());
    scheduler.start((), start_options).await?;
    Ok(scheduler)
}
