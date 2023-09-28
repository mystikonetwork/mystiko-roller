use anyhow::Error as AnyhowError;
use ethers_providers::ProviderError;
use ethers_signers::WalletError;
use log::ParseLevelError as LogError;
use mystiko_crypto::error::MerkleTreeError;
use mystiko_dataloader::handler::HandlerError;
use mystiko_dataloader::DataLoaderError;
use mystiko_protocol::error::ProtocolError;
use mystiko_scheduler::SchedulerError;
use mystiko_server_utils::token_price::PriceMiddlewareError;
use mystiko_server_utils::tx_manager::TransactionMiddlewareError;
use mystiko_storage::StorageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RollerError {
    #[error("chain: {0} config not found error")]
    ChainConfigNotFoundError(u64),
    #[error("pool contract: {0} config not found error")]
    PoolContractConfigNotFoundError(String),
    #[error("convert contract address error: {0}")]
    ConvertContractAddressError(String),
    #[error("roller env private key not set error")]
    RollerEnvPrivateKeyNotSetError,
    #[error("roller env token price api key not set error")]
    RollerEnvTokenPriceApiKeyNotSetError,
    #[error("rollup size: {0} error")]
    RollupSizeError(usize),
    #[error("current gas price too high: {0}")]
    CurrentGasPriceTooHighError(String),
    #[error("circuits type: {0} not found error")]
    CircuitNotFoundError(i32),
    #[error("invalid rollup transaction call data")]
    InvalidTransactionCallDataError,
    #[error("commitment rollup fee is none")]
    CommitmentRollupFeeError,
    #[error("roller internal error: {0}")]
    RollerInternalError(String),
    #[error(transparent)]
    LogError(#[from] LogError),
    #[error(transparent)]
    DatabaseError(#[from] StorageError),
    #[error(transparent)]
    WalletError(#[from] WalletError),
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
    #[error(transparent)]
    DataLoaderError(#[from] DataLoaderError),
    #[error(transparent)]
    HandlerError(#[from] HandlerError),
    #[error(transparent)]
    TokenPriceError(#[from] PriceMiddlewareError),
    #[error(transparent)]
    TxManagerError(#[from] TransactionMiddlewareError),
    #[error(transparent)]
    MerkleTreeError(#[from] MerkleTreeError),
    #[error(transparent)]
    ProtocolError(#[from] ProtocolError),
    #[error(transparent)]
    SchedulerError(#[from] SchedulerError),
    #[error(transparent)]
    AnyhowError(#[from] AnyhowError),
}
