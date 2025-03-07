use crate::mock::create_mock_context;
use crate::mock::mock_proof_data;
use crate::mock::mock_transaction_data;
use crate::mock::mock_transaction_receipt_data;
use crate::mock::MockRollerHandler;
use crate::mock::MockRollerTokenPrice;
use crate::mock::MockRollerTxManager;
use ethers_core::types::Bytes;
use ethers_core::types::U256;
use mystiko_dataloader::data::LiteData;
use mystiko_dataloader::handler::QueryResult;
use mystiko_protos::data::v1::Commitment;
use mystiko_roller::common::JsonProviderWrapper;
use mystiko_roller::roller::ChainRoller;
use mystiko_roller::roller::RollupProofData;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn test_run_empty_commitments_success() {
    let (mut mock_context, mock_provider) = create_mock_context(None).await;
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().times(3).returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![],
        })
    });
    mock_context.handler = Arc::new(Box::new(mock_handler) as Box<_>);
    let arc_mock_context = Arc::new(mock_context);
    let include_count = Bytes::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();

    let roller = ChainRoller::new(arc_mock_context).await.unwrap();
    let result = roller.run().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_once_empty_commitments_success() {
    let (mut mock_context, mock_provider) = create_mock_context(None).await;
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().times(1).returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![],
        })
    });
    mock_context.handler = Arc::new(Box::new(mock_handler) as Box<_>);
    let arc_mock_context = Arc::new(mock_context);
    let include_count = Bytes::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();
    let roller = ChainRoller::new(arc_mock_context).await.unwrap();
    let pool_address = vec!["0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411".to_string()];
    let result = roller.run_once(&pool_address).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_build_plan_error() {
    let (mut mock_context, mock_provider) = create_mock_context(None).await;
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().times(3).returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![Commitment::builder().commitment_hash("123").rollup_fee(vec![1]).build()],
        })
    });
    let mut token_price = MockRollerTokenPrice::new();
    token_price
        .expect_price()
        .returning(|_| Ok(f64::from_str("1000000000").unwrap()));
    token_price
        .expect_swap()
        .returning(|_, _, _, _, _| Ok(U256::from(1000000000_u64)));
    let mut tx_manager = MockRollerTxManager::new();
    tx_manager
        .expect_gas_price()
        .returning(|_| Ok(U256::from_str("1000000000").unwrap()));
    mock_context.handler = Arc::new(Box::new(mock_handler) as Box<_>);
    mock_context.price = Arc::new(token_price);
    mock_context.tx = Arc::new(tx_manager);
    let include_count = Bytes::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();
    let arc_mock_context = Arc::new(mock_context);
    let roller = ChainRoller::new(arc_mock_context).await.unwrap();
    let result = roller.run().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_chain_rollup_success() {
    let (mut mock_context, _mock_provider) = create_mock_context(None).await;
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![],
        })
    });
    let mut tx_manager = MockRollerTxManager::<JsonProviderWrapper>::new();
    tx_manager
        .expect_estimate_gas()
        .returning(|_, _| Ok(U256::from(100_u64)));
    let transaction = mock_transaction_data();
    let receipt = mock_transaction_receipt_data();
    tx_manager.expect_send().returning(move |_, _| Ok(transaction.hash()));
    tx_manager.expect_confirm().returning(move |_, _| Ok(receipt.clone()));
    mock_context.handler = Arc::new(Box::new(mock_handler) as Box<_>);
    mock_context.tx = Arc::new(tx_manager);

    let arc_mock_context = Arc::new(mock_context);
    let address = "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411";
    let gas_price = U256::from(100000000_u64);
    let data = mock_proof_data().await;
    let proof_data = RollupProofData::builder()
        .pool_address(address.to_string())
        .rollup_size(1_usize)
        .next_rollup(true)
        .max_gas_price(gas_price)
        .proof(data.clone())
        .build();
    let roller = ChainRoller::new(arc_mock_context.clone()).await.unwrap();
    let result = roller.send_rollup_transactions(vec![proof_data.clone()]).await.unwrap();
    assert!(result.len() == 1);
    assert_eq!(result[0], "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411".to_string());

    let proof_data = RollupProofData::builder()
        .pool_address(address.to_string())
        .rollup_size(1_usize)
        .next_rollup(false)
        .max_gas_price(gas_price)
        .proof(data.clone())
        .build();
    let roller = ChainRoller::new(arc_mock_context.clone()).await.unwrap();
    let result = roller.send_rollup_transactions(vec![proof_data.clone()]).await.unwrap();
    assert!(result.is_empty());
}
