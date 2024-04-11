use crate::common::RollerMonitorConfig;
use crate::common::{IntoMessage, MonitorAlert};
use crate::common::{RollerMonitorError, RollerMonitorResult};
use async_trait::async_trait;
use ethers_providers::Middleware;
use mystiko_abi::commitment_pool::CommitmentPool;
use mystiko_config::MystikoConfig;
use mystiko_ethers::{ChainConfigProvidersOptions, Provider, ProviderPool, Providers};
use mystiko_notification::Notification;
use mystiko_notification::SnsNotification;
use mystiko_scheduler::SchedulerTask;
use mystiko_sequencer_client::v1::SequencerClient as SequencerClientV1;
use mystiko_sequencer_client::SequencerClient;
use mystiko_utils::address::ethers_address_from_string;
use mystiko_utils::convert::u256_to_biguint;
use mystiko_utils::json::to_safe_json_string;
use std::fmt::Debug;
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct RollerMonitor<
    M = rusoto_sns::PublishInput,
    N: Notification<M> = SnsNotification,
    P: Providers = ProviderPool<ChainConfigProvidersOptions>,
> {
    config: Arc<RollerMonitorConfig>,
    mystiko_config: Arc<MystikoConfig>,
    sequencer: Arc<SequencerClientV1>,
    providers: Arc<P>,
    notification: Arc<N>,
    #[builder(default, setter(skip))]
    _phantom: std::marker::PhantomData<M>,
}

#[async_trait]
impl<M, N, P> SchedulerTask<()> for RollerMonitor<M, N, P>
where
    M: Clone + Send + Sync,
    MonitorAlert<M>: IntoMessage<M>,
    N: Notification<M>,
    N::Error: Debug,
    P: Providers,
{
    type Error = RollerMonitorError;

    async fn run(&self, args: &()) -> RollerMonitorResult<()> {
        log::info!(
            "roller_monitor start to run with args: {:?}",
            to_safe_json_string(&args, false)?
        );
        let chains = self.mystiko_config.chains();
        let checks = chains
            .iter()
            .map(|c| async move { self.check_chain(c.chain_id(), c.name()).await })
            .collect::<Vec<_>>();
        let check_results = futures::future::join_all(checks).await;
        let error = check_results.into_iter().find_map(|result| match result {
            Ok(_) => None,
            Err(err) => Some(err),
        });

        if let Some(error) = error {
            log::error!("roller_monitor raised error when running: {}", error);
            return Err(error);
        }
        log::info!("roller_monitor run successfully");
        Ok(())
    }
}

impl<M, N, P> RollerMonitor<M, N, P>
where
    M: Clone + Send + Send,
    MonitorAlert<M>: IntoMessage<M>,
    N: Notification<M>,
    N::Error: Debug,
    P: Providers,
{
    pub async fn from_config(
        config: Arc<RollerMonitorConfig>,
        mystiko_config: Arc<MystikoConfig>,
        providers: Arc<P>,
        notification: Arc<N>,
    ) -> RollerMonitorResult<Self> {
        let sequencer = Arc::new(SequencerClientV1::connect(&config.sequencer).await?);
        Ok(RollerMonitor::builder()
            .config(config.clone())
            .mystiko_config(mystiko_config)
            .notification(notification)
            .providers(providers)
            .sequencer(sequencer)
            .build())
    }

    pub async fn check_chain(&self, chain_id: u64, chain_name: &str) -> RollerMonitorResult<()> {
        log::info!("check chain={}", chain_name);
        let provider = self.providers.get_provider(chain_id).await?;
        let chain_cfg = self
            .mystiko_config
            .find_chain(chain_id)
            .ok_or(RollerMonitorError::ChainConfigNotFoundError(chain_id))?;
        for contract in chain_cfg.pool_contracts() {
            if contract.version() >= 6 {
                self.check_contract(chain_id, chain_name, contract.address(), provider.clone())
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn check_contract(
        &self,
        chain_id: u64,
        chain_name: &str,
        contract_address: &str,
        provider: Arc<Provider>,
    ) -> RollerMonitorResult<()> {
        log::info!("check chain={} contract={}", chain_name, contract_address);
        let address = ethers_address_from_string(contract_address)
            .map_err(|_| RollerMonitorError::ConvertContractAddressError(contract_address.to_string()))?;
        let pool = CommitmentPool::new(address, provider.clone());
        let queued_cm = pool
            .get_queued_commitments()
            .await
            .map_err(|e| RollerMonitorError::ContractCallError("get_queued_commitments".to_string(), e.to_string()))?;
        let latest_block_number = provider.get_block_number().await?.as_u64();
        if let Some(cm) = queued_cm.first() {
            let cm_hash = u256_to_biguint(cm);
            let commitment = self.sequencer.get_commitments(chain_id, &address, &[cm_hash]).await?;
            let max_delay_block = self.config.get_max_rollup_delay_block(chain_id);
            if let Some(c) = commitment.first() {
                if c.block_number + max_delay_block <= latest_block_number {
                    log::warn!(
                        "commitment={} is not included in the latest {} blocks",
                        cm.clone(),
                        max_delay_block
                    );
                    let error_message = format!(
                        "🚨 Roller Monitor Alert 🚨\n\n\
                        chain={} contract={} commitment={} \
                        is not included for {} blocks",
                        chain_name,
                        contract_address,
                        cm.clone(),
                        max_delay_block
                    );
                    self.notification
                        .push(
                            MonitorAlert::<M>::builder()
                                .error_message(error_message)
                                .topic_arn(self.config.notification.topic_arn.clone())
                                .build()
                                .into_message(),
                        )
                        .await
                        .map_err(|err| RollerMonitorError::PushMessageError(format!("{:?}", err)))?;
                }
            }
        }
        Ok(())
    }
}
