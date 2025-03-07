use crate::mock::create_mock_context;
use crate::mock::mock_proof_data;
use crate::mock::mock_transaction_data;
use crate::mock::mock_transaction_receipt_data;
use crate::mock::MockRollerHandler;
use crate::mock::MockRollerTxManager;
use ethers_core::types::U256;
use mystiko_dataloader::data::LiteData;
use mystiko_dataloader::handler::QueryResult;
use mystiko_roller::common::JsonProviderWrapper;
use mystiko_roller::common::RollerError;
use mystiko_roller::roller::{RollupProofData, RollupTxSender};
use std::sync::Arc;

#[tokio::test]
async fn test_sender_rollup_contract_address_error() {
    let (mut mock_context, _mock_provider) = create_mock_context(None).await;

    // contract address error
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![],
        })
    });
    let handler = Arc::new(Box::new(mock_handler) as Box<_>);
    mock_context.handler = handler;
    let arc_mock_context = Arc::new(mock_context);
    let sender = RollupTxSender::builder().context(arc_mock_context).build();
    let address = "0xxx";
    let gas_price = U256::from(100000000_u64);
    let data = mock_proof_data().await;
    let proof_data = RollupProofData::builder()
        .pool_address(address.to_string())
        .rollup_size(1_usize)
        .max_gas_price(gas_price)
        .proof(data.clone())
        .next_rollup(false)
        .build();
    let result = sender.send(proof_data).await;
    assert!(matches!(
        result.err().unwrap(),
        RollerError::ConvertContractAddressError(_)
    ));
}

#[tokio::test]
async fn test_sender_rollup_success() {
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
    let sender = RollupTxSender::builder().context(arc_mock_context).build();

    // success
    let address = "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411";
    let gas_price = U256::from(100000000_u64);
    let data = mock_proof_data().await;
    let proof_data = RollupProofData::builder()
        .pool_address(address.to_string())
        .rollup_size(1_usize)
        .max_gas_price(gas_price)
        .proof(data)
        .next_rollup(false)
        .build();
    let result = sender.send(proof_data.clone()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sender_rollup_provider_error() {
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
    let mut receipt = mock_transaction_receipt_data();
    receipt.block_number = None;
    tx_manager.expect_send().returning(move |_, _| Ok(transaction.hash()));
    tx_manager.expect_confirm().returning(move |_, _| Ok(receipt.clone()));
    mock_context.handler = Arc::new(Box::new(mock_handler) as Box<_>);
    mock_context.tx = Arc::new(tx_manager);
    let arc_mock_context = Arc::new(mock_context);
    let sender = RollupTxSender::builder().context(arc_mock_context).build();

    // success
    let address = "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411";
    let gas_price = U256::from(100000000_u64);
    let data = mock_proof_data().await;
    let proof_data = RollupProofData::builder()
        .pool_address(address.to_string())
        .rollup_size(1_usize)
        .max_gas_price(gas_price)
        .proof(data)
        .next_rollup(false)
        .build();
    let result = sender.send(proof_data.clone()).await;
    assert!(matches!(result.err().unwrap(), RollerError::ProviderError(_)));
}
