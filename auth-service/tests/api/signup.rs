use auth_proto::SignupRequest;
use tonic::Request;

use crate::helpers::{get_random_email, TestApp};

const VALID_PASSWORD: &str = "P@assw0rd";

#[tokio::test]
async fn signup_should_return_invalid_argument_if_malformed_input() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();
    let test_cases = [
        SignupRequest {
            email: "".to_string(),
            password: VALID_PASSWORD.to_string(),
            requires_2fa: true,
        },
        SignupRequest {
            email: random_email.clone(),
            password: "".to_string(),
            requires_2fa: true,
        },
    ];

    for test_case in test_cases.iter() {
        let response = app.client.signup(Request::new(test_case.clone())).await;
        assert_eq!(
            response.unwrap_err().code(),
            tonic::Code::InvalidArgument,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn signup_should_return_ok_if_valid_input() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();
    let signup_request = SignupRequest {
        email: random_email,
        password: VALID_PASSWORD.to_string(),
        requires_2fa: true,
    };
    let response = app
        .client
        .signup(Request::new(signup_request.clone()))
        .await;
    assert!(response.is_ok(), "Failed for input: {:?}", signup_request);
}

#[tokio::test]
async fn should_return_invalid_argument_if_invalid_input() {
    let mut app = TestApp::new().await;
    let test_cases = [
        SignupRequest {
            email: "".to_string(),
            password: VALID_PASSWORD.to_string(),
            requires_2fa: true,
        },
        SignupRequest {
            email: "random_email".to_string(),
            password: VALID_PASSWORD.to_string(),
            requires_2fa: true,
        },
        SignupRequest {
            email: "test@email.com".to_string(),
            password: "invalid".to_string(),
            requires_2fa: true,
        },
    ];

    for test_case in test_cases.iter() {
        let response = app.client.signup(Request::new(test_case.clone())).await;
        assert_eq!(
            response.unwrap_err().code(),
            tonic::Code::InvalidArgument,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_already_exists_if_email_already_exists() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();
    let signup_request = SignupRequest {
        email: random_email,
        password: VALID_PASSWORD.to_string(),
        requires_2fa: true,
    };
    app.client
        .signup(Request::new(signup_request.clone()))
        .await
        .unwrap();
    let response = app
        .client
        .signup(Request::new(signup_request.clone()))
        .await;
    assert_eq!(
        response.unwrap_err().code(),
        tonic::Code::AlreadyExists,
        "Failed for input: {:?}",
        signup_request
    );
}
