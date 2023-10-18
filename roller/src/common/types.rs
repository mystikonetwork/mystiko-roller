use crate::common::RollerError;
use mystiko_dataloader::data::{ChainData, LiteData};
use mystiko_dataloader::fetcher::DataFetcher;
use mystiko_dataloader::handler::{DataHandler, DatabaseHandler};
use mystiko_dataloader::loader::{ChainDataLoader, LoaderConfigOptions};
use mystiko_dataloader::validator::DataValidator;
use mystiko_ethers::ChainConfigProvidersOptions;
use mystiko_ethers::{JsonRpcClientWrapper, ProviderWrapper};
use mystiko_ethers::{ProviderPool, Providers};
use mystiko_server_utils::token_price::PriceMiddleware;
use mystiko_server_utils::tx_manager::TransactionMiddleware;

pub type RollerResult<T> = anyhow::Result<T, RollerError>;

pub type JsonProviderWrapper = ProviderWrapper<Box<dyn JsonRpcClientWrapper>>;

pub type RollerPriceMiddleware = dyn PriceMiddleware;

pub type RollerTransactionMiddleware = dyn TransactionMiddleware<JsonProviderWrapper>;

pub type RollerProviderPool = ProviderPool<ChainConfigProvidersOptions>;

pub type RollerProviders = Box<dyn Providers>;

pub type RollerChainData = ChainData<LiteData>;

pub type RollerDataHandler = dyn DataHandler<LiteData>;

pub type LiteDataDatabaseHandler<F, S> = DatabaseHandler<LiteData, F, S>;

pub type LiteDataChainDataLoader =
    ChainDataLoader<LiteData, Box<RollerDataHandler>, Box<dyn DataFetcher<LiteData>>, Box<dyn DataValidator<LiteData>>>;

pub type LiteDataLoaderConfigOptions = LoaderConfigOptions<LiteData, Box<RollerDataHandler>>;
