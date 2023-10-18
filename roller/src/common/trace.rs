use crate::common::config::RollerConfig;
use crate::common::RollerResult;
use std::str::FromStr;

pub fn roller_trace_init(config: &RollerConfig) -> RollerResult<()> {
    let _ = env_logger::builder()
        .filter_module("", log::LevelFilter::from_str(&config.extern_logging_level)?)
        .filter_module("mystiko_scheduler", log::LevelFilter::from_str(&config.log_level)?)
        .filter_module("mystiko_dataloader", log::LevelFilter::from_str(&config.log_level)?)
        .filter_module("mystiko_roller", log::LevelFilter::from_str(&config.log_level)?)
        .try_init();
    Ok(())
}
