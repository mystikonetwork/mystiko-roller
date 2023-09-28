use crate::common::{RollerError, RollerResult};
use dotenv::dotenv;
use std::env;
use std::ops::Add;
use std::path::PathBuf;
use std::str::FromStr;
use typed_builder::TypedBuilder;

const ROLLER_ENV_CONFIG_PREFIX: &str = "MYSTIKO_ROLLER";

const ENV_ROLLER_RUN_MOD: &str = "MYSTIKO_ROLLER_RUN_MOD";
const ENV_ROLLER_MEMORY_DB: &str = "MYSTIKO_ROLLER_MEMORY_DB";
const ENV_ROLLER_HOME_PATH: &str = "MYSTIKO_ROLLER_HOME_PATH";
const ENV_ROLLER_CONFIG_PATH: &str = "MYSTIKO_ROLLER_CONFIG_PATH";
const ENV_ROLLER_DATA_PATH: &str = "MYSTIKO_ROLLER_DATA_PATH";
const ENV_ROLLER_CIRCUITS_PATH: &str = "MYSTIKO_ROLLER_CIRCUITS_PATH";
const ENV_ROLLER_PRIVATE_KEY: &str = "MYSTIKO_ROLLER_PRIVATE_KEY";
const ENV_ROLLER_TOKEN_PRICE_API_KEY: &str = "MYSTIKO_ROLLER_TOKEN_PRICE_API_KEY";

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RollerRunMod {
    Mainnet,
    Testnet,
}

impl FromStr for RollerRunMod {
    type Err = RollerError;

    fn from_str(s: &str) -> RollerResult<Self> {
        match s {
            "mainnet" => Ok(RollerRunMod::Mainnet),
            "testnet" => Ok(RollerRunMod::Testnet),
            _ => Ok(RollerRunMod::Testnet),
        }
    }
}
impl RollerRunMod {
    pub fn as_str(&self) -> &str {
        match self {
            RollerRunMod::Mainnet => "mainnet",
            RollerRunMod::Testnet => "testnet",
        }
    }

    pub fn is_testnet(&self) -> bool {
        match self {
            RollerRunMod::Mainnet => false,
            RollerRunMod::Testnet => true,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct RollerEnvConfig {
    pub run_mod: RollerRunMod,
    pub memory_db: bool,
    pub config_path: String,
    pub config_env_prefix: String,
    pub data_file: String,
    pub circuits_path: String,
    pub private_key: String,
    pub token_price_api_key: String,
}

impl RollerEnvConfig {
    pub fn new() -> RollerResult<Self> {
        let run_mod = load_roller_run_mod()?;
        let memory_db = load_roller_memory_db();
        let home_path = load_roller_home_path();
        let config_path = load_roller_config_path(&home_path);
        let data_file = load_roller_data_file(&home_path);
        let circuits_path = load_roller_circuits_path(&home_path);
        let private_key = load_private_key()?;
        let token_price_api_key = load_token_price_api_key()?;
        let config = RollerEnvConfig::builder()
            .run_mod(run_mod)
            .memory_db(memory_db)
            .private_key(private_key)
            .token_price_api_key(token_price_api_key)
            .config_path(config_path)
            .config_env_prefix(ROLLER_ENV_CONFIG_PREFIX.to_string())
            .data_file(data_file)
            .circuits_path(circuits_path)
            .build();
        Ok(config)
    }

    pub fn roller_config_path(&self) -> PathBuf {
        PathBuf::from(&self.config_path)
    }

    pub fn roller_config_file(&self) -> Option<PathBuf> {
        let base_path = PathBuf::from(&self.config_path);
        let file_path = base_path.join("roller.json");
        if file_path.exists() {
            return Some(base_path.join("roller"));
        }
        None
    }

    pub fn mystiko_config_file(&self) -> Option<String> {
        let base_path = PathBuf::from(&self.config_path);
        let file_path = base_path.join("mystiko.json");
        if file_path.exists() {
            return Some(self.config_path.clone().add("/mystiko.json"));
        }
        None
    }
}

pub fn load_roller_run_mod() -> RollerResult<RollerRunMod> {
    dotenv().ok();
    match env::var(ENV_ROLLER_RUN_MOD) {
        Ok(value) => RollerRunMod::from_str(&value),
        Err(_) => Ok(RollerRunMod::Testnet),
    }
}

pub fn load_roller_memory_db() -> bool {
    dotenv().ok();
    match env::var(ENV_ROLLER_MEMORY_DB) {
        Ok(value) => value == "true",
        Err(_) => false,
    }
}

pub fn load_roller_home_path() -> String {
    dotenv().ok();
    match env::var(ENV_ROLLER_HOME_PATH) {
        Ok(value) => value,
        Err(_) => "/home/mystiko-miner/roller".to_string(),
    }
}

pub fn load_roller_config_path(home_path: &str) -> String {
    dotenv().ok();
    match env::var(ENV_ROLLER_CONFIG_PATH) {
        Ok(value) => value,
        Err(_) => home_path.to_string().add("/config"),
    }
}

pub fn load_roller_data_file(home_path: &str) -> String {
    dotenv().ok();
    match env::var(ENV_ROLLER_DATA_PATH) {
        Ok(value) => value,
        Err(_) => home_path.to_string().add("/data/roller.db"),
    }
}

pub fn load_roller_circuits_path(home_path: &str) -> String {
    dotenv().ok();
    match env::var(ENV_ROLLER_CIRCUITS_PATH) {
        Ok(value) => value,
        Err(_) => home_path.to_string().add("/circuits"),
    }
}

pub fn load_private_key() -> RollerResult<String> {
    dotenv().ok();
    match env::var(ENV_ROLLER_PRIVATE_KEY) {
        Ok(value) => Ok(value),
        Err(_) => Err(RollerError::RollerEnvPrivateKeyNotSetError),
    }
}

pub fn load_token_price_api_key() -> RollerResult<String> {
    dotenv().ok();
    match env::var(ENV_ROLLER_TOKEN_PRICE_API_KEY) {
        Ok(value) => Ok(value),
        Err(_) => Err(RollerError::RollerEnvTokenPriceApiKeyNotSetError),
    }
}
