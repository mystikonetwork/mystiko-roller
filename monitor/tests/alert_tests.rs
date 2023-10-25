use mystiko_roller_monitor::{IntoMessage, MonitorAlert};
use rusoto_sns::PublishInput;

#[test]
fn test_create_alert() {
    let alert = MonitorAlert::<PublishInput>::builder()
        .error_message("error_message".to_string())
        .topic_arn("topic_arn".to_string())
        .build();
    let message = alert.into_message();
    let alert2 = MonitorAlert::<PublishInput>::builder()
        .error_message("error_message".to_string())
        .topic_arn("topic_arn".to_string())
        .build();
    assert_eq!(message.message, alert2.error_message);
    assert_eq!(message.topic_arn, alert2.topic_arn);
}
