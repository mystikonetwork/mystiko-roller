use crate::mock::create_mock_context;
use mystiko_roller::loader::RollerChainDataLoader;
use std::sync::Arc;

#[tokio::test]
async fn test_handler() {
    let (mock_context, _) = create_mock_context(None).await;
    let loader = RollerChainDataLoader::from_config(Arc::new(mock_context)).await;
    assert!(loader.is_ok());
}
