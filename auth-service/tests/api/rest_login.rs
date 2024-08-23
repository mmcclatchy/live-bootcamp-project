use std::sync::Arc;

use rstest::rstest;
use serde_json::{json, Value};

use auth_service::{
    domain::{
        data_stores::{TwoFACodeStore, UserStore},
        email::Email,
        password::Password,
        user::{DbUser, NewUser},
    },
    routes::login::{LoginResponse, TwoFactorAuthResponse},
    services::app_state::{AppServices, AppState},
    utils::{
        auth::{validate_token, TokenPurpose},
        constants::JWT_COOKIE_NAME,
    },
};

use crate::helpers::{get_random_email, RESTTestApp};

fn create_login_body(email: &str, password: &str) -> Value {
    json!({
        "email": email,
        "password": password,
    })
}

async fn create_db_user(email: &str, password: &str, requires_2fa: bool) -> DbUser {
    let email = Email::parse(email.to_string()).unwrap();
    let password = Password::parse(password.to_string()).await.unwrap();
    DbUser {
        email: email.to_string(),
        password_hash: password.to_string(),
        requires_2fa,
    }
}

async fn create_new_user(email: &str, password: &str, requires_2fa: bool) -> NewUser {
    let email = Email::parse(email.to_string()).unwrap();
    let password = Password::parse(password.to_string()).await.unwrap();
    NewUser {
        email,
        password,
        requires_2fa,
    }
}

async fn create_existing_user<S: AppServices>(app_state: Arc<AppState<S>>, requires_2fa: bool) -> DbUser {
    let random_email = get_random_email();
    let user = create_new_user(&random_email, "P@assw0rd", requires_2fa).await;
    let mut user_store = app_state.user_store.write().await;
    user_store.add_user(user.clone()).await.unwrap();
    DbUser {
        email: user.email.to_string(),
        password_hash: user.password.to_string(),
        requires_2fa: user.requires_2fa,
    }
}

#[rstest]
#[case::missing_password(json!({ "email": "test@example.com" }))]
#[case::missing_email(json!({ "password": "P@ssword123" }))]
#[tokio::test]
async fn should_return_422_if_malformed_credentials(#[case] test_case: serde_json::Value) {
    let mut app = RESTTestApp::new().await;
    let response = app.post_login(&test_case).await;
    assert_eq!(
        response.status(),
        422,
        "[ERROR][should_return_422_if_malformed_credentials] Failed for input {:?}",
        test_case
    );

    app.clean_up().await.unwrap();
}

#[rstest]
#[case::empty_password("test@example.com", "")]
#[case::weak_password_no_special_char("test@example.com", "password")]
#[case::weak_password_no_number("test@example.com", "Password")]
#[case::weak_password_no_uppercase("test@example.com", "passw0rd")]
#[case::invalid_email_no_at("test_example.com", "P@ssword123")]
#[case::invalid_email_no_dot("test@example_com", "P@ssword123")]
#[case::empty_email("", "P@ssword123")]
#[tokio::test]
async fn should_return_400_if_invalid_input(#[case] email: &str, #[case] password: &str) {
    let mut app = RESTTestApp::new().await;
    let test_case = create_login_body(email, password);
    let response = app.post_login(&test_case).await;

    assert_eq!(
        response.status(),
        400,
        "[ERROR][should_return_400_if_invalid_input] Failed for input {:?}",
        test_case
    );

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = RESTTestApp::new().await;
    let user = create_existing_user(app.app_state.clone(), false).await;
    let login_body = json!({ "email": user.email.to_string(),  "password": "Inv@lid_passw0rd".to_string() });
    let login_response = app.post_login(&login_body).await;

    assert_eq!(
        login_response.status(),
        401,
        "[ERROR][should_return_401_if_incorrect_credentials] Failed for input {:?}",
        login_body
    );

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fs_disabled() {
    let mut app = RESTTestApp::new().await;
    let user = create_existing_user(app.app_state.clone(), false).await;
    let login_body = create_login_body(&user.email.to_string(), &user.password_hash);
    let login_response = app.post_login(&login_body).await;

    println!(
        "[TEST][should_return_200_if_valid_credentials_and_2fs_disabled] {:?}",
        app.app_state
    );
    println!(
        "[TEST][should_return_200_if_valid_credentials_and_2fs_disabled] {:?}",
        login_body
    );
    println!(
        "[TEST][should_return_200_if_valid_credentials_and_2fs_disabled] {:?}",
        login_response
    );

    assert_eq!(login_response.status(), 200);

    let auth_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("[ERROR][should_return_200_if_valid_credentials_and_2fs_disabled] No auth cookie found");

    let token = auth_cookie.value();
    assert!(!token.is_empty());

    let claims = validate_token(app.app_state.banned_token_store.clone(), token)
        .await
        .unwrap();
    assert_eq!(claims.sub, user.email.as_ref());
    assert_eq!(claims.purpose, TokenPurpose::Auth);

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let mut app = RESTTestApp::new().await;
    let user = create_existing_user(app.app_state.clone(), true).await;
    let user_email = Email::parse(user.email.clone()).unwrap();
    let login_body = create_login_body(&user.email, &user.password_hash);

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status(), 206);

    let response_body: TwoFactorAuthResponse = login_response
        .json()
        .await
        .expect("[ERROR][should_return_206_if_valid_credentials_and_2fa_enabled] Failed to parse login response body");
    assert_eq!(response_body.message, "2FA required");

    let two_fa_code_store = app.app_state.two_fa_code_store.read().await;
    let (login_attempt_id, _) = two_fa_code_store.get_code(&user_email).await.unwrap();
    assert_eq!(response_body.login_attempt_id, login_attempt_id.to_string());

    drop(two_fa_code_store);
    app.clean_up().await.unwrap();
}
