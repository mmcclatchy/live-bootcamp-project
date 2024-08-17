use crate::helpers::{get_random_email, RESTTestApp};
use serde_json::json;

#[tokio::test]
async fn initiate_password_reset_works_for_existing_email() {
    let app = RESTTestApp::new().await;
    let email = get_random_email();

    let signup_body = json!({
        "email": email,
        "password": "P@ssw0rd123",
        "requires2FA": false
    });
    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status(), 201);

    let reset_body = json!({ "email": email });
    let reset_response = app.post_initiate_password_reset(&reset_body).await;
    assert_eq!(reset_response.status(), 200);

    let response_body: serde_json::Value = reset_response.json().await.unwrap();
    assert_eq!(
        response_body["message"],
        "If the email exists, a password reset link has been sent."
    );
}

#[tokio::test]
async fn initiate_password_reset_returns_same_message_for_non_existing_email() {
    let app = RESTTestApp::new().await;
    let non_existing_email = get_random_email();

    let reset_body = json!({ "email": non_existing_email });
    let reset_response = app.post_initiate_password_reset(&reset_body).await;
    assert_eq!(reset_response.status(), 200);

    let response_body: serde_json::Value = reset_response.json().await.unwrap();
    assert_eq!(
        response_body["message"],
        "If the email exists, a password reset link has been sent."
    );
}

#[tokio::test]
async fn initiate_password_reset_sends_400_with_invalid_email() {
    let app = RESTTestApp::new().await;
    let reset_body = json!({ "email": "not-an-email" });

    let reset_response = app.post_initiate_password_reset(&reset_body).await;
    assert_eq!(reset_response.status(), 400);

    let response_body: serde_json::Value = reset_response.json().await.unwrap();
    assert_eq!(response_body["error"], "Invalid email address");
}

#[tokio::test]
async fn reset_password_works_with_valid_token() {
    let app = RESTTestApp::new().await;
    let email = get_random_email();

    let signup_body = json!({
        "email": email,
        "password": "P@ssw0rd123",
        "requires2FA": false
    });
    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status(), 201);

    let reset_init_body = json!({ "email": email });
    app.post_initiate_password_reset(&reset_init_body).await;

    // Get the reset token (in a real scenario, this would be sent via email)
    let reset_token = app.get_password_reset_token(&email).await.unwrap();

    let reset_body = json!({
        "token": reset_token,
        "new_password": "NewP@ssw0rd123"
    });
    let reset_response = app.post_reset_password(&reset_body).await;
    assert_eq!(reset_response.status(), 200);

    let response_body: serde_json::Value = reset_response.json().await.unwrap();
    assert_eq!(
        response_body["message"],
        "Password has been reset successfully."
    );

    let login_body = json!({
        "email": email,
        "password": "NewP@ssw0rd123"
    });
    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status(), 200);
}

#[tokio::test]
async fn reset_password_fails_with_invalid_token() {
    let app = RESTTestApp::new().await;
    let reset_body = json!({
        "token": "invalid_token",
        "new_password": "NewP@ssw0rd123"
    });

    let reset_response = app.post_reset_password(&reset_body).await;
    assert_eq!(reset_response.status(), 400);

    let response_body: serde_json::Value = reset_response.json().await.unwrap();
    assert_eq!(response_body["message"], "Invalid auth token");
}

#[tokio::test]
async fn reset_password_fails_with_weak_password() {
    let app = RESTTestApp::new().await;
    let email = get_random_email();

    let signup_body = json!({
        "email": email,
        "password": "P@ssw0rd123",
        "requires2FA": false
    });
    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status(), 201);

    let reset_init_body = json!({
        "email": email
    });
    app.post_initiate_password_reset(&reset_init_body).await;

    let reset_token = app.get_password_reset_token(&email).await.unwrap();

    // Try resetting with a weak password
    let reset_body = json!({
        "token": reset_token,
        "new_password": "weakpassword"
    });
    let reset_response = app.post_reset_password(&reset_body).await;
    assert_eq!(reset_response.status(), 400);

    let response_body: serde_json::Value = reset_response.json().await.unwrap();
    assert!(response_body["message"]
        .to_string()
        .contains("Invalid password"));
}
