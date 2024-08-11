use auth_service::{api::rest::ErrorResponse, utils::constants::JWT_COOKIE_NAME};
use reqwest::Url;

use crate::helpers::{create_app_with_logged_in_cookie, RESTTestApp};

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let app = create_app_with_logged_in_cookie().await;
    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status(), 200);
    let cookie = logout_response
        .cookies()
        .find(|c| c.name() == JWT_COOKIE_NAME)
        .expect("[ERROR][TEST][should_return_200_if_valid_jwt_cookie] No cookie returned");
    assert!(cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let app = create_app_with_logged_in_cookie().await;
    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status(), 200);
    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status(), 400);
    assert_eq!(
        logout_response
            .json::<ErrorResponse>()
            .await
            .expect("[ERROR][TEST][should_return_400_if_jwt_cookie_missing] Could not deserialize response body to ErrorResponse")
            .error,
        "Missing auth token".to_owned()
    );
}

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = RESTTestApp::new().await;
    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status(), 400);
    let cookie = logout_response
        .cookies()
        .find(|c| c.name() == JWT_COOKIE_NAME);
    assert!(cookie.is_none());
    assert_eq!(
        logout_response
            .json::<ErrorResponse>()
            .await
            .expect("[ERROR][TEST][should_return_400_if_jwt_cookie_missing] Could not deserialize response body to ErrorResponse")
            .error,
        "Missing auth token".to_owned()
    );
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = RESTTestApp::new().await;
    app.cookie_jar.add_cookie_str(
        &format!("{JWT_COOKIE_NAME}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/"),
        &Url::parse("http://127.0.0.1")
            .expect("[ERROR][TEST][create_app_with_cookie] Failed to parse URL"),
    );
    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status(), 401);
    let auth_cookie = logout_response
        .cookies()
        .find(|c| c.name() == JWT_COOKIE_NAME);
    assert!(auth_cookie.is_none());
    assert_eq!(
        logout_response
            .json::<ErrorResponse>()
            .await
            .expect("[ERROR][TEST][should_return_401_if_invalid_token] Could not deserialize response body to ErrorResponse")
            .error,
        "Invalid auth token".to_owned()
    );
}
