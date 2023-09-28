use crate::mock::create_mock_env_config;
use mystiko_dataloader::validator::rule::SequenceCheckerError;
use mystiko_dataloader::validator::rule::{RuleCheckError, RuleValidatorError};
use mystiko_dataloader::validator::ValidatorError;
use mystiko_dataloader::DataLoaderError;
use mystiko_roller::common::RollerError;
use mystiko_roller::context::create_roller_context;
use mystiko_roller::scheduler::policy::{RollerAbortPolicy, RollerRetryPolicy};
use mystiko_roller::scheduler::schedule::{run, RollerScheduler};
use mystiko_scheduler::{AbortPolicy, RetryPolicy};
use std::sync::Arc;

#[tokio::test]
async fn test_run() {
    let result = run().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_scheduler_start() {
    let env_config = create_mock_env_config();
    let context = Arc::new(create_roller_context(&env_config).await.unwrap());
    let scheduler = RollerScheduler::new(context).await.unwrap();
    let result = scheduler.start().await;
    drop(scheduler);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_scheduler_wait_shutdown() {
    let env_config = create_mock_env_config();
    let context = Arc::new(create_roller_context(&env_config).await.unwrap());
    let scheduler = RollerScheduler::new(context).await.unwrap();
    let result = scheduler.wait_shutdown().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_scheduler_retry_policy() {
    let roller_policy = RollerRetryPolicy::builder().build();
    let result = roller_policy.should_retry(&RollerError::DataLoaderError(DataLoaderError::LoaderNoContractsError));
    assert!(!result);
}

#[tokio::test]
async fn test_scheduler_abort_policy() {
    let roller_abort = RollerAbortPolicy::builder().build();
    let result = roller_abort.should_abort(&RollerError::DataLoaderError(DataLoaderError::ValidatorError(
        ValidatorError::AnyhowError(anyhow::anyhow!("test")),
    )));
    assert!(!result);

    let result = roller_abort.should_abort(&RollerError::DataLoaderError(DataLoaderError::ValidatorError(
        ValidatorError::AnyhowError(
            RuleValidatorError::RuleCheckError(RuleCheckError::SequenceCheckerError(
                SequenceCheckerError::CommitmentNotSequenceWithHandlerError(2, 1, 1),
            ))
            .into(),
        ),
    )));
    assert!(result);
}
