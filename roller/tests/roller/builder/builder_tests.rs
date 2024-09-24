use crate::mock::create_mock_context;
use crate::mock::MockRollerHandler;
use crate::mock::MockRollerTokenPrice;
use crate::mock::MockRollerTxManager;
use ethers_core::types::Bytes;
use ethers_core::types::U256;
use mystiko_dataloader::data::LiteData;
use mystiko_dataloader::handler::QueryResult;
use mystiko_protos::data::v1::Commitment;
use mystiko_roller::common::RollerError;
use mystiko_roller::roller::RollupPlanData;
use mystiko_roller::roller::RollupTxBuilder;
use num_bigint::BigUint;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn test_builder_gas_price_error() {
    let address = "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411".to_string();
    let (mut mock_context, mock_provider) = create_mock_context(None).await;

    // empty commitments
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![],
        })
    });
    let handler = Arc::new(Box::new(mock_handler) as Box<_>);
    mock_context.handler = handler;
    let arc_mock_context = Arc::new(mock_context.clone());
    let builder = RollupTxBuilder::builder().context(arc_mock_context).build();
    let result = builder.build(address.clone()).await;
    assert!(result.is_ok());

    // provider get included count error
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![Commitment::builder().commitment_hash("123").build()],
        })
    });
    let handler = Arc::new(Box::new(mock_handler) as Box<_>);
    mock_context.handler = handler;
    let arc_mock_context = Arc::new(mock_context.clone());
    let builder = RollupTxBuilder::builder().context(arc_mock_context).build();
    let result = builder.build(address.clone()).await;
    assert!(result
        .err()
        .unwrap()
        .to_string()
        .contains("call contract func=get_commitment_included_count meet error"));

    // handler leaf index is none
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![Commitment::builder().commitment_hash("123").build()],
        })
    });
    let handler = Arc::new(Box::new(mock_handler) as Box<_>);
    mock_context.handler = handler;
    let include_count = Bytes::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();
    let arc_mock_context = Arc::new(mock_context.clone());
    let builder = RollupTxBuilder::builder().context(arc_mock_context).build();
    let result = builder.build(address.clone()).await;
    assert!(result
        .err()
        .unwrap()
        .to_string()
        .contains("roller internal error: handler commitment leaf_index is none"));

    // rollup fee error
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![Commitment::builder().commitment_hash("123").leaf_index(0).build()],
        })
    });
    let handler = Arc::new(Box::new(mock_handler) as Box<_>);
    mock_context.handler = handler;
    let include_count = Bytes::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();
    let arc_mock_context = Arc::new(mock_context.clone());
    let builder = RollupTxBuilder::builder().context(arc_mock_context).build();
    let result = builder.build(address.clone()).await;
    assert!(result
        .err()
        .unwrap()
        .to_string()
        .contains("roller internal error: handler commitment rollup fee is none"));

    // rollup get gas price from provider meet error
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![Commitment::builder()
                .commitment_hash("123")
                .rollup_fee(vec![1])
                .leaf_index(0)
                .build()],
        })
    });
    let mut token_price = MockRollerTokenPrice::new();
    token_price
        .expect_price()
        .returning(|_| Ok(f64::from_str("1000000000").unwrap()));
    token_price
        .expect_swap()
        .returning(|_, _, _, _, _| Ok(U256::from(1000000000_u64)));
    let gas_price = U256::from(1000000);
    let mut mock_tx_manager = MockRollerTxManager::new();
    mock_tx_manager.expect_gas_price().returning(move |_| Ok(gas_price));
    let handler = Arc::new(Box::new(mock_handler) as Box<_>);
    mock_context.handler = handler;
    mock_context.price = Arc::new(token_price);
    mock_context.tx = Arc::new(mock_tx_manager);
    let arc_mock_context = Arc::new(mock_context.clone());
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();
    let builder = RollupTxBuilder::builder().context(arc_mock_context).build();
    let result = builder.build(address.clone()).await;
    assert!(matches!(result.err().unwrap(), RollerError::ProviderError(_)));

    // rollup get block number from provider meet error
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![Commitment::builder()
                .commitment_hash("123")
                .leaf_index(0)
                .rollup_fee(vec![1])
                .build()],
        })
    });
    let mut mock_tx_manager = MockRollerTxManager::new();
    mock_tx_manager.expect_gas_price().returning(move |_| Ok(gas_price));
    let handler = Arc::new(Box::new(mock_handler) as Box<_>);
    mock_context.handler = handler;
    mock_context.tx = Arc::new(mock_tx_manager);
    let arc_mock_context = Arc::new(mock_context.clone());
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();
    let builder = RollupTxBuilder::builder().context(arc_mock_context).build();
    let result = builder.build(address.clone()).await;
    assert!(matches!(result.err().unwrap(), RollerError::ProviderError(_)));

    // rollup meet error gas price too high
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![Commitment::builder()
                .commitment_hash("123")
                .leaf_index(0)
                .rollup_fee(vec![1])
                .build()],
        })
    });
    let mut mock_tx_manager = MockRollerTxManager::new();
    mock_tx_manager.expect_gas_price().returning(move |_| Ok(gas_price));
    let block_number = U256::from(1);
    mock_provider.push(block_number).unwrap();
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();
    let handler = Arc::new(Box::new(mock_handler) as Box<_>);
    mock_context.handler = handler;
    mock_context.tx = Arc::new(mock_tx_manager);
    let arc_mock_context = Arc::new(mock_context.clone());
    let builder = RollupTxBuilder::builder().context(arc_mock_context).build();
    let result = builder.build(address.clone()).await;
    assert!(matches!(
        result.err().unwrap(),
        RollerError::CurrentGasPriceTooHighError(_)
    ));

    // force rollup meet error gas price too high
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![Commitment::builder()
                .commitment_hash("123")
                .leaf_index(0)
                .rollup_fee(vec![1])
                .build()],
        })
    });
    let gas_price = U256::from(10000000000000_u64);
    let mut mock_tx_manager = MockRollerTxManager::new();
    mock_tx_manager.expect_gas_price().returning(move |_| Ok(gas_price));
    let block_number = U256::from(1000);
    mock_provider.push(block_number).unwrap();
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();
    let handler = Arc::new(Box::new(mock_handler) as Box<_>);
    mock_context.handler = handler;
    mock_context.tx = Arc::new(mock_tx_manager);
    let arc_mock_context = Arc::new(mock_context.clone());
    let builder = RollupTxBuilder::builder().context(arc_mock_context).build();
    let result = builder.build(address.clone()).await;
    assert!(matches!(
        result.err().unwrap(),
        RollerError::CurrentGasPriceTooHighError(_)
    ));

    // do rollup
    let mut mock_handler = MockRollerHandler::<LiteData>::new();
    mock_handler.expect_query_commitments().returning(|_| {
        Ok(QueryResult {
            end_block: 0,
            result: vec![Commitment::builder()
                .commitment_hash("123")
                .leaf_index(0)
                .rollup_fee(vec![1])
                .build()],
        })
    });
    let gas_price = U256::from(100000000_u64);
    let mut mock_tx_manager = MockRollerTxManager::new();
    mock_tx_manager.expect_gas_price().returning(move |_| Ok(gas_price));
    mock_tx_manager.expect_tx_eip1559().returning(|| true);
    let block_number = U256::from(1000);
    mock_provider.push(block_number).unwrap();
    mock_provider.push::<Bytes, _>(include_count.clone()).unwrap();
    let handler = Arc::new(Box::new(mock_handler) as Box<_>);
    mock_context.handler = handler;
    mock_context.tx = Arc::new(mock_tx_manager);
    let arc_mock_context = Arc::new(mock_context.clone());
    let builder = RollupTxBuilder::builder().context(arc_mock_context).build();
    let result = builder.build(address.clone()).await;
    assert!(matches!(result.err().unwrap(), RollerError::ProtocolError(_)));
}

#[tokio::test]
async fn test_builder_calc_max_gas_price() {
    let address = "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411".to_string();
    let (mut mock_context, mock_provider) = create_mock_context(None).await;

    // send rollup, support 1559, use 2 * provider gas price
    let block_number = U256::from(1000);
    mock_provider.push(block_number).unwrap();

    let gas_price = U256::from(1_u64);
    let mut mock_tx_manager = MockRollerTxManager::new();
    mock_tx_manager.expect_gas_price().returning(move |_| Ok(gas_price));
    mock_tx_manager.expect_tx_eip1559().returning(|| true);
    mock_context.tx = Arc::new(mock_tx_manager);

    let mut token_price = MockRollerTokenPrice::new();
    token_price
        .expect_price()
        .returning(|_| Ok(f64::from_str("1000000000").unwrap()));
    token_price
        .expect_swap()
        .returning(|_, _, _, _, _| Ok(U256::from(1_000_000_u64)));
    mock_context.price = Arc::new(token_price);

    let arc_mock_context = Arc::new(mock_context.clone());
    let builder = RollupTxBuilder::builder().context(arc_mock_context.clone()).build();
    let plan = RollupPlanData::builder()
        .pool_address(address.clone())
        .total(1)
        .sizes(vec![1])
        .cms(vec![Commitment::builder()
            .block_number(1000_u64)
            .commitment_hash("123")
            .rollup_fee(vec![1])
            .build()])
        .build();
    let result = builder.calc_max_gas_price(&plan).await;
    assert_eq!(result.unwrap(), U256::from(2_u64));

    // send rollup, not support 1559, use provider gas price
    let block_number = U256::from(1000);
    mock_provider.push(block_number).unwrap();
    let mut mock_tx_manager = MockRollerTxManager::new();
    mock_tx_manager.expect_gas_price().returning(move |_| Ok(gas_price));
    mock_tx_manager.expect_tx_eip1559().returning(|| false);
    mock_context.tx = Arc::new(mock_tx_manager);

    let arc_mock_context = Arc::new(mock_context.clone());
    let builder = RollupTxBuilder::builder().context(arc_mock_context.clone()).build();
    let result = builder.calc_max_gas_price(&plan).await;
    assert_eq!(result.unwrap(), U256::from(1_u64));

    // send rollup, support 1559, use plan gas price
    let block_number = U256::from(1000);
    mock_provider.push(block_number).unwrap();

    let gas_price = U256::from(2_u64);
    let mut mock_tx_manager = MockRollerTxManager::new();
    mock_tx_manager.expect_gas_price().returning(move |_| Ok(gas_price));
    mock_tx_manager.expect_tx_eip1559().returning(|| true);
    mock_context.tx = Arc::new(mock_tx_manager);

    let mut token_price = MockRollerTokenPrice::new();
    token_price
        .expect_price()
        .returning(|_| Ok(f64::from_str("1000000000").unwrap()));
    token_price
        .expect_swap()
        .returning(|_, _, _, _, _| Ok(U256::from(1_000_000_u64)));
    mock_context.price = Arc::new(token_price);

    let arc_mock_context = Arc::new(mock_context.clone());
    let builder = RollupTxBuilder::builder().context(arc_mock_context.clone()).build();
    let plan = RollupPlanData::builder()
        .pool_address(address.clone())
        .total(1)
        .sizes(vec![1])
        .cms(vec![Commitment::builder()
            .block_number(1000_u64)
            .commitment_hash("123")
            .rollup_fee(vec![1])
            .build()])
        .build();
    let result = builder.calc_max_gas_price(&plan).await;
    assert_eq!(result.unwrap(), U256::from(3_u64));
}

#[tokio::test]
async fn test_builder_force_calc_max_gas_price() {
    let address = "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411".to_string();
    let (mut mock_context, mock_provider) = create_mock_context(None).await;

    // force send rollup, support 1559, use 2 * provider gas price
    let block_number = U256::from(1000);
    mock_provider.push(block_number).unwrap();

    let gas_price = U256::from(1_u64);
    let mut mock_tx_manager = MockRollerTxManager::new();
    mock_tx_manager.expect_gas_price().returning(move |_| Ok(gas_price));
    mock_tx_manager.expect_tx_eip1559().returning(|| true);
    mock_context.tx = Arc::new(mock_tx_manager);

    let mut token_price = MockRollerTokenPrice::new();
    token_price
        .expect_price()
        .returning(|_| Ok(f64::from_str("1000000000").unwrap()));
    token_price
        .expect_swap()
        .returning(|_, _, _, _, _| Ok(U256::from(1000000_u64)));
    mock_context.price = Arc::new(token_price);

    let arc_mock_context = Arc::new(mock_context.clone());
    let builder = RollupTxBuilder::builder().context(arc_mock_context.clone()).build();
    let plan = RollupPlanData::builder()
        .pool_address(address.clone())
        .total(1)
        .sizes(vec![1])
        .cms(vec![Commitment::builder()
            .commitment_hash("123")
            .block_number(1_u64)
            .rollup_fee(vec![1])
            .build()])
        .build();
    let result = builder.calc_max_gas_price(&plan).await;
    assert_eq!(result.unwrap(), U256::from(2_u64));

    // swap 0, return current provider gas price
    let mut token_price = MockRollerTokenPrice::new();
    token_price
        .expect_price()
        .returning(|_| Ok(f64::from_str("1000000000").unwrap()));
    token_price.expect_swap().returning(|_, _, _, _, _| Ok(U256::from(0)));
    mock_context.price = Arc::new(token_price);

    let arc_mock_context = Arc::new(mock_context.clone());
    let builder = RollupTxBuilder::builder().context(arc_mock_context.clone()).build();
    let plan = RollupPlanData::builder()
        .pool_address(address.clone())
        .total(1)
        .sizes(vec![1])
        .cms(vec![Commitment::builder()
            .commitment_hash("123")
            .block_number(1_u64)
            .rollup_fee(vec![1])
            .build()])
        .build();
    let result = builder.calc_max_gas_price(&plan).await;
    assert_eq!(result.unwrap(), U256::from(1_u64));

    // force send rollup, not support 1559, use provider gas price
    let block_number = U256::from(1000);
    mock_provider.push(block_number).unwrap();
    let mut mock_tx_manager = MockRollerTxManager::new();
    mock_tx_manager.expect_gas_price().returning(move |_| Ok(gas_price));
    mock_tx_manager.expect_tx_eip1559().returning(|| false);
    mock_context.tx = Arc::new(mock_tx_manager);

    let arc_mock_context = Arc::new(mock_context.clone());
    let builder = RollupTxBuilder::builder().context(arc_mock_context.clone()).build();
    let result = builder.calc_max_gas_price(&plan).await;
    assert_eq!(result.unwrap(), U256::from(1_u64));
}

#[tokio::test]
async fn test_builder_generate_proof_error() {
    let (mock_context, _mock_provider) = create_mock_context(None).await;
    let arc_mock_context = Arc::new(mock_context);
    let builder = RollupTxBuilder::builder().context(arc_mock_context).build();

    let address = "0xxxx";
    let cm_hash = BigUint::from_str("1000000000000000000").unwrap();
    let result = builder.generate_proof(address, vec![cm_hash.clone()], 1_u64).await;
    assert!(matches!(
        result.err().unwrap(),
        RollerError::PoolContractConfigNotFoundError(_)
    ));
}

#[tokio::test]
async fn test_builder_merkle_tree() {
    let (mut mock_context, _mock_provider) = create_mock_context(None).await;

    // empty commitments
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
    let builder = RollupTxBuilder::builder().context(arc_mock_context).build();
    let address = "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411";
    let cm_hash = BigUint::from_str("1000000000000000000").unwrap();
    let result = builder.generate_proof(address, vec![cm_hash.clone()], 0_u64).await;
    assert!(matches!(result.err().unwrap(), RollerError::ProtocolError(_)));
}

#[tokio::test]
async fn test_builder_merkle_tree_handler_data_error() {
    let (mut mock_context, _mock_provider) = create_mock_context(None).await;

    // empty commitments
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
    let builder = RollupTxBuilder::builder().context(arc_mock_context).build();
    let address = "0x932f3DD5b6C0F5fe1aEc31Cb38B7a57d01496411";
    let cm_hash = BigUint::from_str("1000000000000000000").unwrap();
    let result = builder.generate_proof(address, vec![cm_hash.clone()], 1_u64).await;
    assert!(matches!(result.err().unwrap(), RollerError::RollerInternalError(_)));
}
