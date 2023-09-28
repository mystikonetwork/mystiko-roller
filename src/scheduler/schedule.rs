use crate::common::{RollerEnvConfig, RollerError, RollerResult};
use crate::context::{create_roller_context, RollerContext};
use crate::scheduler::policy::{RollerAbortPolicy, RollerRetryPolicy};
use crate::scheduler::status::RollerStatusGetter;
use crate::scheduler::task::{RollerRunParams, RollerTask};
use log::info;
use mystiko_scheduler::{AbortPolicy, RetryPolicy, Scheduler, SchedulerOptions, SchedulerStatus, StartOptions};
use std::sync::Arc;

pub struct RollerScheduler {
    context: Arc<RollerContext>,
    scheduler: Scheduler<Option<RollerRunParams>, RollerTask>,
}

impl RollerScheduler {
    pub async fn new(context: Arc<RollerContext>) -> RollerResult<Self> {
        let roller = Arc::new(RollerTask::new(context.clone()).await?);
        let roller_status_getter = RollerStatusGetter::builder().status(context.status.clone()).build();
        let scheduler_options = SchedulerOptions::<Option<_>, RollerTask>::builder()
            .task(roller.clone())
            .status_server_port(context.config.scheduler.status_server_port)
            .status_getter(Arc::new(Box::new(roller_status_getter) as Box<dyn SchedulerStatus>))
            .build();
        let scheduler = Scheduler::new(scheduler_options);

        Ok(Self { context, scheduler })
    }

    pub async fn start(&self) -> RollerResult<()> {
        let roller_abort_policy = RollerAbortPolicy::builder().build();
        let roller_retry_policy = RollerRetryPolicy::builder().build();
        let options = StartOptions::<RollerError>::builder()
            .interval_ms(self.context.config.scheduler.schedule_interval_ms)
            .retry_policy(Arc::new(
                Box::new(roller_retry_policy) as Box<dyn RetryPolicy<RollerError>>
            ))
            .abort_policy(Arc::new(
                Box::new(roller_abort_policy) as Box<dyn AbortPolicy<RollerError>>
            ))
            .build();

        info!("roller start");
        Ok(self.scheduler.start(None, options).await?)
    }

    pub async fn wait_shutdown(&self) -> RollerResult<()> {
        Ok(self.scheduler.wait_shutdown().await?)
    }
}

pub async fn run() -> RollerResult<()> {
    let env_config = RollerEnvConfig::new()?;
    let context = Arc::new(create_roller_context(&env_config).await?);
    let scheduler = Arc::new(RollerScheduler::new(context.clone()).await?);
    scheduler.start().await?;
    scheduler.wait_shutdown().await
}
