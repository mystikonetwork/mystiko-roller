use crate::common::{RollerError, RollerResult};
use crate::context::RollerContext;
use crate::roller::builder::calc::calc_rollup_size_queue;
use crate::roller::builder::{calc_total_rollup_fee, circuit_type_from_rollup_size};
use crate::roller::types::{RollupPlanData, RollupProofData, MAX_ROLLUP_BLOCK};
use ethers_core::types::U256;
use ethers_providers::Middleware;
use log::{debug, info, warn};
use mystiko_abi::commitment_pool::CommitmentPool;
use mystiko_crypto::merkle_tree::MerkleTree;
use mystiko_crypto::zkp::{G16Proof, G16Prover};
use mystiko_dataloader::handler::{CommitmentQueryOption, DataHandler};
use mystiko_downloader::DownloaderBuilder;
use mystiko_protocol::rollup::{Rollup, RollupProof};
use mystiko_protos::data::v1::{Commitment, CommitmentStatus};
use mystiko_utils::address::ethers_address_from_string;
use mystiko_utils::convert::bytes_to_biguint;
use num_bigint::BigUint;
use std::ops::Mul;
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct RollupTxBuilder {
    context: Arc<RollerContext>,
}

impl RollupTxBuilder {
    pub async fn build(&self, pool_address: String) -> RollerResult<Option<RollupProofData>> {
        debug!("build rollup transaction");
        let option = CommitmentQueryOption::builder()
            .chain_id(self.context.config.chain_id)
            .contract_address(pool_address.clone())
            .end_block(MAX_ROLLUP_BLOCK)
            .status(CommitmentStatus::Queued)
            .build();
        let queued: mystiko_dataloader::handler::QueryResult<Vec<Commitment>> =
            self.context.handler.query_commitments(&option).await?;
        if !queued.result.is_empty() {
            let address = ethers_address_from_string(pool_address.clone())
                .map_err(|_| RollerError::ConvertContractAddressError(pool_address.clone()))?;
            let pool = CommitmentPool::new(address, self.context.provider.clone());
            let included_count = pool
                .get_commitment_included_count()
                .await
                .map_err(|e| {
                    RollerError::ContractCallError("get_commitment_included_count".to_string(), e.to_string())
                })?
                .as_u64();
            let mut cms = Vec::new();
            for c in &queued.result {
                if let Some(index) = c.leaf_index {
                    if index >= included_count {
                        cms.push(c.clone());
                    }
                } else {
                    return Err(RollerError::RollerInternalError(
                        "handler commitment leaf_index is none".to_string(),
                    ));
                }
            }
            if !cms.is_empty() {
                return self.build_rollup_plan(&pool_address, cms, included_count).await;
            }
        }

        Ok(None)
    }

    async fn build_rollup_plan(
        &self,
        pool_address: &str,
        queued_cms: Vec<Commitment>,
        included: u64,
    ) -> RollerResult<Option<RollupProofData>> {
        let (total_size_plan, sizes_plan) = calc_rollup_size_queue(
            included as usize,
            queued_cms.len(),
            self.context.config.rollup.max_rollup_size,
        )?;
        info!("build rollup contract {:?} plan {:?}", pool_address, sizes_plan);
        let plan = RollupPlanData::builder()
            .pool_address(pool_address.to_string())
            .total(total_size_plan)
            .sizes(sizes_plan)
            .cms(queued_cms)
            .build();
        let max_gas_price = self.calc_max_gas_price(&plan).await?;
        let rollup_size = *plan.sizes.first().ok_or_else(|| RollerError::RollupSizeError(0))?;
        let rollup_cms = plan
            .cms
            .iter()
            .take(rollup_size)
            .map(|c| bytes_to_biguint(&c.commitment_hash))
            .collect::<Vec<_>>();
        let proof = self.generate_proof(pool_address, rollup_cms, included).await?;
        Ok(Some(
            RollupProofData::builder()
                .pool_address(pool_address.to_string())
                .rollup_size(rollup_size)
                .next_rollup(plan.sizes.len() > 1)
                .max_gas_price(max_gas_price)
                .proof(proof)
                .build(),
        ))
    }

    pub async fn generate_proof(
        &self,
        pool_address: &str,
        new_leaves: Vec<BigUint>,
        included: u64,
    ) -> RollerResult<RollupProof<G16Proof>> {
        let rollup_size = new_leaves.len();
        let circuits_type = circuit_type_from_rollup_size(rollup_size)?;

        info!("generate rollup proof with size={:?}", rollup_size);
        let pool_contract = self
            .context
            .mystiko_config
            .find_pool_contract_by_address(self.context.config.chain_id, pool_address)
            .ok_or(RollerError::PoolContractConfigNotFoundError(pool_address.to_string()))?;

        let circuits_cfg = pool_contract
            .circuit_by_type(&circuits_type)
            .ok_or(RollerError::CircuitNotFoundError(circuits_type as i32))?;

        let mut downloader = DownloaderBuilder::new()
            .folder(&self.context.env_config.circuits_path)
            .build()
            .await?;

        let program = downloader
            .read_bytes_failover(circuits_cfg.program_file(), None)
            .await?;
        let abi = downloader.read_bytes_failover(circuits_cfg.abi_file(), None).await?;
        let proving_key = downloader
            .read_bytes_failover(circuits_cfg.proving_key_file(), None)
            .await?;

        let mut tree = self.build_merkle_tree(pool_address, included).await?;
        let mut rollup = Rollup::builder()
            .tree(&mut tree)
            .new_leaves(new_leaves)
            .program(program)
            .abi(abi)
            .proving_key(proving_key)
            .build();
        let prover = Arc::new(G16Prover);
        Ok(rollup.prove(prover)?)
    }

    pub async fn calc_max_gas_price(&self, plan: &RollupPlanData) -> RollerResult<U256> {
        let plan_max = self.calc_max_gas_price_by_plan(plan).await?;
        let config_max = U256::from(self.context.config.max_gas_price());
        let provider_current = self.context.tx.gas_price(&self.context.provider).await?;
        info!(
            "plan max gas price={:?} provider current gas price={:?} config max gas price={:?} ",
            plan_max, provider_current, config_max
        );
        if plan_max == U256::zero() {
            return Ok(provider_current);
        }
        match plan_max.cmp(&config_max) {
            std::cmp::Ordering::Greater => self.check_plan_gas_price_greater(&plan_max, &provider_current),
            std::cmp::Ordering::Less | std::cmp::Ordering::Equal => {
                self.check_plan_gas_price_less(plan, &plan_max, &config_max, &provider_current)
                    .await
            }
        }
    }

    fn check_plan_gas_price_greater(&self, plan_max: &U256, provider_current: &U256) -> RollerResult<U256> {
        if plan_max < provider_current {
            warn!(
                "price too high plan max gas price={:?} provider current gas price={:?}",
                plan_max, provider_current
            );
            return Err(RollerError::CurrentGasPriceTooHighError(provider_current.to_string()));
        }

        self.choose_gas_price(*plan_max, *provider_current)
    }

    async fn check_plan_gas_price_less(
        &self,
        plan: &RollupPlanData,
        plan_max: &U256,
        config_max: &U256,
        provider_current: &U256,
    ) -> RollerResult<U256> {
        let block_number = self.context.provider.get_block_number().await?.as_u64();
        let first_cm = plan
            .cms
            .first()
            .ok_or_else(|| RollerError::RollerInternalError("commitments is empty".to_string()))?;

        if block_number >= first_cm.block_number + self.context.config.force_rollup_block_count() {
            if config_max < provider_current {
                warn!(
                    "gas price too high config max gas price={:?} provider current gas price={:?}",
                    config_max, provider_current
                );
                return Err(RollerError::CurrentGasPriceTooHighError(provider_current.to_string()));
            }

            info!("do force rollup");
            return self.choose_gas_price(*config_max, *provider_current);
        } else if plan_max < provider_current {
            warn!(
                "gas price too high plan max gas price={:?} provider current gas price={:?}",
                plan_max, provider_current
            );
            return Err(RollerError::CurrentGasPriceTooHighError(provider_current.to_string()));
        }

        self.choose_gas_price(*plan_max, *provider_current)
    }

    fn choose_gas_price(&self, max_price: U256, provider_current: U256) -> RollerResult<U256> {
        if self.context.tx.tx_eip1559() {
            Ok(std::cmp::min(provider_current.mul(2), max_price))
        } else {
            Ok(provider_current)
        }
    }

    async fn calc_max_gas_price_by_plan(&self, plan: &RollupPlanData) -> RollerResult<U256> {
        let total_fee = calc_total_rollup_fee(&plan.cms, plan.total)?;
        let mut total_gas_cost = 0;
        for size in plan.sizes.iter() {
            let cost = self.context.config.clone().rollup_gas_cost(*size);
            total_gas_cost += cost;
        }

        let chain_cfg = self
            .context
            .mystiko_config
            .find_chain(self.context.config.chain_id)
            .ok_or(RollerError::ChainConfigNotFoundError(self.context.config.chain_id))?;
        let contract_cfg = chain_cfg
            .find_pool_contract_by_address(&plan.pool_address)
            .ok_or(RollerError::PoolContractConfigNotFoundError(plan.pool_address.clone()))?;
        let asset_symbol = chain_cfg.asset_symbol().to_string();
        let asset_decimals = chain_cfg.asset_decimals();
        let swap_amount = self
            .context
            .price
            .swap(
                contract_cfg.asset_symbol(),
                contract_cfg.asset_decimals(),
                total_fee,
                &asset_symbol,
                asset_decimals,
            )
            .await?;
        Ok(swap_amount / total_gas_cost)
    }

    async fn build_merkle_tree(&self, pool_address: &str, included: u64) -> RollerResult<MerkleTree> {
        let option = CommitmentQueryOption::builder()
            .chain_id(self.context.config.chain_id)
            .contract_address(pool_address.to_string())
            .end_block(MAX_ROLLUP_BLOCK)
            .build();
        let cms = self.context.handler.query_commitments(&option).await?.result;
        if cms.len() < included as usize {
            return Err(RollerError::RollerInternalError(format!(
                "handler commitments len={:?} less than included count={:?}",
                cms.len(),
                included
            )));
        }
        let elements = cms
            .iter()
            .take(included as usize)
            .map(|c| bytes_to_biguint(&c.commitment_hash))
            .collect::<Vec<_>>();
        Ok(MerkleTree::new(
            Some(elements),
            Some(self.context.config.merkle_tree_height()),
            None,
        )?)
    }
}
