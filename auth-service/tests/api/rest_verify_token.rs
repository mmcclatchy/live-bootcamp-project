use auth_service::{api::rest::ErrorResponse, domain::email::Email, utils::auth::generate_auth_token};
use secrecy::Secret;
use serde_json::json;

use crate::helpers::{create_app_with_logged_in_token, get_random_email, RESTTestApp};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = RESTTestApp::new().await;
    let body = json!({ "not_token": "any"} );
    let response = app.post_verify_token(&body).await;
    assert_eq!(
        response.status(),
        422,
        "[ERROR][should_return_422_if_malformed_input] Failed for input {:?}",
        body,
    );

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let mut app = RESTTestApp::new().await;
    let email = get_random_email();
    let email = Email::parse(Secret::new(email)).unwrap();
    let token = generate_auth_token(&email).unwrap();
    let request_body = json!({ "token": token });
    let response = app.post_verify_token(&request_body).await;
    assert_eq!(
        response.status(),
        200,
        "[ERROR][should_return_200_valid_token] Failed for input {:?}",
        response,
    );

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let mut app = RESTTestApp::new().await;
    let request_body = json!({ "token": "invalid token" });
    let response = app.post_verify_token(&request_body).await;
    assert_eq!(
        response.status(),
        401,
        "[ERROR][should_return_401_if_invalid_token] Failed for input {:?}",
        response,
    );
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("[ERROR][should_return_401_if_invalid_token] Could not deserialize response body to ErrorResponse")
            .error,
        "Invalid credentials".to_owned()
    );

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
    let (mut app, token) = create_app_with_logged_in_token().await;

    let logout_response = app.post_logout().await;
    assert_eq!(
        logout_response.status(),
        200,
        "[ERROR][should_return_401_if_banned_token][logout] Failed for input {:?}",
        logout_response,
    );

    let verify_token_request_body = json!({ "token": token });
    let verify_token_response = app.post_verify_token(&verify_token_request_body).await;
    assert_eq!(
        verify_token_response.status(),
        401,
        "[ERROR][should_return_401_if_banned_token][verify-token] Failed for input {:?}",
        verify_token_response,
    );
    assert_eq!(
        verify_token_response
            .json::<ErrorResponse>()
            .await
            .expect("[ERROR][should_return_401_if_banned_token][verify-token] Could not deserialize response body to ErrorResponse")
            .error,
        "Invalid credentials".to_owned()
    );

    app.clean_up().await.unwrap();
}
