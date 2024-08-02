use auth_proto::SignupRequest;
use tonic::Request;

use crate::helpers::{get_random_email, TestApp};

const VALID_PASSWORD: &str = "P@ssw0rd123";

#[tokio::test]
async fn signup_works_for_valid_credentials() {
    let mut app = TestApp::new().await;
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
async fn signup_fails_with_invalid_email() {
    let mut app = TestApp::new().await;
    let request = Request::new(SignupRequest {
        email: "not-an-email".to_string(),
        password: VALID_PASSWORD.to_string(),
        requires_2fa: false,
    });

    let response = app.client.signup(request).await;
    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn signup_fails_with_weak_password() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let request = Request::new(SignupRequest {
        email,
        password: "weak".to_string(),
        requires_2fa: false,
    });

    let response = app.client.signup(request).await;
    assert!(response.is_err());
    assert_eq!(response.unwrap_err().code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn signup_fails_if_email_already_exists() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let signup_request = SignupRequest {
        email: email.clone(),
        password: VALID_PASSWORD.to_string(),
        requires_2fa: false,
    };

    let request = Request::new(signup_request.clone());
    let response = app.client.signup(request).await;
    assert!(response.is_ok(), "First signup should succeed");

    // Second signup with the same email should fail
    let request = Request::new(signup_request);
    let response = app.client.signup(request).await;
    assert!(response.is_err(), "Second signup should fail");
    let error = response.unwrap_err();
    assert_eq!(
        error.code(),
        tonic::Code::AlreadyExists,
        "Expected AlreadyExists error, got: {:?}",
        error
    );
}
