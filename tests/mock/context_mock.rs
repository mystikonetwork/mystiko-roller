use crate::mock::handler_mock::MockRollerHandler;
use crate::mock::provider_mock::create_mock_providers;
use crate::mock::token_price_mock::MockRollerTokenPrice;
use crate::mock::tx_manager_mock::MockRollerTxManager;
use crate::mock::{create_mock_env_config, create_mock_mystiko_config, create_mock_roller_config};
use ethers_providers::MockProvider;
use log::LevelFilter;
use mystiko_dataloader::data::LiteData;
use mystiko_ethers::Providers;
use mystiko_roller::common::RollerProviders;
use mystiko_roller::context::RollerContext;
use mystiko_roller::scheduler::status::RollerStatusWrapper;
use std::sync::Arc;

pub async fn create_mock_context(roller_config_path: Option<&str>) -> (RollerContext, MockProvider) {
    let _ = env_logger::builder()
        .filter_module("mystiko_roller", LevelFilter::Debug)
        .try_init();

    let mut env_config = create_mock_env_config();
    if let Some(path) = roller_config_path {
        env_config.config_path = path.to_string();
    }
    let roller_config = create_mock_roller_config(&env_config).await;
    let mystiko_config = Arc::new(create_mock_mystiko_config().await);
    let mock_handler = MockRollerHandler::<LiteData>::new();
    let handler = Arc::new(Box::new(mock_handler) as Box<_>);
    let price = MockRollerTokenPrice::new();
    let tx = MockRollerTxManager::new();
    let (mock_provider, providers) = create_mock_providers();
    let provider = providers.get_provider(roller_config.chain_id).await.unwrap();
    let status = Arc::new(RollerStatusWrapper::new().await);
    let context = RollerContext::builder()
        .env_config(Arc::new(env_config))
        .config(Arc::new(roller_config))
        .mystiko_config(mystiko_config)
        .handler(handler)
        .provider(provider)
        .providers(Arc::new(Box::new(providers) as RollerProviders))
        .tx(Arc::new(tx))
        .price(Arc::new(price))
        .status(status)
        .build();
    (context, mock_provider)
}
