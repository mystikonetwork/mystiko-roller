use crate::mock::create_mock_context;
use crate::mock::create_mock_providers;
use crate::mock::mock_proof_data;
use crate::mock::{create_mock_env_config, create_mock_mystiko_config, create_mock_roller_config};
use crate::mock::{mock_transaction_data, mock_transaction_receipt_data};

#[tokio::test]
async fn test_create_roller_mock() {
    let env = create_mock_env_config();
    let _ = create_mock_roller_config(&env).await;
    let _ = create_mock_mystiko_config().await;
    let _ = create_mock_context(None).await;
    let _ = mock_proof_data();
    let _ = create_mock_providers();
    let _ = mock_transaction_data();
    let _ = mock_transaction_receipt_data();
}
