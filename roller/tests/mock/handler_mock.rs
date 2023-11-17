use async_trait::async_trait;
use mockall::mock;
use mystiko_config::ContractConfig;
use mystiko_dataloader::data::ChainData;
use mystiko_dataloader::data::LoadedData;
use mystiko_dataloader::handler::{
    CommitmentQueryOption, DataHandler, HandleOption, HandleResult, NullifierQueryOption, QueryResult,
    Result as HandlerErrorResult,
};
use mystiko_dataloader::loader::ResetOptions;
use mystiko_protos::data::v1::{Commitment, Nullifier};

mock! {
    #[derive(Debug)]
    pub RollerHandler<R>
    where
        R: LoadedData,
    {}

    #[async_trait]
    impl<R> DataHandler<R> for RollerHandler<R>
    where
        R: LoadedData,
    {
        async fn query_loading_contracts(&self, chain_id: u64) -> HandlerErrorResult<Option<Vec<ContractConfig>>>;
        async fn query_chain_loaded_block(&self, chain_id: u64) -> HandlerErrorResult<Option<u64>>;
        async fn query_contract_loaded_block(&self, chain_id: u64, contract_address: &str) -> HandlerErrorResult<Option<u64>>;
        async fn query_commitments(&self, option: &CommitmentQueryOption) -> HandlerErrorResult<QueryResult<Vec<Commitment>>>;
        async fn count_commitments(&self, option: &CommitmentQueryOption) -> HandlerErrorResult<QueryResult<u64>>;
        async fn query_nullifiers(&self, option: &NullifierQueryOption) -> HandlerErrorResult<QueryResult<Vec<Nullifier>>>;
        async fn count_nullifiers(&self, option: &NullifierQueryOption) -> HandlerErrorResult<QueryResult<u64>>;
        async fn handle(&self, data: &ChainData<R>, option: &HandleOption) -> HandleResult;
        async fn reset(&self, option: &ResetOptions) -> HandlerErrorResult<()>;
    }
}
