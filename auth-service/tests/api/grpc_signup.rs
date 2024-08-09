use crate::helpers::{get_random_email, wait_for_user, GRPCTestApp};
use auth_proto::SignupRequest;
use auth_service::domain::{email::Email, password::Password};
use tonic::Request;

const VALID_PASSWORD: &str = "P@ssw0rd123";

#[tokio::test]
async fn grpc_signup_works_for_valid_credentials() {
    let mut app = GRPCTestApp::new().await;
    app.log_user_store("grpc_signup_works_for_valid_credentials")
        .await;

    let email = get_random_email();
    let request = Request::new(SignupRequest {
        email: email.clone(),
        password: VALID_PASSWORD.to_string(),
        requires_2fa: false,
    });

    let response = app.client.signup(request).await.expect(
        "[ERROR][signup][tests][grpc_signup_works_for_valid_credentials] Failed to send signup request",
    );
    println!(
        "[signup][tests][grpc_signup_works_for_valid_credentials] {:?}",
        response
    );
    assert_eq!(
        response.into_inner().message,
        "User created successfully".to_string()
    );

    let user_store = app.app_state.user_store.read().await;

    let user_email = Email::parse(email).unwrap();
    println!(
        "[signup][tests][grpc_signup_works_for_valid_credentials] {:?}",
        user_email
    );

    let user = wait_for_user(user_store, &user_email, 5, 100)
        .await
        .expect("[ERROR][signup][tests][grpc_signup_works_for_valid_credentials] User not found");
    println!(
        "[signup][tests][grpc_signup_works_for_valid_credentials] {:?}",
        user
    );
    assert_eq!(user.email, user_email);

    let user_password = Password::parse(VALID_PASSWORD.to_string()).expect("Invalid Password");
    println!(
        "[signup][tests][grpc_signup_works_for_valid_credentials] {:?}",
        user_password
    );
    assert_eq!(user.password, user_password)
}

#[tokio::test]
async fn grpc_signup_fails_with_invalid_email() {
    let mut app = GRPCTestApp::new().await;
    app.log_user_store("grpc_signup_fails_with_invalid_email")
        .await;

    let request = Request::new(SignupRequest {
        email: "not-an-email".to_string(),
        password: VALID_PASSWORD.to_string(),
        requires_2fa: false,
    });

    let response = app.client.signup(request).await;
    println!(
        "[signup][tests][grpc_signup_fails_with_invalid_email] {:?}",
        response
    );
    assert!(response.is_err());
    let error = response.unwrap_err();
    println!(
        "[signup][tests][grpc_signup_fails_with_invalid_email] {:?}",
        error
    );
    assert_eq!(error.code(), tonic::Code::InvalidArgument);
    assert!(error.message().contains("Invalid email"));
}

#[tokio::test]
async fn grpc_signup_fails_with_invalid_password() {
    let mut app = GRPCTestApp::new().await;
    app.log_user_store("grpc_signup_fails_with_invalid_password")
        .await;

    let email = get_random_email();
    let request = Request::new(SignupRequest {
        email,
        password: "invalid_password".to_string(),
        requires_2fa: false,
    });

    let response = app.client.signup(request).await;
    println!(
        "[signup][tests][grpc_signup_fails_with_invalid_password] {:?}",
        response
    );
    assert!(response.is_err());
    let error = response.unwrap_err();
    println!(
        "[signup][tests][grpc_signup_fails_with_invalid_password] {:?}",
        error
    );
    assert_eq!(error.code(), tonic::Code::InvalidArgument);
    assert!(error.message().contains("Invalid password"));
}

#[tokio::test]
async fn grpc_signup_should_return_already_exists_if_email_already_exists() {
    let mut app = GRPCTestApp::new().await;
    app.log_user_store("grpc_signup_should_return_already_exists_if_email_already_exists")
        .await;

    let email = get_random_email();

    println!("gRPC SignupRequest 1");
    let request = Request::new(SignupRequest {
        email: email.clone(),
        password: VALID_PASSWORD.to_string(),
        requires_2fa: false,
    });

    let response = app.client.signup(request).await;
    println!(
        "[signup][tests][grpc_signup_should_return_already_exists_if_email_already_exists][1] {:?}",
        response
    );
    assert!(response.is_ok());

    println!("gRPC SignupRequest 2");
    let request = Request::new(SignupRequest {
        email,
        password: VALID_PASSWORD.to_string(),
        requires_2fa: false,
    });
    let response = app.client.signup(request).await;
    println!(
        "[signup][tests][grpc_signup_should_return_already_exists_if_email_already_exists][2] {:?}",
        response
    );

    assert!(response.is_err());
    let error = response.unwrap_err();
    println!(
        "[signup][tests][grpc_signup_should_return_already_exists_if_email_already_exists] {:?}",
        error
    );

    assert_eq!(error.code(), tonic::Code::AlreadyExists);
    assert_eq!(error.message(), "User already exists");
}
