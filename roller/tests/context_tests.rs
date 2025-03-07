mod mock;

use crate::mock::create_mock_env_config;
use mystiko_roller::common::RollerError;
use mystiko_roller::context::create_roller_context;

#[tokio::test]
async fn test_create_roller_context() {
    let env_config = create_mock_env_config();
    let c = create_roller_context(&env_config, None).await;
    assert!(c.is_ok());

    let mut env_config = create_mock_env_config();
    let c = create_roller_context(&env_config, Some(137)).await;
    assert_eq!(c.unwrap().config.chain_id, 137);

    env_config.config_path = "./tests/test_files/config/wrong_roller_config".to_string();
    let c = create_roller_context(&env_config, None).await;
    assert!(matches!(c.err().unwrap(), RollerError::AnyhowError(_)));

    env_config.config_path = "./tests/test_files/config/wrong_mystiko_config".to_string();
    let c = create_roller_context(&env_config, None).await;
    assert!(matches!(c.err().unwrap(), RollerError::AnyhowError(_)));

    env_config.config_path = "./tests/test_files/config/wrong_chain_id_config".to_string();
    let c = create_roller_context(&env_config, None).await;
    assert!(matches!(c.err().unwrap(), RollerError::ChainConfigNotFoundError(_)));

    env_config.config_path = "./tests/test_files/config/wrong_tx_manager_config".to_string();
    let c = create_roller_context(&env_config, None).await;
    assert!(matches!(c.err().unwrap(), RollerError::AnyhowError(_)));

    env_config.config_path = "./tests/test_files/home/config".to_string();
    let private_key = env_config.private_key.clone();
    env_config.private_key = "0x".to_string();
    let c = create_roller_context(&env_config, None).await;
    assert!(matches!(c.err().unwrap(), RollerError::WalletError(_)));

    env_config.private_key = private_key;
    env_config.config_path = "./tests/test_files/config/wrong_token_price_config".to_string();
    let c = create_roller_context(&env_config, None).await;
    assert!(matches!(c.err().unwrap(), RollerError::TokenPriceError(_)));
}
