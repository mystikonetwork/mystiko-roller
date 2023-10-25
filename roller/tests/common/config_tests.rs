use crate::mock::create_mock_env_config;
use mystiko_protos::loader::v1::{FetcherConfig, ProviderFetcherChainConfig, ProviderFetcherConfig};
use mystiko_roller::common::config::{create_roller_config, RollerRollupChainConfig};
use std::collections::HashMap;

#[tokio::test]
async fn test_default_roller_config() {
    let env_config = create_mock_env_config();
    let mut roller_config = create_roller_config(&env_config).unwrap();
    let mut chains1 = HashMap::new();
    chains1.insert(1, ProviderFetcherChainConfig::builder().delay_num_blocks(10).build());
    roller_config.loader.config.fetcher_config = Some(
        FetcherConfig::builder()
            .provider(ProviderFetcherConfig::builder().chains(chains1).build())
            .build(),
    );
    roller_config.rollup.chains.insert(
        1,
        RollerRollupChainConfig::builder()
            .max_gas_price(100_u64)
            .force_rollup_block_count(10_u64)
            .build(),
    );
    println!("roller_config: {}", serde_json::to_string(&roller_config).unwrap());
}

#[tokio::test]
async fn test_default_roller_env_config() {
    let env_config = create_mock_env_config();
    let roller_config = create_roller_config(&env_config).unwrap();
    assert_eq!(roller_config.loader.config.fetchers.len(), 3);
    assert!(roller_config
        .loader
        .config
        .fetcher_config
        .as_ref()
        .unwrap()
        .provider
        .as_ref()
        .unwrap()
        .chains
        .get(&1)
        .is_some());

    std::env::set_var(
        "ROLLER_TEST_CONFIG.LOADER.CONFIG.FETCHER_CONFIG.ETHERSCAN.CHAINS.5.API_KEY",
        "ABD",
    );
    let mut env_config = create_mock_env_config();
    env_config.config_env_prefix = "ROLLER_TEST_CONFIG".to_string();
    let roller_config = create_roller_config(&env_config).unwrap();
    assert_eq!(roller_config.loader.config.fetchers.len(), 3);
    assert_eq!(
        roller_config
            .loader
            .config
            .fetcher_config
            .as_ref()
            .unwrap()
            .etherscan
            .as_ref()
            .unwrap()
            .chains
            .get(&5)
            .unwrap()
            .api_key,
        Some("ABD".to_string())
    );
    std::env::remove_var("ROLLER_TEST_CONFIG.LOADER.CONFIG.FETCHER_CONFIG.ETHERSCAN.CHAINS.5.API_KEY");
}
