use anyhow::Result;
use async_trait::async_trait;
use ethers_core::types::{Bytes, U64};
use ethers_providers::{MockError, MockProvider, RetryClientBuilder, RetryPolicy};
use mockall::mock;
use mystiko_config::MystikoConfig;
use mystiko_ethers::Provider;
use mystiko_ethers::{FailoverProvider, ProviderWrapper, Providers};
use mystiko_grpc::GrpcServer;
use mystiko_notification::SnsNotification;
use mystiko_protos::data::v1::Commitment;
use mystiko_protos::sequencer::v1::sequencer_service_server::SequencerServiceServer;
use mystiko_protos::sequencer::v1::{
    ChainLoadedBlockRequest, ChainLoadedBlockResponse, ContractLoadedBlockRequest, ContractLoadedBlockResponse,
    FetchChainRequest, FetchChainResponse, GetCommitmentsRequest, GetCommitmentsResponse, GetNullifiersRequest,
    GetNullifiersResponse, HealthCheckRequest, HealthCheckResponse,
};
use mystiko_protos::service::v1::ServerOptions;
use mystiko_roller_monitor::{start_monitor, start_monitor_with_config, RollerMonitor, RollerMonitorConfig};
use rusoto_core::RusotoError;
use rusoto_sns::*;
use serial_test::serial;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct TestProvders;

#[async_trait]
impl Providers for TestProvders {
    async fn get_provider(&self, chain_id: u64) -> Result<Arc<Provider>> {
        let test_provider = MockProvider::default();
        if chain_id == 1 {
            Ok(Arc::new(mock_chain_1(test_provider)))
        } else if chain_id == 56 {
            Ok(Arc::new(mock_chain_56(test_provider)))
        } else {
            Err(anyhow::anyhow!("provider not found!"))
        }
    }
    async fn has_provider(&self, _chain_id: u64) -> bool {
        true
    }

    async fn set_provider(&self, _chain_id: u64, _provider: Arc<Provider>) -> Option<Arc<Provider>> {
        None
    }

    async fn delete_provider(&self, _chain_id: u64) -> Option<Arc<Provider>> {
        None
    }
}

mock! {
  #[derive(Debug)]
  SnsClient{}

  #[async_trait]
  impl Sns for SnsClient {
    async fn add_permission(
      &self,
      input: AddPermissionInput,
  ) -> Result<(), RusotoError<AddPermissionError>>;

  async fn check_if_phone_number_is_opted_out(
    &self,
    input: CheckIfPhoneNumberIsOptedOutInput,
  ) -> Result<CheckIfPhoneNumberIsOptedOutResponse, RusotoError<CheckIfPhoneNumberIsOptedOutError>>;

  async fn confirm_subscription(
    &self,
    input: ConfirmSubscriptionInput,
  ) -> Result<ConfirmSubscriptionResponse, RusotoError<ConfirmSubscriptionError>>;

  async fn create_platform_application(
    &self,
    input: CreatePlatformApplicationInput,
  ) -> Result<CreatePlatformApplicationResponse, RusotoError<CreatePlatformApplicationError>>;

  async fn create_platform_endpoint(
    &self,
    input: CreatePlatformEndpointInput,
  ) -> Result<CreateEndpointResponse, RusotoError<CreatePlatformEndpointError>>;

  async fn create_sms_sandbox_phone_number(
    &self,
    input: CreateSMSSandboxPhoneNumberInput,
  ) -> Result<CreateSMSSandboxPhoneNumberResult, RusotoError<CreateSMSSandboxPhoneNumberError>>;

  async fn create_topic(
    &self,
    input: CreateTopicInput,
  ) -> Result<CreateTopicResponse, RusotoError<CreateTopicError>>;

  async fn delete_endpoint(
    &self,
    input: DeleteEndpointInput,
  ) -> Result<(), RusotoError<DeleteEndpointError>>;
  async fn delete_platform_application(
    &self,
    input: DeletePlatformApplicationInput,
  ) -> Result<(), RusotoError<DeletePlatformApplicationError>>;

  async fn delete_sms_sandbox_phone_number(
    &self,
    input: DeleteSMSSandboxPhoneNumberInput,
  ) -> Result<DeleteSMSSandboxPhoneNumberResult, RusotoError<DeleteSMSSandboxPhoneNumberError>>;

  async fn delete_topic(
    &self,
    input: DeleteTopicInput,
  ) -> Result<(), RusotoError<DeleteTopicError>>;
  async fn get_endpoint_attributes(
    &self,
    input: GetEndpointAttributesInput,
  ) -> Result<GetEndpointAttributesResponse, RusotoError<GetEndpointAttributesError>>;

  async fn get_platform_application_attributes(
    &self,
    input: GetPlatformApplicationAttributesInput,
  ) -> Result<GetPlatformApplicationAttributesResponse, RusotoError<GetPlatformApplicationAttributesError>>;

  async fn get_sms_attributes(
    &self,
    input: GetSMSAttributesInput,
  ) -> Result<GetSMSAttributesResponse, RusotoError<GetSMSAttributesError>>;

  async fn get_sms_sandbox_account_status(
    &self,
    input: GetSMSSandboxAccountStatusInput,
  ) -> Result<GetSMSSandboxAccountStatusResult, RusotoError<GetSMSSandboxAccountStatusError>>;

  async fn get_subscription_attributes(
    &self,
    input: GetSubscriptionAttributesInput,
  ) -> Result<GetSubscriptionAttributesResponse, RusotoError<GetSubscriptionAttributesError>>;
  async fn get_topic_attributes(
    &self,
    input: GetTopicAttributesInput,
  ) -> Result<GetTopicAttributesResponse, RusotoError<GetTopicAttributesError>>;

  async fn list_endpoints_by_platform_application(
    &self,
    input: ListEndpointsByPlatformApplicationInput,
  ) -> Result<ListEndpointsByPlatformApplicationResponse, RusotoError<ListEndpointsByPlatformApplicationError>>;

  async fn list_origination_numbers(
    &self,
    input: ListOriginationNumbersRequest,
  ) -> Result<ListOriginationNumbersResult, RusotoError<ListOriginationNumbersError>>;

  async fn list_phone_numbers_opted_out(
    &self,
    input: ListPhoneNumbersOptedOutInput,
  ) -> Result<ListPhoneNumbersOptedOutResponse, RusotoError<ListPhoneNumbersOptedOutError>>;

  async fn list_platform_applications(
    &self,
    input: ListPlatformApplicationsInput,
  ) -> Result<ListPlatformApplicationsResponse, RusotoError<ListPlatformApplicationsError>>;

  async fn list_sms_sandbox_phone_numbers(
    &self,
    input: ListSMSSandboxPhoneNumbersInput,
  ) -> Result<ListSMSSandboxPhoneNumbersResult, RusotoError<ListSMSSandboxPhoneNumbersError>>;

  async fn list_subscriptions(
    &self,
    input: ListSubscriptionsInput,
  ) -> Result<ListSubscriptionsResponse, RusotoError<ListSubscriptionsError>>;

  async fn list_subscriptions_by_topic(
    &self,
    input: ListSubscriptionsByTopicInput,
  ) -> Result<ListSubscriptionsByTopicResponse, RusotoError<ListSubscriptionsByTopicError>>;

  async fn list_tags_for_resource(
    &self,
    input: ListTagsForResourceRequest,
  ) -> Result<ListTagsForResourceResponse, RusotoError<ListTagsForResourceError>>;

  async fn list_topics(
    &self,
    input: ListTopicsInput,
  ) -> Result<ListTopicsResponse, RusotoError<ListTopicsError>>;

  async fn opt_in_phone_number(
    &self,
    input: OptInPhoneNumberInput,
  ) -> Result<OptInPhoneNumberResponse, RusotoError<OptInPhoneNumberError>>;

  async fn publish(
    &self,
    input: PublishInput,
  ) -> Result<PublishResponse, RusotoError<PublishError>>;

  async fn remove_permission(
    &self,
    input: RemovePermissionInput,
  ) -> Result<(), RusotoError<RemovePermissionError>>;

  async fn set_endpoint_attributes(
    &self,
    input: SetEndpointAttributesInput,
  ) -> Result<(), RusotoError<SetEndpointAttributesError>>;

  async fn set_platform_application_attributes(
    &self,
    input: SetPlatformApplicationAttributesInput,
  ) -> Result<(), RusotoError<SetPlatformApplicationAttributesError>>;

  async fn set_sms_attributes(
    &self,
    input: SetSMSAttributesInput,
  ) -> Result<SetSMSAttributesResponse, RusotoError<SetSMSAttributesError>>;

  async fn set_subscription_attributes(
    &self,
    input: SetSubscriptionAttributesInput,
  ) -> Result<(), RusotoError<SetSubscriptionAttributesError>>;

  async fn set_topic_attributes(
    &self,
    input: SetTopicAttributesInput,
  ) -> Result<(), RusotoError<SetTopicAttributesError>>;

  async fn subscribe(
    &self,
    input: SubscribeInput,
  ) -> Result<SubscribeResponse, RusotoError<SubscribeError>>;

  async fn tag_resource(
    &self,
    input: TagResourceRequest,
  ) -> Result<TagResourceResponse, RusotoError<TagResourceError>>;

  async fn unsubscribe(
    &self,
    input: UnsubscribeInput,
  ) -> Result<(), RusotoError<UnsubscribeError>>;

  async fn untag_resource(
    &self,
    input: UntagResourceRequest,
  ) -> Result<UntagResourceResponse, RusotoError<UntagResourceError>>;

  async fn verify_sms_sandbox_phone_number(
    &self,
    input: VerifySMSSandboxPhoneNumberInput,
  ) -> Result<VerifySMSSandboxPhoneNumberResult, RusotoError<VerifySMSSandboxPhoneNumberError>>;

  }
}

mock! {
  #[derive(Debug)]
  SequencerService {}
  #[async_trait]
  impl mystiko_protos::sequencer::v1::sequencer_service_server::SequencerService for SequencerService {
      async fn fetch_chain(&self, _request: Request<FetchChainRequest>,) -> Result<Response<FetchChainResponse>,Status>;
      async fn chain_loaded_block(&self, _chain_id: Request<ChainLoadedBlockRequest>) -> Result<Response<ChainLoadedBlockResponse>,Status>;
      async fn contract_loaded_block(&self, _request: Request<ContractLoadedBlockRequest>,) -> Result<Response<ContractLoadedBlockResponse>,Status>;
      async fn get_commitments(&self, _request: tonic::Request<GetCommitmentsRequest>) -> Result<Response<GetCommitmentsResponse>, Status>;
      async fn get_nullifiers(&self, _request: tonic::Request<GetNullifiersRequest>) -> Result<Response<GetNullifiersResponse>, Status>;
      async fn health_check(&self, _request: Request<HealthCheckRequest>,) -> Result<Response<HealthCheckResponse>,Status>;

  }
}

#[tokio::test]
#[serial]
async fn test_from_config() {
    let monitor_config = RollerMonitorConfig::new(Some("./tests/files/monitor_test.json".into())).unwrap();
    let client = MockSnsClient::new();
    let mock_notification = Arc::new(SnsNotification::<MockSnsClient>::builder().client(client).build());
    let mock_providers = Arc::new(TestProvders);
    let mystiko_config = Arc::new(
        MystikoConfig::from_options(monitor_config.mystiko.clone())
            .await
            .unwrap(),
    );
    let monitor_result = RollerMonitor::<PublishInput, SnsNotification<MockSnsClient>, TestProvders>::from_config(
        Arc::new(monitor_config.clone()),
        mystiko_config,
        mock_providers.clone(),
        mock_notification.clone(),
    )
    .await;
    assert!(monitor_result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_start_monitor_error() {
    let scheduler_result = start_monitor(Some("./tests/files/start_monitor_test.json".into())).await;
    assert!(scheduler_result.is_ok());
    let monitor_scheduler = scheduler_result.unwrap();
    let stop_result = monitor_scheduler.stop().await;
    assert!(stop_result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_start_monitor_with_config_success() {
    let mut service = MockSequencerService::new();
    service
        .expect_get_commitments()
        .withf(|req| req.get_ref().chain_id == 1)
        .returning(move |_| {
            Ok(Response::new(
                GetCommitmentsResponse::builder()
                    .chain_id(1_u32)
                    .contract_address("123".to_string())
                    .commitments(vec![Commitment::builder().block_number(10000_u64).build()])
                    .build(),
            ))
        });
    let options = ServerOptions::builder()
        .port(21111u32)
        .accept_http1(true)
        .enable_web(true)
        .build();
    let mut server = setup_grpc_server(service, options.clone()).await;
    let mock_providers = Arc::new(TestProvders);
    let client = MockSnsClient::new();
    let mock_notification = Arc::new(SnsNotification::<MockSnsClient>::builder().client(client).build());
    let monitor_config = RollerMonitorConfig::new(Some("./tests/files/monitor_one_chain_test.json".into())).unwrap();
    let mystiko_config = Arc::new(
        MystikoConfig::from_options(monitor_config.mystiko.clone())
            .await
            .unwrap(),
    );
    let monitor_scheduler_result =
        start_monitor_with_config::<PublishInput, SnsNotification<MockSnsClient>, TestProvders>(
            Arc::new(monitor_config),
            mystiko_config,
            mock_providers.clone(),
            mock_notification.clone(),
        )
        .await;
    assert!(monitor_scheduler_result.is_ok());
    let monitor_scheduler = monitor_scheduler_result.unwrap();
    tokio::time::sleep(Duration::from_secs(7)).await;
    let stop_result = monitor_scheduler.stop().await;
    assert!(stop_result.is_ok());
    server.stop().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_start_monitor_with_notify_success() {
    let mut service = MockSequencerService::new();
    service
        .expect_get_commitments()
        .withf(|req| req.get_ref().chain_id == 1)
        .returning(move |_| {
            Ok(Response::new(
                GetCommitmentsResponse::builder()
                    .chain_id(1_u32)
                    .contract_address("123".to_string())
                    .commitments(vec![Commitment::builder().block_number(100_u64).build()])
                    .build(),
            ))
        });
    let options = ServerOptions::builder()
        .port(21111u32)
        .accept_http1(true)
        .enable_web(true)
        .build();
    let mut server = setup_grpc_server(service, options.clone()).await;
    let mock_providers = Arc::new(TestProvders);
    let mut client = MockSnsClient::new();
    client
        .expect_publish()
        .withf(|input| input.topic_arn.as_ref().unwrap() == "test_topic")
        .returning(|_| Ok(PublishResponse::default()));
    let mock_notification = Arc::new(SnsNotification::<MockSnsClient>::builder().client(client).build());
    let monitor_config = RollerMonitorConfig::new(Some("./tests/files/monitor_one_chain_test.json".into())).unwrap();
    let mystiko_config = Arc::new(
        MystikoConfig::from_options(monitor_config.mystiko.clone())
            .await
            .unwrap(),
    );
    let monitor_scheduler_result =
        start_monitor_with_config::<PublishInput, SnsNotification<MockSnsClient>, TestProvders>(
            Arc::new(monitor_config),
            mystiko_config,
            mock_providers.clone(),
            mock_notification.clone(),
        )
        .await;
    assert!(monitor_scheduler_result.is_ok());
    let monitor_scheduler = monitor_scheduler_result.unwrap();
    tokio::time::sleep(Duration::from_secs(7)).await;
    let stop_result = monitor_scheduler.stop().await;
    assert!(stop_result.is_ok());
    server.stop().await.unwrap();
}

#[tokio::test]
#[serial]
async fn test_start_monitor_with_notify_error() {
    let mut service = MockSequencerService::new();
    service
        .expect_get_commitments()
        .withf(|req| req.get_ref().chain_id == 1)
        .returning(move |_| {
            Ok(Response::new(
                GetCommitmentsResponse::builder()
                    .chain_id(1_u32)
                    .contract_address("123".to_string())
                    .commitments(vec![Commitment::builder().block_number(100_u64).build()])
                    .build(),
            ))
        });
    let options = ServerOptions::builder()
        .port(21111u32)
        .accept_http1(true)
        .enable_web(true)
        .build();
    let mut server = setup_grpc_server(service, options.clone()).await;
    let mock_providers = Arc::new(TestProvders);
    let mut client = MockSnsClient::new();
    client
        .expect_publish()
        .withf(|input| input.topic_arn.as_ref().unwrap() == "test_topic")
        .returning(|_| {
            Err(RusotoError::Service(PublishError::NotFound(
                "NotFoundError".to_string(),
            )))
        });
    let mock_notification = Arc::new(SnsNotification::<MockSnsClient>::builder().client(client).build());
    let monitor_config = RollerMonitorConfig::new(Some("./tests/files/monitor_one_chain_test.json".into())).unwrap();
    let mystiko_config = Arc::new(
        MystikoConfig::from_options(monitor_config.mystiko.clone())
            .await
            .unwrap(),
    );
    let monitor_scheduler_result =
        start_monitor_with_config::<PublishInput, SnsNotification<MockSnsClient>, TestProvders>(
            Arc::new(monitor_config),
            mystiko_config,
            mock_providers.clone(),
            mock_notification.clone(),
        )
        .await;
    assert!(monitor_scheduler_result.is_ok());
    let monitor_scheduler = monitor_scheduler_result.unwrap();
    tokio::time::sleep(Duration::from_secs(7)).await;
    let stop_result = monitor_scheduler.stop().await;
    assert!(stop_result.is_ok());
    server.stop().await.unwrap();
}

async fn setup_grpc_server(service: MockSequencerService, options: ServerOptions) -> GrpcServer {
    let mut server = GrpcServer::default();
    server
        .start(SequencerServiceServer::new(service), options)
        .await
        .unwrap();
    server
}

#[derive(Debug, Default)]
struct MockProviderRetryPolicy;

impl RetryPolicy<MockError> for MockProviderRetryPolicy {
    fn should_retry(&self, _error: &MockError) -> bool {
        false
    }

    fn backoff_hint(&self, _error: &MockError) -> Option<Duration> {
        Duration::from_secs(1).into()
    }
}

fn mock_chain_1(test_provider: MockProvider) -> Provider {
    let block_number = U64::from(10000_u64);
    test_provider.push(block_number).unwrap();

    // first line offset of data , second line count of data
    let cms = Bytes::from_str(concat!(
        "0000000000000000000000000000000000000000000000000000000000000020",
        "0000000000000000000000000000000000000000000000000000000000000003",
        "0000000000000000000000000000000000000000000000000000000000000001",
        "0000000000000000000000000000000000000000000000000000000000000002",
        "0000000000000000000000000000000000000000000000000000000000000003"
    ))
    .unwrap();
    test_provider.push::<Bytes, _>(cms.clone()).unwrap();
    create_mock_provider(&test_provider)
}

fn mock_chain_56(test_provider: MockProvider) -> Provider {
    create_mock_provider(&test_provider)
}

fn create_mock_provider(provider: &MockProvider) -> Provider {
    let retry_provider_builder = RetryClientBuilder::default();
    let inner_provider = retry_provider_builder.build(provider.clone(), Box::<MockProviderRetryPolicy>::default());
    let mut provider_builder = FailoverProvider::dyn_rpc();
    provider_builder = provider_builder.add_provider(Box::new(inner_provider));
    Provider::new(ProviderWrapper::new(Box::new(provider_builder.build())))
}
