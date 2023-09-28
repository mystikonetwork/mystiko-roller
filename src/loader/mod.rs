mod v1;

pub use v1::*;

use crate::common::RollerResult;
use async_trait::async_trait;

#[async_trait]
pub trait ChainDataLoader: Send + Sync {
    async fn load(&self) -> RollerResult<()>;
}
