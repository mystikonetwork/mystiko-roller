use crate::common::{LiteDataDatabaseHandler, RollerChainData, RollerEnvConfig, RollerResult};
use async_trait::async_trait;
use mystiko_config::MystikoConfig;
use mystiko_dataloader::data::LiteData;
use mystiko_dataloader::handler::{
    CommitmentQueryOption, HandleOption, NullifierQueryOption, QueryResult, Result as HandlerResult,
};
use mystiko_dataloader::handler::{DataHandler, HandleResult};
use mystiko_protos::data::v1::{Commitment, Nullifier};
use mystiko_storage::{Collection, Document, MigrationHistory, SqlStatementFormatter};
use mystiko_storage::{StatementFormatter, Storage};
use mystiko_storage_sqlite::SqliteStorage;
use std::sync::Arc;

#[derive(Debug)]
pub struct RollerDatabaseHandler<F: StatementFormatter = SqlStatementFormatter, S: Storage = SqliteStorage> {
    pub database_handler: LiteDataDatabaseHandler<F, S>,
}

#[async_trait]
impl<F, S> DataHandler<LiteData> for RollerDatabaseHandler<F, S>
where
    F: StatementFormatter,
    S: Storage,
{
    async fn query_chain_loaded_block(&self, chain_id: u64) -> HandlerResult<Option<u64>> {
        self.database_handler.query_chain_loaded_block(chain_id).await
    }

    async fn query_contract_loaded_block(&self, chain_id: u64, contract_address: &str) -> HandlerResult<Option<u64>> {
        self.database_handler
            .query_contract_loaded_block(chain_id, contract_address)
            .await
    }

    async fn query_commitment(&self, option: &CommitmentQueryOption) -> HandlerResult<Option<Commitment>> {
        self.database_handler.query_commitment(option).await
    }

    async fn query_commitments(&self, option: &CommitmentQueryOption) -> HandlerResult<QueryResult<Vec<Commitment>>> {
        self.database_handler.query_commitments(option).await
    }

    async fn count_commitments(&self, option: &CommitmentQueryOption) -> HandlerResult<QueryResult<u64>> {
        self.database_handler.count_commitments(option).await
    }

    async fn query_nullifier(&self, option: &NullifierQueryOption) -> HandlerResult<Option<Nullifier>> {
        self.database_handler.query_nullifier(option).await
    }

    async fn query_nullifiers(&self, option: &NullifierQueryOption) -> HandlerResult<QueryResult<Vec<Nullifier>>> {
        self.database_handler.query_nullifiers(option).await
    }

    async fn count_nullifiers(&self, option: &NullifierQueryOption) -> HandlerResult<QueryResult<u64>> {
        self.database_handler.count_nullifiers(option).await
    }

    async fn handle(&self, data: &RollerChainData, option: &HandleOption) -> HandleResult {
        self.database_handler.handle(data, option).await
    }
}

impl RollerDatabaseHandler {
    pub async fn new(env_config: &RollerEnvConfig, mystiko_config: Arc<MystikoConfig>) -> RollerResult<Self> {
        let formatter = SqlStatementFormatter::sqlite();
        let storage = match env_config.memory_db {
            true => SqliteStorage::from_memory().await?,
            false => SqliteStorage::from_path(&env_config.data_file).await?,
        };
        let collection = Arc::new(Collection::new(formatter, storage));

        let database_handler = LiteDataDatabaseHandler::builder()
            .config(mystiko_config.clone())
            .collection(collection)
            .build();

        Ok(RollerDatabaseHandler { database_handler })
    }

    pub async fn migrate(&self) -> RollerResult<Vec<Document<MigrationHistory>>> {
        Ok(self.database_handler.migrate().await?)
    }

    pub async fn initialize(&self) -> RollerResult<()> {
        self.database_handler.initialize().await?;
        Ok(())
    }
}
