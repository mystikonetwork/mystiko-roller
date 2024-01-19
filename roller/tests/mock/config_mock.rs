use mystiko_config::MystikoConfig;
use mystiko_roller::common::config::{create_mystiko_config, RollerConfig};
use mystiko_roller::common::{RollerEnvConfig, RollerRunMod};

pub fn create_mock_env_config() -> RollerEnvConfig {
    RollerEnvConfig::builder()
        .run_mod(RollerRunMod::Testnet)
        .memory_db(true)
        .config_path("./tests/test_files/home/config".to_string())
        .config_is_staging(false)
        .config_env_prefix("ROLLER_TEST".to_string())
        .data_file("".to_string())
        .circuits_path("./tests/test_files/home/circuits".to_string())
        .private_key("0xd344aefc75ff1df9645054aeddfa688c543b81d115450dfe498a8a20927dd236".to_string())
        .token_price_api_key("token_price_api_key".to_string())
        .build()
}

pub async fn create_mock_roller_config(env_config: &RollerEnvConfig) -> RollerConfig {
    RollerConfig::new(env_config).unwrap()
}

pub async fn create_mock_mystiko_config() -> MystikoConfig {
    let env_config = create_mock_env_config();
    create_mystiko_config(&env_config).await.unwrap()
}
