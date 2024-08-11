use std::sync::Arc;

use rstest::rstest;
use serde_json::{json, Value};

use auth_service::{
    domain::{
        data_stores::{BannedTokenStore, UserStore},
        email::Email,
        password::Password,
        user::User,
    },
    services::app_state::AppState,
    utils::constants::JWT_COOKIE_NAME,
};

use crate::helpers::{get_random_email, RESTTestApp};

fn create_login_body(email: &str, password: &str) -> Value {
    json!({
        "email": email,
        "password": password,
    })
}

fn create_user(email: &str, password: &str, requires_2fa: bool) -> User {
    let email = Email::parse(email.to_string()).unwrap();
    let password = Password::parse(password.to_string()).unwrap();
    User {
        email,
        password,
        requires_2fa,
    }
}

async fn create_existing_user<T: BannedTokenStore, U: UserStore>(
    app_state: Arc<AppState<T, U>>,
) -> User {
    let random_email = get_random_email();
    let user = create_user(&random_email, "P@assw0rd", false);
    let mut user_store = app_state.user_store.write().await;
    user_store.add_user(user.clone()).await.unwrap();
    user
}

#[rstest]
#[case::missing_password(json!({ "email": "test@example.com" }))]
#[case::missing_email(json!({ "password": "P@ssword123" }))]
#[tokio::test]
async fn should_return_422_if_malformed_credentials(#[case] test_case: serde_json::Value) {
    let app = RESTTestApp::new().await;
    let response = app.post_login(&test_case).await;
    assert_eq!(
        response.status(),
        422,
        "[ERROR][should_return_422_if_malformed_credentials] Failed for input {:?}",
        test_case
    );
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
    let app = RESTTestApp::new().await;
    let test_case = create_login_body(email, password);
    let response = app.post_login(&test_case).await;
    assert_eq!(
        response.status(),
        400,
        "[ERROR][should_return_400_if_invalid_input] Failed for input {:?}",
        test_case
    );
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = RESTTestApp::new().await;
    let user = create_existing_user(app.app_state.clone()).await;
    let login_body =
        json!({ "email": user.email.to_string(),  "password": "Inv@lid_passw0rd".to_string() });
    let login_response = app.post_login(&login_body).await;
    assert_eq!(
        login_response.status(),
        401,
        "[ERROR][should_return_401_if_incorrect_credentials] Failed for input {:?}",
        login_body
    )
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fs_disabled() {
    let app = RESTTestApp::new().await;
    let user = create_existing_user(app.app_state.clone()).await;
    let login_body = create_login_body(&user.email.to_string(), &user.password.to_string());
    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status(), 200);
    let auth_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect(
            "[ERROR][should_return_200_if_valid_credentials_and_2fs_disabled] No auth cookie found",
        );
    assert!(!auth_cookie.value().is_empty());
}
