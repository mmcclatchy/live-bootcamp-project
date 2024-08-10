use auth_service::{api::rest::ErrorResponse, utils::constants::JWT_COOKIE_NAME};
use reqwest::Url;

use crate::helpers::RESTTestApp;

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = RESTTestApp::new().await;
    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status(), 400)
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = RESTTestApp::new().await;
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME,
        ),
        &Url::parse("http://127.0.0.1")
            .expect("[ERROR][TEST][should_return_401_if_invalid_token] Failed to parse URL"),
    );
    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status(), 401);
}
