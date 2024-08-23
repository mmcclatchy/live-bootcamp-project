use rstest::rstest;
use serde_json::json;

use auth_service::domain::data_stores::UserStore;
use auth_service::{api::rest::ErrorResponse, domain::email::Email};

use crate::helpers::{get_random_email, RESTTestApp};

const VALID_PASSWORD: &str = "P@ssw0rd123";

#[rstest]
#[case::missing_email(json!({ "password": VALID_PASSWORD, "requires2FA": true }))]
#[case::missing_requires2fa(json!({ "email": get_random_email(), "password": VALID_PASSWORD }))]
#[case::missing_password(json!({ "email": get_random_email(), "requires2FA": true }))]
#[tokio::test]
async fn rest_signup_should_return_422_if_malformed_input(#[case] input: serde_json::Value) {
    let mut app = RESTTestApp::new().await;
    app.log_user_store("rest_signup_should_return_422_if_malformed_input")
        .await;

    let response = app.post_signup(&input).await;
    assert_eq!(response.status().as_u16(), 422, "Failed for input: {:?}", input);

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn rest_signup_should_return_201_if_valid_input() {
    let mut app = RESTTestApp::new().await;
    app.log_user_store("rest_signup_should_return_201_if_valid_input").await;

    let random_email = get_random_email();
    let signup_request = serde_json::json!({
        "email": random_email,
        "password": VALID_PASSWORD,
        "requires2FA": true,
    });
    let response = app.post_signup(&signup_request).await;

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
    // let password = Password::parse(VALID_PASSWORD.to_string()).await.unwrap();
    // assert_eq!(user.password, password);

    drop(user_store);
    app.clean_up().await.unwrap();
}

#[rstest]
#[case::empty_password(
    json!({
        "email": "test@email.com",
        "password": "",
        "requires2FA": true,
    })
)]
#[case::too_short_password(
    json!({
        "email": "test@email.com",
        "password": "P@ssw0",
        "requires2FA": true,
    })
)]
#[case::no_number(
    json!({
        "email": "test@email.com",
        "password": "Password",
        "requires2FA": true,
    })
)]
#[case::no_uppercase(
    json!({
        "email": "test@email.com",
        "password": "passw0rd",
        "requires2FA": true,
    })
)]
#[tokio::test]
async fn rest_signup_should_return_400_if_invalid_password(#[case] test_case: serde_json::Value) {
    let mut app = RESTTestApp::new().await;
    app.log_user_store("rest_signup_should_return_400_if_invalid_password")
        .await;

    println!("{:?}", test_case);
    let response = app.post_signup(&test_case).await;
    assert_eq!(response.status().as_u16(), 400, "Failed for input: {:?}", test_case);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Invalid Password: Must be at least 8 characters long, contain at least one uppercase character and one number"
            .to_owned()
    );

    app.clean_up().await.unwrap();
}

#[rstest]
#[case::empty_email(
    json!({
        "email": "",
        "password": VALID_PASSWORD,
        "requires2FA": true,
    })
)]
#[case::invalid_email_format(
    json!({
        "email": "invalid_email_format",
        "password": VALID_PASSWORD,
        "requires2FA": true,
    })
)]
#[tokio::test]
async fn rest_signup_should_return_400_if_invalid_email(#[case] test_case: serde_json::Value) {
    let mut app = RESTTestApp::new().await;
    app.log_user_store("rest_signup_should_return_400_if_invalid_input")
        .await;

    println!("{:?}", test_case);
    let response = app.post_signup(&test_case).await;
    assert_eq!(response.status().as_u16(), 400, "Failed for input: {:?}", test_case);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Invalid email address".to_owned()
    );

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn rest_signup_should_return_409_if_email_already_exists() {
    let mut app = RESTTestApp::new().await;
    app.log_user_store("rest_signup_should_return_409_if_email_already_exists")
        .await;

    let random_email = get_random_email();
    let signup_request = serde_json::json!({
        "email": random_email,
        "password": VALID_PASSWORD,
        "requires2FA": true,
    });
    println!(
        "[rest_signup_should_return_409_if_email_already_exists] {:?}",
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

    app.clean_up().await.unwrap();
}
