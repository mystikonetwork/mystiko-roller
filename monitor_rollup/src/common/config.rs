use crate::{common::MonitorRollupResult, MonitorRollupError};
use mystiko_protos::common::v1::ConfigOptions;
use mystiko_scheduler::{RetryPolicy, StartOptions};
use mystiko_utils::config::{load_config, ConfigFile, ConfigLoadOptions};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use typed_builder::TypedBuilder;

const MONITOR_ROLLUP_PREFIX: &str = "MYSTIKO_MONITOR_ROLLUP";

#[derive(Debug, Clone, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct MonitorRollupConfig {
    #[serde(default = "default_logging_level")]
    #[builder(default = default_logging_level())]
    pub logging_level: String,

    #[serde(default = "default_extern_logging_level")]
    #[builder(default = default_extern_logging_level())]
    pub extern_logging_level: String,

    #[builder(default)]
    #[serde(default)]
    pub scheduler: SchedulerConfig,

    #[builder(default)]
    #[serde(default)]
    pub mystiko: ConfigOptions,
}

impl Default for MonitorRollupConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl MonitorRollupConfig {
    pub fn new(config_path: Option<PathBuf>) -> MonitorRollupResult<Self> {
        let config_file: Option<ConfigFile<PathBuf>> = config_path.map(|p| p.into());
        let options = if let Some(config_file) = config_file {
            ConfigLoadOptions::<PathBuf>::builder()
                .paths(vec![config_file])
                .env_prefix(MONITOR_ROLLUP_PREFIX)
                .build()
        } else {
            ConfigLoadOptions::<PathBuf>::builder()
                .env_prefix(MONITOR_ROLLUP_PREFIX)
                .build()
        };
        let cfg = load_config::<PathBuf, Self>(&options)?;
        Ok(cfg)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TypedBuilder)]
#[builder(field_defaults(default, setter(into)))]
pub struct SchedulerConfig {
    #[builder(default = default_scheduler_interval_ms())]
    #[serde(default = "default_scheduler_interval_ms")]
    pub interval_ms: u64,
    #[builder(default = default_scheduler_task_timeout_ms())]
    #[serde(default = "default_scheduler_task_timeout_ms")]
    pub task_timeout_ms: Option<u64>,
    #[builder(default)]
    #[serde(default)]
    pub no_retry_on_timeout: bool,
    #[builder(default)]
    #[serde(default)]
    pub max_retry_times: u32,
    #[builder(default = default_scheduler_status_server_bind_address())]
    #[serde(default = "default_scheduler_status_server_bind_address")]
    pub status_server_bind_address: String,
    #[builder(default = 21829)]
    #[builder(default = 21829)]
    pub status_server_port: u16,
}

#[derive(Debug)]
pub struct DefaultRetryPolicy;

impl From<SchedulerConfig> for StartOptions<MonitorRollupError> {
    fn from(config: SchedulerConfig) -> Self {
        StartOptions::<MonitorRollupError>::builder()
            .interval_ms(config.interval_ms)
            .no_retry_on_timeout(config.no_retry_on_timeout)
            .task_timeout_ms(config.task_timeout_ms)
            .max_retry_times(config.max_retry_times)
            .retry_policy(Arc::new(
                Box::new(DefaultRetryPolicy) as Box<dyn RetryPolicy<MonitorRollupError>>
            ))
            .build()
    }
}

impl RetryPolicy<MonitorRollupError> for DefaultRetryPolicy {
    fn should_retry(&self, error: &MonitorRollupError) -> bool {
        matches!(error, MonitorRollupError::ProviderError(_))
    }
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
#[builder(field_defaults(default, setter(into)))]
pub struct NotificationConfig {
    #[builder(default)]
    #[serde(default)]
    pub topic_arn: Option<String>,
    #[builder(default)]
    #[serde(default)]
    pub region: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct ChainMonitorConfig {
    pub max_rollup_delay_block: u64,
}

fn default_logging_level() -> String {
    "info".to_string()
}

fn default_extern_logging_level() -> String {
    "warn".to_string()
}

fn default_scheduler_interval_ms() -> u64 {
    120_000_u64
}

fn default_scheduler_task_timeout_ms() -> Option<u64> {
    Some(600_000_000_u64)
}

fn default_scheduler_status_server_bind_address() -> String {
    "0.0.0.0".to_string()
}
