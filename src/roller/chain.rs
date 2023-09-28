use crate::common::RollerResult;
use crate::context::RollerContext;
use crate::roller::builder::RollupTxBuilder;
use crate::roller::pool::RollerPoolContracts;
use crate::roller::sender::RollupTxSender;
use crate::roller::types::RollupProofData;
use log::{error, info};
use std::sync::Arc;

pub struct ChainRoller {
    pools: RollerPoolContracts,
    tx_builder: Arc<RollupTxBuilder>,
    tx_sender: Arc<RollupTxSender>,
}

impl ChainRoller {
    pub async fn new(context: Arc<RollerContext>) -> RollerResult<ChainRoller> {
        let pools = RollerPoolContracts::new(context.clone()).await?;
        let builder = RollupTxBuilder::builder().context(context.clone()).build();
        let sender = RollupTxSender::builder().context(context.clone()).build();
        Ok(ChainRoller {
            pools,
            tx_builder: Arc::new(builder),
            tx_sender: Arc::new(sender),
        })
    }

    pub async fn run(&self) -> RollerResult<()> {
        let txs = self.build_rollup_transactions().await?;
        if !txs.is_empty() {
            self.send_rollup_transactions(txs).await?;
        }
        Ok(())
    }

    pub async fn build_rollup_transactions(&self) -> RollerResult<Vec<RollupProofData>> {
        let mut txs = vec![];
        for address in self.pools.addresses().await?.into_iter() {
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

    pub async fn send_rollup_transactions(&self, txs: Vec<RollupProofData>) -> RollerResult<()> {
        let mut rollup_data = vec![];
        for tx in txs.into_iter() {
            let result = self.tx_sender.send(tx.clone()).await;
            match result {
                Ok(date) => {
                    info!(
                        "pool contract[address={:?}] send pool rollup transaction success {:?}",
                        tx.pool_address, date
                    );
                    rollup_data.push(date);
                }
                Err(e) => {
                    error!(
                        "pool contract[address={:?}] send pool rollup transaction error {:?}",
                        tx.pool_address.clone(),
                        e
                    );
                }
            }
        }

        self.pools.update_latest_rollup_block(&rollup_data).await;
        Ok(())
    }
}
