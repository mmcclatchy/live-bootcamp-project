use rstest::rstest;
use secrecy::{ExposeSecret, Secret};
use serde_json::{json, Value};
use uuid::Uuid;

use auth_service::{
    domain::{data_stores::TwoFACodeStore, email::Email},
    routes::login::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME,
};
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{get_random_email, RESTTestApp};

const LOG_PREFIX: &str = "[TEST][rest_verify_2fa]";
const TEST_EMAIL: &str = "test@example.com";
const TEST_PASSWORD: &str = "P@ssw0rd";

fn get_valid_email() -> Email {
    Email::parse(Secret::new(get_random_email())).unwrap()
}

async fn get_two_fa_login_response(app: RESTTestApp, email: Email) -> (RESTTestApp, TwoFactorAuthResponse, Email) {
    println!("[get_two_fa_login_response] Initialized");
    let login_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": TEST_PASSWORD,
    });

    let login_response = app.post_login(&login_body).await;

    assert_eq!(login_response.status(), 206);
    let login_response = login_response.json().await.unwrap();

    println!("[get_two_fa_login_response] Exiting");
    (app, login_response, email)
}

async fn create_app_with_login_response(expected_email_calls: u64) -> (RESTTestApp, TwoFactorAuthResponse, Email) {
    let app = RESTTestApp::new().await;
    let email = get_valid_email();

    Mock::given(path("/email/withTemplate"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(expected_email_calls)
        .mount(&app.email_server)
        .await;

    let signup_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": TEST_PASSWORD,
        "requires2FA": true,
    });
    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status(), 201);

    get_two_fa_login_response(app, email).await
}

#[rstest]
#[case::missing_email(json!({ "loginAttemptId": Uuid::new_v4().to_string(), "2FACode": "123456" }))]
#[case::missing_log_in_attempt_id(json!({ "email": TEST_EMAIL.to_string(), "2FACode": "123456" }))]
#[case::missing_2fa_code(json!({ "email": TEST_EMAIL.to_string(), "loginAttemptId": Uuid::new_v4().to_string() }))]
#[tokio::test]
async fn should_return_422_if_malformed_input(#[case] test_case: Value) {
    let mut app = RESTTestApp::new().await;
    let response = app.post_verify_2fa(&test_case).await;

    assert_eq!(response.status(), 422);

    app.clean_up().await.unwrap();
}

#[rstest]
#[case::invalid_email("invalid_email", Uuid::new_v4().to_string(), "123456")]
#[case::invalid_login_attempt_id(TEST_EMAIL, "invalid_login_attempt_id".to_string(), "123456")]
#[case::invalid_2fa_code(TEST_EMAIL, Uuid::new_v4().to_string(), "invalid_2fa_code")]
#[tokio::test]
async fn should_return_400_if_invalid_input(
    #[case] email: &str,
    #[case] login_attempt_id: String,
    #[case] two_fa_code: &str,
) {
    let mut app = RESTTestApp::new().await;
    let v2fa_body = json!({
        "email": email,
        "loginAttemptId": login_attempt_id,
        "2FACode": two_fa_code,
    });
    let response = app.post_verify_2fa(&v2fa_body).await;

    let two_fa_auth_response_status = response.status();

    assert_eq!(two_fa_auth_response_status, 400);

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let (mut app, login_response, email) = create_app_with_login_response(1).await;

    let two_fa_code_store = app.app_state.two_fa_code_store.read().await;
    let (_, two_fa_code) = two_fa_code_store.get_code(&email).await.unwrap();
    drop(two_fa_code_store);

    let invalid_two_fa_code = match two_fa_code.as_ref().expose_secret().as_str() {
        "123456" => "654321".to_string(),
        _ => "123456".to_string(),
    };

    let test_cases = [
        json!({
            "email": TEST_EMAIL,
            "loginAttemptId": Uuid::new_v4().to_string(),
            "2FACode": two_fa_code.as_ref().expose_secret().clone(),
        }),
        json!({
            "email": TEST_EMAIL,
            "loginAttemptId": login_response.login_attempt_id,
            "2FACode": invalid_two_fa_code,
        }),
    ];

    for (i, test_case) in test_cases.iter().enumerate() {
        println!(
            "{LOG_PREFIX}[should_return_401_if_incorrect_credentials] Testing case {}: {}",
            i + 1,
            test_case
        );
        let response = app.post_verify_2fa(&test_case).await;
        assert_eq!(response.status(), 401, "Test case {} failed", i + 1);
    }

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let (app, login_response, email) = create_app_with_login_response(2).await;

    let two_fa_code_store = app.app_state.two_fa_code_store.read().await;
    let (_, two_fa_code) = two_fa_code_store.get_code(&email).await.unwrap();
    drop(two_fa_code_store);

    let (mut app, _, email) = get_two_fa_login_response(app, email).await;

    let verify_2fa_body = json!({
        "email": email.as_ref().expose_secret(),
        "loginAttemptId": login_response.login_attempt_id,
        "2FACode": two_fa_code.as_ref().expose_secret().clone(),
    });

    let verify_2fa_response = app.post_verify_2fa(&verify_2fa_body).await;
    assert_eq!(verify_2fa_response.status(), 401);

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_401_if_code_used_twice() {
    let (mut app, login_response, email) = create_app_with_login_response(1).await;

    let two_fa_code_store = app.app_state.two_fa_code_store.read().await;
    let (_, two_fa_code) = two_fa_code_store.get_code(&email).await.unwrap();
    drop(two_fa_code_store);

    let verify_2fa_body = json!({
        "email": email.as_ref().expose_secret(),
        "loginAttemptId": login_response.login_attempt_id,
        "2FACode": two_fa_code.as_ref().expose_secret().clone(),
    });

    let verify_2fa_response = app.post_verify_2fa(&verify_2fa_body).await;
    assert_eq!(verify_2fa_response.status(), 200);

    let verify_2fa_response = app.post_verify_2fa(&verify_2fa_body).await;
    assert_eq!(verify_2fa_response.status(), 401);

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_200_if_correct_code() {
    let (mut app, login_response, email) = create_app_with_login_response(1).await;

    let two_fa_code_store = app.app_state.two_fa_code_store.read().await;
    let (_, two_fa_code) = two_fa_code_store.get_code(&email).await.unwrap();
    drop(two_fa_code_store);

    let verify_2fa_body = json!({
        "email": email.as_ref().expose_secret(),
        "loginAttemptId": login_response.login_attempt_id,
        "2FACode": two_fa_code.as_ref().expose_secret().clone(),
    });

    let verify_2fa_response = app.post_verify_2fa(&verify_2fa_body).await;
    assert_eq!(verify_2fa_response.status(), 200);

    let auth_cookie = verify_2fa_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("[ERROR][should_return_200_if_valid_credentials_and_2fs_disabled] No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    app.clean_up().await.unwrap();
}
