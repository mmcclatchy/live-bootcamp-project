use crate::helpers::{get_random_email, RESTTestApp};
use auth_service::domain::data_stores::UserStore;
use auth_service::{
    api::rest::ErrorResponse,
    domain::{email::Email, password::Password},
};

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
