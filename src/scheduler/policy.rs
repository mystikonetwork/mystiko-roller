use crate::common::RollerError;
use mystiko_dataloader::validator::rule::RuleCheckError;
use mystiko_dataloader::validator::rule::RuleValidatorError;
use mystiko_dataloader::validator::rule::SequenceCheckerError;
use mystiko_dataloader::validator::ValidatorError;
use mystiko_dataloader::DataLoaderError;
use mystiko_scheduler::{AbortPolicy, RetryPolicy};
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct RollerAbortPolicy {}

#[derive(Debug, TypedBuilder)]
pub struct RollerRetryPolicy {}

impl AbortPolicy<RollerError> for RollerAbortPolicy {
    fn should_abort(&self, error: &RollerError) -> bool {
        if let RollerError::DataLoaderError(DataLoaderError::ValidatorError(ValidatorError::AnyhowError(
            anyhow_error,
        ))) = error
        {
            if let Some(RuleValidatorError::RuleCheckError(RuleCheckError::SequenceCheckerError(
                SequenceCheckerError::CommitmentNotSequenceWithHandlerError(..),
            ))) = anyhow_error.downcast_ref::<RuleValidatorError>()
            {
                return true;
            }
        }
        false
    }
}

impl RetryPolicy<RollerError> for RollerRetryPolicy {
    fn should_retry(&self, _error: &RollerError) -> bool {
        false
    }
}
