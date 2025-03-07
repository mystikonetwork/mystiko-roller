use ethers_core::types::{TxHash, U256};
use mystiko_crypto::zkp::G16Proof;
use mystiko_protocol::rollup::RollupProof;
use mystiko_protos::data::v1::Commitment;
use typed_builder::TypedBuilder;

pub const MAX_ROLLUP_BLOCK: u64 = u64::MAX;

#[derive(Debug, TypedBuilder)]
pub struct RollupPlanData {
    pub pool_address: String,
    pub total: usize,
    pub sizes: Vec<usize>,
    pub cms: Vec<Commitment>,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct RollupProofData {
    pub pool_address: String,
    pub rollup_size: usize,
    pub max_gas_price: U256,
    pub proof: RollupProof<G16Proof>,
    pub next_rollup: bool,
}

#[derive(Debug, TypedBuilder)]
pub struct RollupTransactionData {
    pub pool_address: String,
    pub transaction_hash: TxHash,
    pub block_number: u64,
}
