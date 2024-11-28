use crate::common::{MonitorRollupError, MonitorRollupResult};
use async_trait::async_trait;
use log::{error, info};
use mystiko_abi::commitment_pool::CommitmentPool;
use mystiko_config::MystikoConfig;
use mystiko_ethers::{ChainConfigProvidersOptions, Provider, ProviderPool, Providers};
use mystiko_roller::common::RollerEnvConfig;
use mystiko_roller::context::create_roller_context;
use mystiko_roller::scheduler::task::RollerTask;
use mystiko_scheduler::SchedulerTask;
use mystiko_utils::address::ethers_address_from_string;
use mystiko_utils::json::to_safe_json_string;
use std::fmt::Debug;
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct MonitorRollup<P: Providers = ProviderPool<ChainConfigProvidersOptions>> {
    mystiko_config: Arc<MystikoConfig>,
    providers: Arc<P>,
}

#[async_trait]
impl<P> SchedulerTask<()> for MonitorRollup<P>
where
    P: Providers,
{
    type Error = MonitorRollupError;

    async fn run(&self, args: &()) -> MonitorRollupResult<()> {
        info!(
            "monitor_rollup start to run with args: {:?}",
            to_safe_json_string(&args, false)?
        );
        let chains = self.mystiko_config.chains();
        let checks = chains
            .iter()
            .map(|c| async move { self.check_chain(c.chain_id()).await })
            .collect::<Vec<_>>();
        let results = futures::future::join_all(checks).await;
        let chains = results
            .iter()
            .filter(|(_, result)| *result)
            .map(|(chain_id, _)| *chain_id)
            .collect::<Vec<u64>>();
        self.rollup(chains).await;
        info!("monitor_rollup run successfully");
        Ok(())
    }
}

impl<P> MonitorRollup<P>
where
    P: Providers,
{
    pub async fn from_config(mystiko_config: Arc<MystikoConfig>, providers: Arc<P>) -> MonitorRollupResult<Self> {
        Ok(MonitorRollup::builder()
            .mystiko_config(mystiko_config)
            .providers(providers)
            .build())
    }

    async fn check_chain(&self, chain_id: u64) -> (u64, bool) {
        match self.check_chain_status(chain_id).await {
            Ok(result) => (chain_id, result),
            Err(err) => {
                log::error!("check_chain raised error: {}", err);
                (chain_id, false)
            }
        }
    }

    async fn check_chain_status(&self, chain_id: u64) -> MonitorRollupResult<bool> {
        log::info!("check chain={}", chain_id);
        let provider = self.providers.get_provider(chain_id).await?;
        let chain_cfg = self
            .mystiko_config
            .find_chain(chain_id)
            .ok_or(MonitorRollupError::ChainConfigNotFoundError(chain_id))?;
        for contract in chain_cfg.pool_contracts() {
            if contract.version() >= 6 && !contract.disabled() {
                let do_rollup = self
                    .check_contract(chain_id, contract.address(), provider.clone())
                    .await?;
                if do_rollup {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    async fn check_contract(
        &self,
        chain_id: u64,
        contract_address: &str,
        provider: Arc<Provider>,
    ) -> MonitorRollupResult<bool> {
        log::info!("check chain={} contract={}", chain_id, contract_address);
        let address = ethers_address_from_string(contract_address)
            .map_err(|_| MonitorRollupError::ConvertContractAddressError(contract_address.to_string()))?;
        let pool = CommitmentPool::new(address, provider.clone());
        let queued = pool.get_commitment_queued_count().await.map_err(|e| {
            MonitorRollupError::ContractCallError("get_commitment_queued_count".to_string(), e.to_string())
        })?;
        if !queued.is_zero() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn rollup(&self, chains: Vec<u64>) {
        for chain_id in chains {
            let _ = self.rollup_for_chain(chain_id).await.map_err(|e| {
                error!("rollup for chain {:?} raised error: {:?}", chain_id, e);
            });
        }
    }

    async fn rollup_for_chain(&self, chain_id: u64) -> MonitorRollupResult<()> {
        info!("rollup for chain={}", chain_id);
        let env_config = RollerEnvConfig::new()?;
        let context = create_roller_context(&env_config, Some(chain_id)).await?;
        let roller = RollerTask::new(Arc::new(context)).await?;
        roller.run(&None).await?;
        info!("rollup for chain={} successfully", chain_id);
        Ok(())
    }
}
