use crate::helpers::{get_random_email, GRPCTestApp, RESTTestApp};
use auth_proto::SignupRequest;
use serde_json::json;
use tonic::Request;

const VALID_PASSWORD: &str = "P@ssw0rd123";

#[tokio::test]
async fn rest_signup_works_for_valid_credentials() {
    let app = RESTTestApp::new().await;
    let email = get_random_email();
    let body = json!({
        "email": email,
        "password": VALID_PASSWORD,
        "requires2FA": false
    });

    let response = app.post_signup(&body).await;
    assert_eq!(response.status().as_u16(), 200);
    let response_body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(response_body["message"], "User created successfully");
}

#[tokio::test]
async fn rest_signup_fails_with_invalid_email() {
    let app = RESTTestApp::new().await;
    let body = json!({
        "email": "not-an-email",
        "password": VALID_PASSWORD,
        "requires2FA": false
    });

    let response = app.post_signup(&body).await;
    assert_eq!(response.status().as_u16(), 400);
    let error_message = RESTTestApp::get_error_message(response).await;
    assert!(error_message.contains("Invalid email"));
}

#[tokio::test]
async fn grpc_signup_works_for_valid_credentials() {
    let mut app = GRPCTestApp::new().await;
    let email = get_random_email();
    let request = Request::new(SignupRequest {
        email: email.clone(),
        password: VALID_PASSWORD.to_string(),
        requires_2fa: false,
    });

    let response = app
        .client
        .signup(request)
        .await
        .expect("Failed to send signup request");
    assert_eq!(
        response.into_inner().message,
        "User created successfully".to_string()
    );
}

#[tokio::test]
async fn grpc_signup_fails_with_invalid_email() {
    let mut app = GRPCTestApp::new().await;
    let request = Request::new(SignupRequest {
        email: "not-an-email".to_string(),
        password: VALID_PASSWORD.to_string(),
        requires_2fa: false,
    });

    let response = app.client.signup(request).await;
    assert!(response.is_err());
    let error = response.unwrap_err();
    assert_eq!(error.code(), tonic::Code::InvalidArgument);
    assert!(error.message().contains("Invalid email"));
}
