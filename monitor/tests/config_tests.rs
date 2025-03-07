use mystiko_config::MystikoConfigOptions;
use mystiko_protos::service::v1::ClientOptions;
use mystiko_roller_monitor::{
    ChainMonitorConfig, DefaultRetryPolicy, NotificationConfig, RollerMonitorConfig, RollerMonitorError,
    SchedulerConfig,
};
use mystiko_scheduler::RetryPolicy;
use serial_test::serial;

#[test]
#[serial]
fn test_default_config() {
    let mut config = RollerMonitorConfig::default();
    assert_eq!(config.logging_level, "info");
    assert_eq!(config.extern_logging_level, "warn");
    assert_eq!(config.notification.topic_arn, None);
    assert_eq!(config.notification.region, "".to_string());
    let chain_config = ChainMonitorConfig::builder().max_rollup_delay_block(1000_u64).build();
    assert_eq!(chain_config.max_rollup_delay_block, 1000_u64);
    config.chains.insert(1_u64, chain_config);
    assert_eq!(config.chains.len(), 1);
    let notification_config = NotificationConfig::builder().region(String::from("region")).build();
    assert_eq!(notification_config.region, String::from("region"));
    assert_eq!(notification_config.topic_arn, None);
    let default_sequencer_options = ClientOptions::default();
    let test_sequencer_options = config.clone().sequencer;
    assert_eq!(test_sequencer_options.host, default_sequencer_options.host);
    assert_eq!(test_sequencer_options.port, default_sequencer_options.port);
    assert_eq!(test_sequencer_options.is_ssl, default_sequencer_options.is_ssl);
    assert_eq!(test_sequencer_options.ssl_cert, default_sequencer_options.ssl_cert);
    assert_eq!(
        test_sequencer_options.ssl_cert_path,
        default_sequencer_options.ssl_cert_path
    );
    assert_eq!(
        test_sequencer_options.ssl_server_name,
        default_sequencer_options.ssl_server_name
    );
    let default_mystiko_options = MystikoConfigOptions::default();
    let test_mystiko_options = config.clone().mystiko;
    assert_eq!(test_mystiko_options.file_path, default_mystiko_options.file_path);
    assert_eq!(test_mystiko_options.is_testnet, default_mystiko_options.is_testnet);
    assert_eq!(test_mystiko_options.is_staging, default_mystiko_options.is_staging);
    assert_eq!(test_mystiko_options.git_revision, default_mystiko_options.git_revision);
    assert_eq!(
        test_mystiko_options.remote_base_url,
        default_mystiko_options.remote_base_url
    );
    let scheduler_config = SchedulerConfig::default();
    assert_eq!(scheduler_config.interval_ms, 7_200_000_u64);
    assert!(!scheduler_config.no_retry_on_timeout);
    assert_eq!(scheduler_config.task_timeout_ms, Some(60_000_u64));
    assert_eq!(scheduler_config.max_retry_times, 0);
    assert_eq!(scheduler_config.status_server_bind_address, "0.0.0.0");
    assert_eq!(scheduler_config.status_server_port, 21828);
    assert_eq!(config.scheduler, scheduler_config);
    let config = config.clone();
    assert_eq!(config.logging_level, "info");
    assert_eq!(config.extern_logging_level, "warn");
}

#[test]
#[serial]
fn test_config_from_file() {
    let config = RollerMonitorConfig::new(Some("./tests/files/config_test.json".into())).unwrap();
    check_config(config);
}

#[test]
#[serial]
fn test_config_from_env() {
    let env_variables = vec![
        ("MYSTIKO_ROLLER_MONITOR.LOGGING_LEVEL", "warn"),
        ("MYSTIKO_ROLLER_MONITOR.EXTERN_LOGGING_LEVEL", "error"),
        ("MYSTIKO_ROLLER_MONITOR.NOTIFICATION.TOPIC_ARN", "test_topic"),
        ("MYSTIKO_ROLLER_MONITOR.NOTIFICATION.REGION", "test_region"),
        ("MYSTIKO_ROLLER_MONITOR.CHAINS.1.MAX_ROLLUP_DELAY_BLOCK", "1000"),
        ("MYSTIKO_ROLLER_MONITOR.CHAINS.2.MAX_ROLLUP_DELAY_BLOCK", "2000"),
        ("MYSTIKO_ROLLER_MONITOR.SEQUENCER.HOST", "0.0.0.0"),
        ("MYSTIKO_ROLLER_MONITOR.SEQUENCER.PORT", "21111"),
        ("MYSTIKO_ROLLER_MONITOR.SEQUENCER.IS_SSL", "false"),
        ("MYSTIKO_ROLLER_MONITOR.SEQUENCER.SSL_CERT", "test_ssl_cert"),
        ("MYSTIKO_ROLLER_MONITOR.SEQUENCER.SSL_CERT_PATH", "test_ssl_cert_path"),
        (
            "MYSTIKO_ROLLER_MONITOR.SEQUENCER.SSL_SERVER_NAME",
            "test_ssl_server_name",
        ),
        ("MYSTIKO_ROLLER_MONITOR.MYSTIKO.FILE_PATH", "test_file_path"),
        ("MYSTIKO_ROLLER_MONITOR.MYSTIKO.IS_TESTNET", "true"),
        ("MYSTIKO_ROLLER_MONITOR.MYSTIKO.IS_STAGING", "false"),
        ("MYSTIKO_ROLLER_MONITOR.MYSTIKO.REMOTE_BASE_URL", "test_remote_base_url"),
        ("MYSTIKO_ROLLER_MONITOR.MYSTIKO.GIT_REVISION", "test_git_revision"),
        ("MYSTIKO_ROLLER_MONITOR.SCHEDULER.INTERVAL_MS", "120000"),
        ("MYSTIKO_ROLLER_MONITOR.SCHEDULER.TASK_TIMEOUT_MS", "1200000"),
        ("MYSTIKO_ROLLER_MONITOR.SCHEDULER.NO_RETRY_ON_TIMEOUT", "true"),
        ("MYSTIKO_ROLLER_MONITOR.SCHEDULER.MAX_RETRY_TIMES", "0"),
        ("MYSTIKO_ROLLER_MONITOR.SCHEDULER.MAX_RETRY_TIMES", "2"),
        ("MYSTIKO_ROLLER_MONITOR.SCHEDULER.STATUS_SERVER_BIND_ADDRESS", "0.0.0.0"),
        ("MYSTIKO_ROLLER_MONITOR.SCHEDULER.STATUS_SERVER_PORT", "21828"),
    ];
    for (key, value) in env_variables.iter() {
        std::env::set_var(key, value);
    }
    let config = RollerMonitorConfig::new(None);
    for (key, _) in env_variables.iter() {
        std::env::remove_var(key);
    }
    check_config(config.unwrap());
}

fn check_config(config: RollerMonitorConfig) {
    assert_eq!(config.logging_level, "warn");
    assert_eq!(config.extern_logging_level, "error");
    assert_eq!(config.notification.topic_arn, Some("test_topic".to_string()));
    assert_eq!(config.notification.region, "test_region");
    let chains = config.chains;
    assert_eq!(chains.len(), 2);
    assert_eq!(chains.get(&1_u64).unwrap().max_rollup_delay_block, 1000);
    assert_eq!(chains.get(&2_u64).unwrap().max_rollup_delay_block, 2000);
    let mystiko = config.mystiko;
    assert_eq!(mystiko.file_path, Some("test_file_path".to_string()));
    assert!(mystiko.is_testnet);
    assert!(!mystiko.is_staging);
    assert_eq!(mystiko.git_revision, Some("test_git_revision".to_string()));
    assert_eq!(mystiko.remote_base_url, Some("test_remote_base_url".to_string()));

    let scheduler_config = config.scheduler;
    assert_eq!(scheduler_config.interval_ms, 120000);
    assert!(scheduler_config.no_retry_on_timeout);
    assert_eq!(scheduler_config.task_timeout_ms, Some(1200000));
    assert_eq!(scheduler_config.max_retry_times, 2);
    assert_eq!(scheduler_config.status_server_bind_address, "0.0.0.0".to_string());
    assert_eq!(scheduler_config.status_server_port, 21828);

    let sequencer = config.sequencer;
    assert_eq!(sequencer.host(), "0.0.0.0");
    assert_eq!(sequencer.port(), 21111);
    assert!(!sequencer.is_ssl());
    assert_eq!(sequencer.ssl_cert(), "test_ssl_cert");
    assert_eq!(sequencer.ssl_cert_path(), "test_ssl_cert_path");
    assert_eq!(sequencer.ssl_server_name(), "test_ssl_server_name");
}

#[test]
fn test_default_policy() {
    let retry_policy = DefaultRetryPolicy;
    let error = RollerMonitorError::AnyhowError(anyhow::anyhow!("AnyhowError"));
    assert!(!retry_policy.should_retry(&error));
    let error = RollerMonitorError::ParseConfigError(anyhow::anyhow!("AnyhowError"));
    assert!(!retry_policy.should_retry(&error));
    let error = RollerMonitorError::PushMessageError("push error".to_string());
    assert!(retry_policy.should_retry(&error));
}
