use rstest::rstest;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::helpers::RESTTestApp;

#[rstest]
#[case::missing_email(json!({ "loginAttemptId": Uuid::new_v4().to_string(), "2FACode": "123456" }))]
#[case::missing_log_in_attempt_id(json!({ "email": "test@example.com".to_string(), "2FACode": "123456" }))]
#[case::missing_2fa_code(json!({ "email": "test@example.com".to_string(), "loginAttemptId": Uuid::new_v4().to_string() }))]
#[tokio::test]
async fn should_return_422_if_malformed_input(#[case] test_case: Value) {
    let app = RESTTestApp::new().await;
    let response = app.post_verify_2fa(&test_case).await;

    assert_eq!(response.status(), 422);
}
