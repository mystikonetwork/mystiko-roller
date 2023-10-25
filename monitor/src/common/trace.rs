use crate::{common::types::RollerMonitorResult, RollerMonitorConfig};
use std::str::FromStr;

pub fn roller_monitor_trace_init(config: &RollerMonitorConfig) -> RollerMonitorResult<()> {
    let logging_level = log::LevelFilter::from_str(&config.logging_level)?;
    let _ = env_logger::builder()
        .filter_module("", log::LevelFilter::from_str(&config.extern_logging_level)?)
        .filter_module("mystiko_roller_monitor", logging_level)
        .filter_module("mystiko_scheduler", logging_level)
        .try_init();
    Ok(())
}
