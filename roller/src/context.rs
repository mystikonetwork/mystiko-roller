use crate::common::config::{
    create_mystiko_config, create_roller_config, create_token_price_config, create_tx_manager_config, RollerConfig,
};
use crate::common::{
    roller_trace_init, JsonProviderWrapper, RollerDataHandler, RollerEnvConfig, RollerError, RollerPriceMiddleware,
    RollerProviderPool, RollerProviders, RollerResult, RollerTransactionMiddleware,
};
use crate::handler::RollerDatabaseHandler;
use crate::scheduler::status::RollerStatusWrapper;
use ethers_signers::{LocalWallet, Signer};
use log::info;
use mystiko_config::MystikoConfig;
use mystiko_ethers::Providers;
use mystiko_ethers::{DefaultProviderFactory, Provider, ProviderFactory, ProviderOptions, ProvidersOptions};
use mystiko_server_utils::token_price::TokenPrice;
use mystiko_server_utils::tx_manager::TxManagerBuilder;
use mystiko_types::TransactionType;
use std::str::FromStr;
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct RollerContext {
    pub env_config: Arc<RollerEnvConfig>,
    pub config: Arc<RollerConfig>,
    pub mystiko_config: Arc<MystikoConfig>,
    pub handler: Arc<Box<RollerDataHandler>>,
    pub provider: Arc<Provider>,
    pub providers: Arc<RollerProviders>,
    pub tx: Arc<RollerTransactionMiddleware>,
    pub price: Arc<RollerPriceMiddleware>,
    pub status: Arc<RollerStatusWrapper>,
}

pub async fn create_roller_context(env_config: &RollerEnvConfig, chain_id: Option<u64>) -> RollerResult<RollerContext> {
    let mut config = create_roller_config(env_config)?;
    if let Some(chain) = chain_id {
        config.chain_id = chain;
    }
    let chain_id = config.chain_id;

    roller_trace_init(&config)?;
    info!("start roller with chain id={:?}", chain_id);
    let mystiko_config = Arc::new(create_mystiko_config(env_config, &config.mystiko).await?);
    let chain_cfg = mystiko_config
        .find_chain(chain_id)
        .ok_or(RollerError::ChainConfigNotFoundError(chain_id))?;
    let tx_type = match chain_cfg.transaction_type() {
        TransactionType::Legacy => false,
        TransactionType::Eip1559 => true,
        TransactionType::Eip2930 => false,
    };

    let handler = RollerDatabaseHandler::new(config.memory_db, env_config, mystiko_config.clone()).await?;
    handler.migrate().await?;
    handler.initialize().await?;
    let handler = Arc::new(Box::new(handler) as Box<RollerDataHandler>);
    let providers: RollerProviderPool = mystiko_config.clone().into();
    let provider = if let Some(signer_provider) = config.signer_provider.clone() {
        info!("use signer provider: {}", signer_provider);
        let options = ProvidersOptions::Failover(vec![ProviderOptions::builder().url(signer_provider.clone()).build()]);
        Arc::new(DefaultProviderFactory::new().create_provider(options).await?)
    } else {
        providers.get_provider(chain_id).await?
    };
    let tx_manager_cfg = create_tx_manager_config(env_config)?;
    let local_wallet = LocalWallet::from_str(&env_config.private_key)?.with_chain_id(chain_id);
    info!("local wallet address is {:?}", local_wallet.address());
    let builder = TxManagerBuilder::builder()
        .chain_id(chain_id)
        .config(tx_manager_cfg)
        .wallet(local_wallet)
        .build();
    let tx_manager = builder.build::<JsonProviderWrapper>(Some(tx_type), &provider).await?;
    let tx_manager = Arc::new(tx_manager) as Arc<RollerTransactionMiddleware>;
    info!("chain support 1559 {:?}", tx_manager.tx_eip1559());
    let is_testnet = config.mystiko.is_testnet.unwrap_or_default();
    let token_price_cfg = create_token_price_config(is_testnet, env_config)?;
    let token_price = TokenPrice::new(&token_price_cfg, &env_config.token_price_api_key)?;
    let token_price = Arc::new(token_price) as Arc<RollerPriceMiddleware>;
    let status = Arc::new(RollerStatusWrapper::new().await);
    Ok(RollerContext::builder()
        .env_config(Arc::new(env_config.clone()))
        .config(Arc::new(config))
        .mystiko_config(mystiko_config)
        .handler(handler)
        .provider(provider)
        .providers(Arc::new(Box::new(providers) as RollerProviders))
        .tx(tx_manager)
        .price(token_price)
        .status(status)
        .build())
}
