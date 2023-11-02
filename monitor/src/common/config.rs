use crate::{common::RollerMonitorResult, RollerMonitorError};
use mystiko_protos::{common::v1::ConfigOptions, service::v1::ClientOptions};
use mystiko_scheduler::{RetryPolicy, StartOptions};
use mystiko_utils::config::{load_config, ConfigFile, ConfigLoadOptions};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use typed_builder::TypedBuilder;

const MONITOR_CONFIG_PREFIX: &str = "MYSTIKO_ROLLER_MONITOR";

#[derive(Debug, Clone, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RollerMonitorConfig {
    #[serde(default = "default_logging_level")]
    #[builder(default = default_logging_level())]
    pub logging_level: String,

    #[serde(default = "default_extern_logging_level")]
    #[builder(default = default_extern_logging_level())]
    pub extern_logging_level: String,

    #[builder(default)]
    #[serde(default)]
    pub chains: HashMap<u64, ChainMonitorConfig>,

    #[builder(default)]
    #[serde(default)]
    pub scheduler: SchedulerConfig,

    #[builder(default)]
    #[serde(default)]
    pub sequencer: ClientOptions,

    #[builder(default)]
    #[serde(default)]
    pub mystiko: ConfigOptions,

    #[builder(default)]
    #[serde(default)]
    pub notification: NotificationConfig,
}

impl Default for RollerMonitorConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl RollerMonitorConfig {
    pub fn new(config_path: Option<PathBuf>) -> RollerMonitorResult<Self> {
        let config_file: Option<ConfigFile<PathBuf>> = config_path.map(|p| p.into());
        let options = if let Some(config_file) = config_file {
            ConfigLoadOptions::<PathBuf>::builder()
                .paths(vec![config_file])
                .env_prefix(MONITOR_CONFIG_PREFIX)
                .build()
        } else {
            ConfigLoadOptions::<PathBuf>::builder()
                .env_prefix(MONITOR_CONFIG_PREFIX)
                .build()
        };
        let cfg = load_config::<PathBuf, Self>(&options)?;
        Ok(cfg)
    }

    pub fn get_max_rollup_delay_block(&self, chain_id: u64) -> u64 {
        match self.chains.get(&chain_id) {
            Some(c) => c.max_rollup_delay_block,
            None => get_default_max_delay_block(chain_id),
        }
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
    #[builder(default = 21828)]
    #[builder(default = 21828)]
    pub status_server_port: u16,
}

#[derive(Debug)]
pub struct DefaultRetryPolicy;

impl From<SchedulerConfig> for StartOptions<RollerMonitorError> {
    fn from(config: SchedulerConfig) -> Self {
        StartOptions::<RollerMonitorError>::builder()
            .interval_ms(config.interval_ms)
            .no_retry_on_timeout(config.no_retry_on_timeout)
            .task_timeout_ms(config.task_timeout_ms)
            .max_retry_times(config.max_retry_times)
            .retry_policy(Arc::new(
                Box::new(DefaultRetryPolicy) as Box<dyn RetryPolicy<RollerMonitorError>>
            ))
            .build()
    }
}

impl RetryPolicy<RollerMonitorError> for DefaultRetryPolicy {
    fn should_retry(&self, error: &RollerMonitorError) -> bool {
        matches!(
            error,
            RollerMonitorError::ProviderError(_)
                | RollerMonitorError::SequencerClientError(_)
                | RollerMonitorError::PushMessageError(_)
        )
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
    pub topic_arn_key: Option<String>,
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
    7_200_000_u64
}

fn default_scheduler_task_timeout_ms() -> Option<u64> {
    Some(60_000_u64)
}

fn default_scheduler_status_server_bind_address() -> String {
    "0.0.0.0".to_string()
}

fn get_default_max_delay_block(chain_id: u64) -> u64 {
    match chain_id {
        1 | 5 => 500,
        56 | 97 => 1875,
        137 | 80001 => 2500,
        8453 | 84531 => 2500,
        43113 => 1875,
        4002 => 3125,
        1287 => 292,
        _ => 2000,
    }
}
