use std::sync::Arc;

use auth_service::{
    domain::{
        data_stores::UserStore,
        email::Email,
        password::{self, Password},
        user::User,
    },
    routes::login::LoginRequest,
    services::app_state::AppState,
};
use serde_json::{json, Value};

use crate::helpers::RESTTestApp;

const USER_EMAIL: &str = "test@email.com";
const USER_PASSWORD: &str = "P@assw0rd";

fn create_valid_login_request() -> Value {
    json!({
        "email": USER_EMAIL.to_string(),
        "password": USER_PASSWORD.to_string(),
    })
}

// fn create_valid_signup_request() -> Value {
//     json!({
//         "email": USER_EMAIL.to_string(),
//         "password": USER_PASSWORD.to_string(),
//         "requires2FA": false,
//     })
// }

fn get_user_email() -> Email {
    Email::parse(USER_EMAIL.to_string()).unwrap()
}

fn get_user_password() -> Password {
    Password::parse(USER_PASSWORD.to_string()).unwrap()
}

fn create_user() -> User {
    let email = get_user_email();
    let password = get_user_password();
    User {
        email,
        password,
        requires_2fa: false,
    }
}

async fn create_existing_user<T: UserStore>(app_state: Arc<AppState<T>>) -> User {
    let user = create_user();
    let mut user_store = app_state.user_store.write().await;
    user_store.add_user(user.clone()).await.unwrap();
    user
}

#[tokio::test]
async fn rest_post_login_should_return_422_if_malformed_credentials() {
    let app = RESTTestApp::new().await;
    let test_cases = [
        json!({ "email": "test@example.com" }),
        json!({ "password": "P@ssword123" }),
    ];
    for test_case in test_cases.iter() {
        let response = app.post_login(&test_case).await;
        assert_eq!(
            response.status(),
            422,
            "[TEST][ERROR][rest_post_login_should_return_422_if_malformed_credentials] Failed for input {:?}",
            test_case
        )
    }
}

#[tokio::test]
async fn rest_post_login_should_return_400_if_invalid_input() {
    let app = RESTTestApp::new().await;
    let test_cases = [
        json!({ "email": "test@example.com", "password": "" }),
        json!({ "email": "test@example.com", "password": "password" }),
        json!({ "email": "test@example.com", "password": "Password" }),
        json!({ "email": "test@example.com", "password": "passw0rd" }),
        json!({ "email": "test_example.com", "password": "P@ssword123" }),
        json!({ "email": "test@example_com", "password": "P@ssword123" }),
        json!({ "email": "", "password": "P@ssword123" }),
    ];
    for test_case in test_cases.iter() {
        let response = app.post_login(&test_case).await;
        assert_eq!(
            response.status(),
            400,
            "[TEST][ERROR][rest_post_login_should_return_400_if_invalid_input] Failed for input {:?}",
            test_case
        )
    }
}

#[tokio::test]
async fn rest_post_login_should_return_401_if_incorrect_credentials() {
    let app = RESTTestApp::new().await;
    let user = create_existing_user(app.app_state.clone()).await;
    let login_request =
        json!({ "email": user.email.to_string(),  "password": "Inv@lid_passw0rd".to_string() });
    let login_response = app.post_login(&login_request).await;
    assert_eq!(
        login_response.status(),
        401,
        "[ERROR][TEST][rest_post_login_should_return_401_if_incorrect_credentials] Failed for input {:?}",
        login_request
    )
}
