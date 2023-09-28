use crate::common::{RollerEnvConfig, RollerResult};
use log::info;
use mystiko_config::MystikoConfig;
use mystiko_protos::common::v1::ConfigOptions;
use mystiko_protos::loader::v1::{
    FetcherType, LoaderConfig, RuleValidatorCheckerType, RuleValidatorConfig, ValidatorConfig, ValidatorType,
};
use mystiko_server_utils::token_price::config::TokenPriceConfig;
use mystiko_server_utils::tx_manager::config::TxManagerConfig;
use mystiko_utils::config::{load_config, ConfigFile, ConfigLoadOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RollerConfig {
    #[serde(default = "default_logging_level")]
    #[builder(default = default_logging_level())]
    pub log_level: String,

    #[serde(default = "default_logging_level")]
    #[builder(default = default_logging_level())]
    pub extern_logging_level: String,

    #[serde(default = "default_check_chain_id")]
    #[builder(default = default_check_chain_id())]
    pub chain_id: u64,

    #[builder(default)]
    #[serde(default)]
    pub scheduler: RollerSchedulerConfig,

    #[builder(default)]
    #[serde(default)]
    pub loader: RollerLoaderConfig,

    #[builder(default)]
    #[serde(default)]
    pub rollup: RollerRollupConfig,
}

impl Default for RollerConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl RollerConfig {
    pub fn merkle_tree_height(&self) -> u32 {
        self.rollup.merkle_tree_height
    }

    pub fn max_gas_price(&self) -> u64 {
        self.rollup
            .chains
            .get(&self.chain_id)
            .map_or_else(default_max_gas_price, |c| c.max_gas_price)
    }

    pub fn force_rollup_block_count(&self) -> u64 {
        self.rollup
            .chains
            .get(&self.chain_id)
            .map_or_else(default_force_rollup_block_count, |c| c.force_rollup_block_count)
    }

    pub fn rollup_gas_cost(&self, rollup_size: usize) -> u64 {
        match self.rollup.chains.get(&self.chain_id) {
            None => default_rollup_gas_cost_by_rollup_size(rollup_size),
            Some(c) => *c
                .gas_cost
                .get(&rollup_size)
                .unwrap_or(&default_rollup_gas_cost_by_rollup_size(rollup_size)),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RollerSchedulerConfig {
    #[serde(default = "default_schedule_interval_ms")]
    #[builder(default = default_schedule_interval_ms())]
    pub schedule_interval_ms: u64,

    #[serde(default = "default_status_server_port")]
    #[builder(default = default_status_server_port())]
    pub status_server_port: u16,
}

impl Default for RollerSchedulerConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RollerLoaderConfig {
    #[serde(default)]
    #[builder(default)]
    pub config: LoaderConfig,
}

impl RollerLoaderConfig {
    pub fn set_default_roller_fetcher(&mut self) {
        self.config.fetchers.insert(0, FetcherType::Packer as i32);
        self.config.fetchers.insert(1, FetcherType::Indexer as i32);
        self.config.fetchers.insert(2, FetcherType::Etherscan as i32);
        self.config.fetchers.insert(3, FetcherType::Provider as i32);
    }

    pub fn set_default_roller_validator(&mut self) {
        self.config.validators.insert(0, ValidatorType::Rule as i32);
        let mut checkers = HashMap::new();
        checkers.insert(0, RuleValidatorCheckerType::Integrity as i32);
        checkers.insert(1, RuleValidatorCheckerType::Sequence as i32);
        checkers.insert(2, RuleValidatorCheckerType::Counter as i32);
        self.config.validator_config = Some(
            ValidatorConfig::builder()
                .rule(RuleValidatorConfig::builder().checkers(checkers).build())
                .build(),
        );
    }
}

impl Default for RollerLoaderConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RollerRollupConfig {
    #[serde(default = "default_merkle_tree_height")]
    #[builder(default = default_merkle_tree_height())]
    pub merkle_tree_height: u32,

    #[serde(default)]
    #[builder(default)]
    pub chains: HashMap<u64, RollerRollupChainConfig>,
}

impl Default for RollerRollupConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RollerRollupChainConfig {
    #[serde(default = "default_max_gas_price")]
    #[builder(default = default_max_gas_price())]
    pub max_gas_price: u64,
    #[serde(default = "default_force_rollup_block_count")]
    #[builder(default = default_force_rollup_block_count())]
    pub force_rollup_block_count: u64,
    #[serde(default = "default_rollup_gas_cost")]
    #[builder(default = default_rollup_gas_cost())]
    pub gas_cost: HashMap<usize, u64>,
}

impl Default for RollerRollupChainConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl RollerConfig {
    pub fn new(env_config: &RollerEnvConfig) -> RollerResult<Self> {
        let config_file = env_config.roller_config_file();
        let config_file: Option<ConfigFile<PathBuf>> = config_file.map(|p| p.into());
        let options = if let Some(file) = config_file {
            ConfigLoadOptions::<PathBuf>::builder()
                .paths(file)
                .env_prefix(env_config.config_env_prefix.clone())
                .build()
        } else {
            ConfigLoadOptions::<PathBuf>::builder()
                .env_prefix(env_config.config_env_prefix.clone())
                .build()
        };

        let mut roller_config = load_config::<PathBuf, Self>(&options)?;
        if roller_config.loader.config.fetchers.is_empty() {
            roller_config.loader.set_default_roller_fetcher();
        }
        if roller_config.loader.config.validators.is_empty() {
            roller_config.loader.set_default_roller_validator();
        }
        Ok(roller_config)
    }
}

pub fn create_roller_config(env_config: &RollerEnvConfig) -> RollerResult<RollerConfig> {
    RollerConfig::new(env_config)
}

pub async fn create_mystiko_config(env_config: &RollerEnvConfig) -> RollerResult<MystikoConfig> {
    let config_file = env_config.mystiko_config_file();
    match config_file {
        Some(c) => {
            info!("load mystiko configure from local file");
            MystikoConfig::from_json_file(&c).await.map_err(|e| e.into())
        }
        None => {
            info!("load mystiko configure from remote url");
            let remote_options = ConfigOptions::builder()
                .is_testnet(env_config.run_mod.is_testnet())
                .build();
            MystikoConfig::from_remote(&remote_options).await.map_err(|e| e.into())
        }
    }
}

pub fn create_token_price_config(env_config: &RollerEnvConfig) -> RollerResult<TokenPriceConfig> {
    TokenPriceConfig::new(env_config.run_mod.as_str(), Some(env_config.roller_config_path())).map_err(|e| e.into())
}

pub fn create_tx_manager_config(env_config: &RollerEnvConfig) -> RollerResult<TxManagerConfig> {
    TxManagerConfig::new(Some(env_config.roller_config_path())).map_err(|e| e.into())
}

fn default_logging_level() -> String {
    "info".to_string()
}

fn default_check_chain_id() -> u64 {
    1
}

fn default_schedule_interval_ms() -> u64 {
    120_000
}

fn default_status_server_port() -> u16 {
    21818
}

fn default_merkle_tree_height() -> u32 {
    20
}

fn default_max_gas_price() -> u64 {
    100000000000_u64
}

fn default_force_rollup_block_count() -> u64 {
    100
}

fn default_rollup_gas_cost() -> HashMap<usize, u64> {
    let mut cost = HashMap::new();
    cost.insert(1, default_rollup_gas_cost_by_rollup_size(1));
    cost.insert(2, default_rollup_gas_cost_by_rollup_size(2));
    cost.insert(4, default_rollup_gas_cost_by_rollup_size(4));
    cost.insert(8, default_rollup_gas_cost_by_rollup_size(8));
    cost.insert(16, default_rollup_gas_cost_by_rollup_size(16));
    cost
}

fn default_rollup_gas_cost_by_rollup_size(rollup_size: usize) -> u64 {
    match rollup_size {
        1 => 331000_u64,
        2 => 336000_u64,
        4 => 340000_u64,
        8 => 360000_u64,
        16 => 410000_u64,
        _ => 480000_u64,
    }
}
