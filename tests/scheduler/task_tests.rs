use crate::mock::create_mock_context;
use mystiko_roller::scheduler::task::RollerTask;
use std::sync::Arc;

#[tokio::test]
async fn test_roller_task_run() {
    let (mock_context, _mock_provider) =
        create_mock_context(Some("./tests/test_files/config/config_fetcher_provider")).await;
    let roller_task = RollerTask::new(Arc::new(mock_context)).await.unwrap();
    let result = roller_task.run(&None).await;
    assert!(result.is_err());
}
