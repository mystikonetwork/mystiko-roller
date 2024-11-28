use anyhow::Error as AnyhowError;
use ethers_providers::ProviderError;
use log::ParseLevelError;
use mystiko_scheduler::SchedulerError;
use rusoto_core::region::ParseRegionError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitorRollupError {
    #[error("failed to parse config: {0}")]
    ParseConfigError(AnyhowError),
    #[error("chain={0} config not found error")]
    ChainConfigNotFoundError(u64),
    #[error("convert contract address={0} error")]
    ConvertContractAddressError(String),
    #[error("call contract func={0} meet error: {1}")]
    ContractCallError(String, String),
    #[error(transparent)]
    ParseLevelError(#[from] ParseLevelError),
    #[error(transparent)]
    SchedulerError(#[from] SchedulerError),
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
    #[error(transparent)]
    ParseRegionError(#[from] ParseRegionError),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    AnyhowError(#[from] AnyhowError),
    #[error(transparent)]
    RollerError(#[from] mystiko_roller::common::RollerError),
}
