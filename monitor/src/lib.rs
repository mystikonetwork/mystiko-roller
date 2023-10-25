mod common;
mod roller_monitor;

pub use common::*;
pub use roller_monitor::*;

use mystiko_config::MystikoConfig;
use mystiko_ethers::{ProviderPool, Providers};
use mystiko_notification::{Notification, SnsNotification};
use mystiko_scheduler::{Scheduler, SchedulerOptions, StartOptions};
use mystiko_utils::json::to_safe_json_string;
use rusoto_core::Region;
use std::fmt::Debug;
use std::path::PathBuf;
use std::{str::FromStr, sync::Arc};

pub async fn start_monitor(config_path: Option<String>) -> RollerMonitorResult<Scheduler<(), RollerMonitor>> {
    let config = Arc::new(RollerMonitorConfig::new(config_path.map(PathBuf::from))?);
    let mystiko_config = Arc::new(MystikoConfig::from_options(config.mystiko.clone()).await?);
    let providers = Arc::new(ProviderPool::new(mystiko_config.clone()));
    let notification = Arc::new(SnsNotification::from_region(
        Region::from_str(&config.notification.region).map_err(RollerMonitorError::ParseRegionError)?,
    ));
    start_monitor_with_config(config, mystiko_config, providers, notification).await
}

pub async fn start_monitor_with_config<M, N, P>(
    config: Arc<RollerMonitorConfig>,
    mystiko_config: Arc<MystikoConfig>,
    providers: Arc<P>,
    notification: Arc<N>,
) -> RollerMonitorResult<Scheduler<(), RollerMonitor<M, N, P>>>
where
    M: Clone + Send + Sync + 'static,
    MonitorAlert<M>: IntoMessage<M>,
    N: Notification<M> + 'static,
    N::Error: Debug + 'static,
    P: Providers + 'static,
{
    roller_monitor_trace_init(&config)?;
    log::info!(
        "starting roller_monitor with config: {}",
        to_safe_json_string(&config, false)?
    );
    let monitor = RollerMonitor::from_config(config.clone(), mystiko_config, providers, notification).await?;
    let scheduler = Scheduler::new(
        SchedulerOptions::<(), RollerMonitor<M, N, P>>::builder()
            .task(Arc::new(monitor))
            .status_server_bind_address(config.scheduler.status_server_bind_address.to_string())
            .status_server_port(config.scheduler.status_server_port)
            .build(),
    );
    let start_options = StartOptions::<RollerMonitorError>::from(config.scheduler.clone());
    scheduler.start((), start_options).await?;
    Ok(scheduler)
}
