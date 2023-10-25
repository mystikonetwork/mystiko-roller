use crate::common::{RollerError, RollerResult};
use crate::context::RollerContext;
use crate::roller::builder::RollupTxBuilder;
use crate::roller::sender::RollupTxSender;
use crate::roller::types::RollupProofData;
use log::{error, info, warn};
use std::sync::Arc;

pub struct ChainRoller {
    context: Arc<RollerContext>,
    pool_contracts: Vec<String>,
    tx_builder: Arc<RollupTxBuilder>,
    tx_sender: Arc<RollupTxSender>,
}

impl ChainRoller {
    pub async fn new(context: Arc<RollerContext>) -> RollerResult<ChainRoller> {
        let chain_config = context
            .mystiko_config
            .find_chain(context.config.chain_id)
            .ok_or(RollerError::ChainConfigNotFoundError(context.config.chain_id))?;
        let pool_contracts: Vec<String> = chain_config
            .pool_contracts()
            .iter()
            .map(|pool| pool.address().to_string())
            .collect();
        let builder = RollupTxBuilder::builder().context(context.clone()).build();
        let sender = RollupTxSender::builder().context(context.clone()).build();
        Ok(ChainRoller {
            context,
            pool_contracts,
            tx_builder: Arc::new(builder),
            tx_sender: Arc::new(sender),
        })
    }

    pub async fn run(&self) -> RollerResult<()> {
        let mut pools = self.pool_contracts.clone();
        for _ in 0..self.context.config.rollup.max_rollup_one_round {
            if pools.is_empty() {
                break;
            }
            pools = self.run_once(&pools).await?;
        }
        Ok(())
    }

    pub async fn run_once(&self, pools: &[String]) -> RollerResult<Vec<String>> {
        let txs = self.build_rollup_transactions(pools).await?;
        if !txs.is_empty() {
            self.send_rollup_transactions(txs).await
        } else {
            Ok(vec![])
        }
    }

    pub async fn build_rollup_transactions(&self, pools: &[String]) -> RollerResult<Vec<RollupProofData>> {
        let mut txs = vec![];
        for address in pools.iter() {
            let result = self.tx_builder.build(address.clone()).await;
            match result {
                Ok(tx) => {
                    if let Some(tx) = tx {
                        txs.push(tx)
                    }
                }
                Err(e) => {
                    error!(
                        "pool contract[address={:?}] build rollup transaction error {:?}",
                        address, e
                    );
                }
            }
        }
        Ok(txs)
    }

    pub async fn send_rollup_transactions(&self, txs: Vec<RollupProofData>) -> RollerResult<Vec<String>> {
        let mut next_rollup_pools = vec![];
        for tx in txs.into_iter() {
            let result = self.tx_sender.send(tx.clone()).await;
            match result {
                Ok(date) => {
                    info!(
                        "pool contract[address={:?}] send pool rollup transaction success {:?}",
                        tx.pool_address, date
                    );

                    if tx.next_rollup {
                        next_rollup_pools.push(tx.pool_address.clone());
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("nonce too low") {
                        warn!(
                            "pool contract[address={:?}] send pool rollup transaction error {:?}",
                            tx.pool_address.clone(),
                            e
                        );
                    } else {
                        error!(
                            "pool contract[address={:?}] send pool rollup transaction error {:?}",
                            tx.pool_address.clone(),
                            e
                        );
                    }
                }
            }
        }

        Ok(next_rollup_pools)
    }
}
