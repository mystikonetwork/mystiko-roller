use mystiko_roller::common::RollerError;

#[tokio::test]
async fn test_error() {
    let err = RollerError::RollerEnvPrivateKeyNotSetError;
    let err_str = format!("{:?}", err);
    assert_eq!(err_str, "RollerEnvPrivateKeyNotSetError");
}
