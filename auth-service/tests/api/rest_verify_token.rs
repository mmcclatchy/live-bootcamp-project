use serde_json::json;

use crate::helpers::create_app_with_logged_in_cookie;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = create_app_with_logged_in_cookie().await;
    let body = json!({ "not_token": "any"} );
    let response = app.post_verify_token(&body).await;
    assert_eq!(
        response.status(),
        422,
        "[TEST][ERROR][should_return_422_if_malformed_input] Failed for input {:?}",
        body,
    );
}
