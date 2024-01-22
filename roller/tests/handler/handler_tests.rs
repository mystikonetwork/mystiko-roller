use crate::mock::{create_mock_env_config, create_mock_mystiko_config};
use mystiko_dataloader::data::ContractData;
use mystiko_dataloader::handler::{CommitmentQueryOption, DataHandler, HandleOption, NullifierQueryOption};
use mystiko_roller::common::RollerChainData;
use mystiko_roller::handler::RollerDatabaseHandler;
use std::sync::Arc;

#[tokio::test]
async fn test_handler() {
    let env_config = create_mock_env_config();
    let mystiko_config = Arc::new(create_mock_mystiko_config().await);
    let handler = RollerDatabaseHandler::new(true, &env_config, mystiko_config.clone())
        .await
        .unwrap();
    handler.migrate().await.unwrap();
    handler.initialize().await.unwrap();
    let contracts = handler.query_loading_contracts(1).await.unwrap();
    assert!(contracts.is_none());
    let block = handler.query_chain_loaded_block(1).await.unwrap();
    assert!(block.is_some());
    let block = handler
        .query_contract_loaded_block(1, "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411")
        .await
        .unwrap();
    assert!(block.is_some());
    let commitments = handler
        .query_commitments(&CommitmentQueryOption {
            chain_id: 1,
            contract_address: "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411".to_string(),
            start_block: Some(0),
            commitment_hash: None,
            end_block: 0,
            status: None,
        })
        .await
        .unwrap();
    assert!(commitments.result.is_empty());
    let count = handler
        .count_commitments(&CommitmentQueryOption {
            chain_id: 1,
            contract_address: "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411".to_string(),
            start_block: Some(0),
            commitment_hash: None,
            end_block: 0,
            status: None,
        })
        .await
        .unwrap();
    assert_eq!(count.result, 0);
    let nullifiers = handler
        .query_nullifiers(&NullifierQueryOption {
            chain_id: 1,
            contract_address: "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411".to_string(),
            start_block: Some(0),
            end_block: 0,
            nullifier: None,
        })
        .await;
    assert!(nullifiers.is_err());
    let count = handler
        .count_nullifiers(&NullifierQueryOption {
            chain_id: 1,
            contract_address: "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411".to_string(),
            start_block: Some(0),
            end_block: 0,
            nullifier: None,
        })
        .await;
    assert!(count.is_err());
    let result = handler
        .handle(
            &RollerChainData::builder()
                .chain_id(1_u64)
                .contracts_data(vec![ContractData::builder()
                    .start_block(1_u64)
                    .end_block(2_u64)
                    .address("0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411".to_string())
                    .build()])
                .build(),
            &HandleOption::builder().config(mystiko_config).build(),
        )
        .await;
    assert!(result.is_ok());
}
