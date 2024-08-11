use auth_service::{domain::email::Email, utils::auth::generate_auth_token};
use serde_json::json;

use crate::helpers::{get_random_email, RESTTestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = RESTTestApp::new().await;
    let body = json!({ "not_token": "any"} );
    let response = app.post_verify_token(&body).await;
    assert_eq!(
        response.status(),
        422,
        "[TEST][ERROR][should_return_422_if_malformed_input] Failed for input {:?}",
        body,
    );
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let app = RESTTestApp::new().await;
    let email = get_random_email();
    let email = Email::parse(email).unwrap();
    let token = generate_auth_token(&email).unwrap();
    let request_body = json!({ "token": token });
    let response = app.post_verify_token(&request_body).await;
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = RESTTestApp::new().await;
    let request_body = json!({ "token": "invalid token" });
    let response = app.post_verify_token(&request_body).await;
    assert_eq!(response.status(), 401);
}
