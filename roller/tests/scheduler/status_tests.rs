use mystiko_roller::scheduler::status::{RollerStatusAction, RollerStatusGetter, RollerStatusWrapper};
use mystiko_status_server::Status;
use std::sync::Arc;

#[tokio::test]
async fn test_roller_status_wrapper() {
    let wrapper = RollerStatusWrapper::new().await;
    let status = wrapper.get_status().await;
    assert_eq!(status.action, RollerStatusAction::Idle);

    wrapper.set_action(RollerStatusAction::Loading).await;
    let status = wrapper.get_status().await;
    assert_eq!(status.action, RollerStatusAction::Loading);

    wrapper.set_action(RollerStatusAction::Rollup).await;
    let status = wrapper.get_status().await;
    assert_eq!(status.action, RollerStatusAction::Rollup);
}

#[tokio::test]
async fn test_roller_status_getter() {
    let wrapper = RollerStatusWrapper::new().await;
    let getter = RollerStatusGetter::builder().status(Arc::new(wrapper)).build();
    let status = getter.status().await.unwrap();
    assert_eq!(status.0, mime::APPLICATION_JSON);

    // Extract the body contents as a String
    let body_bytes = hyper::body::to_bytes(status.1).await.expect("Failed to read body");
    let body_string = String::from_utf8(body_bytes.to_vec()).expect("Body content was not valid UTF-8");
    assert_eq!(body_string, r#"{"action":"Idle"}"#);
}
