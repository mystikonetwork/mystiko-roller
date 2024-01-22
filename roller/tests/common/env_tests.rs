use mystiko_roller::common::{
    load_private_key, load_roller_circuits_path, load_roller_config_path, load_roller_data_file, load_roller_home_path,
    load_token_price_api_key,
};
use std::env;

#[tokio::test]
async fn test_load_roller_home_path() {
    let home_path = load_roller_home_path();
    assert_eq!(home_path, "/home/mystiko-miner/roller");

    env::set_var("MYSTIKO_ROLLER.HOME_PATH", "/home");
    let home_path = load_roller_home_path();
    assert_eq!(home_path, "/home");

    env::remove_var("MYSTIKO_ROLLER.CONFIG_PATH");
    let config_path = load_roller_config_path(home_path.as_str());
    assert_eq!(config_path, "/home/config");

    env::remove_var("MYSTIKO_ROLLER.CIRCUITS_PATH");
    let circuits_path = load_roller_circuits_path(home_path.as_str());
    assert_eq!(circuits_path, "/home/circuits");

    env::set_var("MYSTIKO_ROLLER.CONFIG_PATH", "./tests/test_files/config/base");
    let config_path = load_roller_config_path(home_path.as_str());
    assert_eq!(config_path, "./tests/test_files/config/base");

    env::set_var("MYSTIKO_ROLLER.DATA_PATH", "./tests/test_files/db");
    let db_path = load_roller_data_file(home_path.as_str());
    assert_eq!(db_path, "./tests/test_files/db");

    env::set_var("MYSTIKO_ROLLER.CIRCUITS_PATH", "./tests/test_files/circuits");
    let circuits_path = load_roller_circuits_path(home_path.as_str());
    assert_eq!(circuits_path, "./tests/test_files/circuits");

    env::remove_var("MYSTIKO_ROLLER.CONFIG_PATH");
    env::remove_var("MYSTIKO_ROLLER.DATA_PATH");
    env::remove_var("MYSTIKO_ROLLER.CIRCUITS_PATH");
}

#[tokio::test]
async fn test_load_roller_private_key() {
    let private_key = load_private_key();
    assert!(private_key.is_err());
    env::set_var(
        "MYSTIKO_ROLLER.PRIVATE_KEY",
        "0x2f0ddd32231ec7dadcef459447c73fae18b9b3e3d0e0acf00e999ca5ffb8efec",
    );
    let private_key = load_private_key().unwrap();
    assert_eq!(
        private_key,
        "0x2f0ddd32231ec7dadcef459447c73fae18b9b3e3d0e0acf00e999ca5ffb8efec"
    );
    env::remove_var("MYSTIKO_ROLLER.PRIVATE_KEY");
}

#[tokio::test]
async fn test_load_coin_market_api_key() {
    let coin_market_api_key = load_token_price_api_key();
    assert!(coin_market_api_key.is_err());
    env::set_var("MYSTIKO_ROLLER.TOKEN_PRICE_API_KEY", "coin_market_api_key");
    let coin_market_api_key = load_token_price_api_key().unwrap();
    assert_eq!(coin_market_api_key, "coin_market_api_key");
    env::remove_var("MYSTIKO_ROLLER.TOKEN_PRICE_API_KEY");
}
