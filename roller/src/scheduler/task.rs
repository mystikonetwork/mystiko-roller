use crate::common::{RollerError, RollerResult};
use crate::context::RollerContext;
use crate::loader::ChainDataLoader;
use crate::loader::RollerChainDataLoader;
use crate::roller::ChainRoller;
use crate::scheduler::status::RollerStatusAction;
use async_trait::async_trait;
use log::warn;
use mystiko_scheduler::SchedulerTask;
use std::sync::Arc;

pub struct RollerTask {
    context: Arc<RollerContext>,
    loader: Arc<dyn ChainDataLoader>,
    roller: ChainRoller,
}

#[derive(Debug)]
pub struct RollerRunParams {}

#[async_trait]
impl SchedulerTask<Option<RollerRunParams>> for RollerTask {
    type Error = RollerError;

    async fn run(&self, args: &Option<RollerRunParams>) -> anyhow::Result<(), Self::Error> {
        self.run(args).await
    }
}

impl RollerTask {
    pub async fn new(context: Arc<RollerContext>) -> RollerResult<RollerTask> {
        let loader = Arc::new(RollerChainDataLoader::from_config(context.clone()).await?);
        let roller = ChainRoller::new(context.clone()).await?;
        let c = RollerTask {
            context,
            loader,
            roller,
        };
        Ok(c)
    }

    pub async fn run(&self, args: &Option<RollerRunParams>) -> RollerResult<()> {
        self.load().await?;
        self.rollup(args).await
    }

    pub async fn load(&self) -> RollerResult<()> {
        self.context.status.set_action(RollerStatusAction::Loading).await;
        let result = self.loader.load().await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("load failed: {:?}", e);
                self.context.status.set_action(RollerStatusAction::Idle).await;
                Err(e)
            }
        }
    }

    pub async fn rollup(&self, _args: &Option<RollerRunParams>) -> RollerResult<()> {
        self.context.status.set_action(RollerStatusAction::Rollup).await;
        let result = self.roller.run().await;
        match result {
            Ok(_) => {
                self.context.status.set_action(RollerStatusAction::Idle).await;
                Ok(())
            }
            Err(e) => {
                warn!("load failed: {:?}", e);
                self.context.status.set_action(RollerStatusAction::Idle).await;
                Err(e)
            }
        }
    }
}
