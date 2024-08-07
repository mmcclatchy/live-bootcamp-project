use crate::helpers::{get_random_email, wait_for_user, GRPCTestApp, RESTTestApp};
use auth_proto::SignupRequest;
use auth_service::domain::data_stores::UserStore;
use auth_service::{
    api::rest::ErrorResponse,
    domain::{email::Email, password::Password, user},
};
use axum::extract::State;
use quickcheck::single_shrinker;
use serde_json::json;
use tonic::Request;

const VALID_PASSWORD: &str = "P@ssw0rd123";

#[tokio::test]
async fn rest_signup_should_return_422_if_malformed_input() {
    let app = RESTTestApp::new().await;
    app.log_user_store("rest_signup_should_return_422_if_malformed_input")
        .await;

    let random_email = get_random_email();
    let test_cases = [
        serde_json::json!({ "password": VALID_PASSWORD, "requires2FA": true }),
        serde_json::json!({ "email": random_email, "password": VALID_PASSWORD }),
        serde_json::json!({ "email": random_email, "requires2FA": true }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn rest_signup_should_return_201_if_valid_input() {
    let app = RESTTestApp::new().await;
    app.log_user_store("rest_signup_should_return_201_if_valid_input")
        .await;

    let random_email = get_random_email();
    let signup_request = serde_json::json!({
        "email": random_email,
        "password": VALID_PASSWORD,
        "requires2FA": true,
    });
    let response = app.post_signup(&signup_request).await;
    // tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    assert_eq!(
        response.status().as_u16(),
        201,
        "Failed for input: {:?}. Response body: {:?}",
        signup_request,
        response.text().await.unwrap_or_default()
    );

    let app_state = &app.app_state;
    let user_store = app_state.user_store.read().await;
    let email = Email::parse(random_email).unwrap();
    let user = user_store.get_user(&email).await.expect("User not found");

    assert_eq!(user.email, email);
    let password = Password::parse(VALID_PASSWORD.to_string()).unwrap();
    assert_eq!(user.password, password);
}

#[tokio::test]
async fn rest_signup_should_return_400_if_invalid_password() {
    let app = RESTTestApp::new().await;
    app.log_user_store("rest_signup_should_return_400_if_invalid_password")
        .await;

    let test_cases = [
        serde_json::json!({
            "email": "test@email.com",
            "password": "",
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": "test@email.com",
            "password": "P@ssw0",
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": "test@email.com",
            "password": "Password",
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": "test@email.com",
            "password": "passw0rd",
            "requires2FA": true,
        }),
    ];

    for test_case in test_cases.iter() {
        println!("{:?}", test_case);
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );
        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid Password: Must be at least 8 characters long, contain at least one uppercase character and one number".to_owned()
        );
    }
}

#[tokio::test]
async fn rest_signup_should_return_400_if_invalid_email() {
    let app = RESTTestApp::new().await;
    app.log_user_store("rest_signup_should_return_400_if_invalid_input")
        .await;

    app.log_user_store("rest_signup_should_return_400_if_invalid_input")
        .await;

    let test_cases = [
        serde_json::json!({
            "email": "",
            "password": VALID_PASSWORD,
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": "invalid_email_format",
            "password": VALID_PASSWORD,
            "requires2FA": true,
        }),
    ];

    for test_case in test_cases.iter() {
        println!("{:?}", test_case);
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );
        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid email address".to_owned()
        );
    }
}

#[tokio::test]
async fn rest_signup_should_return_409_if_email_already_exists() {
    let app = RESTTestApp::new().await;
    app.log_user_store("rest_signup_should_return_409_if_email_already_exists")
        .await;

    let random_email = get_random_email();
    let signup_request = serde_json::json!({
        "email": random_email,
        "password": VALID_PASSWORD,
        "requires2FA": true,
    });
    println!(
        "[TEST][rest_signup_should_return_409_if_email_already_exists] {:?}",
        signup_request
    );
    app.post_signup(&signup_request).await;

    let response = app.post_signup(&signup_request).await;

    assert_eq!(
        response.status().as_u16(),
        409,
        "Failed for input: {:?}",
        signup_request
    );
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );
}

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

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

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

    // let user = user_store
    //     .get_user(&user_email)
    //     .await
    //     .expect("[ERROR][signup][tests][grpc_signup_works_for_valid_credentials] User not found");
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
