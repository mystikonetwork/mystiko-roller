use crate::common::{RollerError, RollerResult};
use crate::context::RollerContext;
use crate::roller::types::RollupTransactionData;
use log::error;
use mystiko_dataloader::handler::DataHandler;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use typed_builder::TypedBuilder;

#[derive(Debug, TypedBuilder)]
pub struct RollerPoolContract {
    last_rollup_block: u64,
}

pub struct RollerPoolContracts {
    context: Arc<RollerContext>,
    pools: RwLock<HashMap<String, RollerPoolContract>>,
}

impl RollerPoolContracts {
    pub async fn new(context: Arc<RollerContext>) -> RollerResult<RollerPoolContracts> {
        let chain_config = context
            .mystiko_config
            .find_chain(context.config.chain_id)
            .ok_or(RollerError::ChainConfigNotFoundError(context.config.chain_id))?;

        let mut pools = HashMap::new();
        for pool in chain_config.pool_contracts() {
            pools.insert(
                pool.address().to_string(),
                RollerPoolContract::builder()
                    .last_rollup_block(pool.start_block() + 1)
                    .build(),
            );
        }

        Ok(RollerPoolContracts {
            context,
            pools: RwLock::new(pools),
        })
    }

    pub async fn addresses(&self) -> RollerResult<Vec<String>> {
        let pools = self.pools.read().await;
        let mut addresses = vec![];
        for (addr, pool) in pools.iter() {
            let pool_addr = addr.to_string();
            let loaded_block = self
                .context
                .handler
                .query_contract_loaded_block(self.context.config.chain_id, &pool_addr)
                .await?;
            if let Some(loaded_block) = loaded_block {
                if loaded_block >= pool.last_rollup_block {
                    addresses.push(pool_addr);
                }
            }
        }

        Ok(addresses)
    }

    pub async fn update_latest_rollup_block(&self, txs: &[RollupTransactionData]) {
        let mut pools = self.pools.write().await;
        for tx in txs {
            if let Some(pool) = pools.get_mut(&tx.pool_address) {
                pool.last_rollup_block = tx.block_number;
            } else {
                error!("pool contract[address={:?}] not found", tx.pool_address);
            }
        }
    }
}
