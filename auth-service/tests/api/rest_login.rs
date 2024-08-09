use serde_json::json;

use crate::helpers::RESTTestApp;

// fn create_valid_login_request() -> Value {
//     json!({
//         "email": "test@example.com".to_string(),
//         "password": "P@ssword123".to_string(),
//     })
// }

#[tokio::test]
async fn rest_post_login_should_return_422_if_malformed_credentials() {
    let app = RESTTestApp::new().await;
    let test_cases = [
        json!({ "email": "test@example.com" }),
        json!({ "password": "P@ssword123" }),
    ];
    for test_case in test_cases.iter() {
        let response = app.post_login(&test_case).await;
        assert_eq!(
            response.status(),
            422,
            "[TEST][ERROR][rest_post_login_should_return_422_if_malformed_credentials] Failed for input {:?}",
            test_case
        )
    }
}
