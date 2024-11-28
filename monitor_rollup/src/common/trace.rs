use crate::{common::types::MonitorRollupResult, MonitorRollupConfig};
use std::str::FromStr;

pub fn monitor_rollup_trace_init(config: &MonitorRollupConfig) -> MonitorRollupResult<()> {
    let logging_level = log::LevelFilter::from_str(&config.logging_level)?;
    let _ = env_logger::builder()
        .filter_module("", log::LevelFilter::from_str(&config.extern_logging_level)?)
        .filter_module("mystiko_monitor_rollup", logging_level)
        .filter_module("mystiko_roller", logging_level)
        .filter_module("mystiko_scheduler", logging_level)
        .try_init();
    Ok(())
}
