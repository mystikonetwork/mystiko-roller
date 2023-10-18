use crate::common::RollerError;
use mystiko_scheduler::{AbortPolicy, RetryPolicy};
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct RollerAbortPolicy {}

#[derive(Debug, TypedBuilder)]
pub struct RollerRetryPolicy {}

impl AbortPolicy<RollerError> for RollerAbortPolicy {
    fn should_abort(&self, _error: &RollerError) -> bool {
        false
    }
}

impl RetryPolicy<RollerError> for RollerRetryPolicy {
    fn should_retry(&self, _error: &RollerError) -> bool {
        false
    }
}
