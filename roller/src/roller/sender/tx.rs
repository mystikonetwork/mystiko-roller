use crate::common::{RollerError, RollerResult};
use crate::context::RollerContext;
use crate::roller::types::RollupProofData;
use crate::roller::RollupTransactionData;
use ethers_core::types::{Bytes, U256};
use ethers_providers::Middleware;
use log::{debug, info, warn};
use mystiko_abi::commitment_pool::{CommitmentPool, RollupRequest};
use mystiko_ethers::Provider;
use mystiko_server_utils::tx_manager::TransactionData;
use mystiko_utils::address::ethers_address_from_string;
use mystiko_utils::convert::biguint_to_u256;
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct RollupTxSender {
    context: Arc<RollerContext>,
}

impl RollupTxSender {
    pub async fn send(&self, proof_data: RollupProofData) -> RollerResult<RollupTransactionData> {
        debug!("send rollup transaction");
        let address = ethers_address_from_string(&proof_data.pool_address)
            .map_err(|_| RollerError::ConvertContractAddressError(proof_data.pool_address.clone()))?;
        let pool = CommitmentPool::new(address, self.context.provider.clone());
        let tx_data = self.build_tx_param(&pool, &proof_data).await?;

        let mut tx_data = TransactionData::builder()
            .to(address)
            .data(tx_data)
            .value(U256::zero())
            .gas(U256::zero())
            .max_price(proof_data.max_gas_price)
            .build();
        let gas_limit = self.context.tx.estimate_gas(&tx_data, &self.context.provider).await?;
        tx_data.gas = gas_limit;
        info!(
            "send rollup transaction with max gas price={:?}",
            proof_data.max_gas_price
        );
        let tx_hash = self.context.tx.send(&tx_data, &self.context.provider).await?;
        info!("send rollup transaction hash: {:?}", tx_hash);
        let receipt = self.context.tx.confirm(&tx_hash, &self.context.provider).await?;
        let block_number = if let Some(block_number) = receipt.block_number {
            block_number.as_u64()
        } else {
            warn!("rollup transaction receipt block number is none");
            self.context.provider.get_block_number().await?.as_u64()
        };

        info!("rollup transaction have been confirmed");
        Ok(RollupTransactionData::builder()
            .pool_address(proof_data.pool_address)
            .transaction_hash(receipt.transaction_hash)
            .block_number(block_number)
            .build())
    }

    async fn build_tx_param(
        &self,
        pool: &CommitmentPool<Provider>,
        proof_data: &RollupProofData,
    ) -> RollerResult<Bytes> {
        let request = RollupRequest {
            proof: proof_data.proof.zk_proof.convert_to()?,
            rollup_size: proof_data.rollup_size as u32,
            new_root: biguint_to_u256(&proof_data.proof.new_root),
            leaf_hash: biguint_to_u256(&proof_data.proof.leaves_hash),
        };

        let call = pool.rollup(request);
        let call_data = call.calldata().ok_or(RollerError::InvalidTransactionCallDataError)?;
        Ok(call_data)
    }
}
