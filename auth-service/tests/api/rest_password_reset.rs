use crate::helpers::{get_random_email, RESTTestApp};
use auth_service::utils::{
    auth::{validate_token, TokenPurpose},
    constants::JWT_COOKIE_NAME,
};
use secrecy::{ExposeSecret, Secret};
use serde_json::json;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[tokio::test]
async fn initiate_password_reset_should_return_200_if_existing_email() {
    let mut app = RESTTestApp::new().await;
    let email = get_random_email();

    let signup_body = json!({
        "email": email,
        "password": "P@ssw0rd123",
        "requires2FA": false
    });
    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status(), 201);

    Mock::given(path("/email/withTemplate"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let init_reset_body = json!({ "email": email });
    let initiate_reset_response = app.post_initiate_password_reset(&init_reset_body).await;
    println!(
        "[TEST][initiate_password_reset_should_return_200_if_existing_email] {:?}",
        initiate_reset_response
    );

    assert_eq!(initiate_reset_response.status(), 200);

    let response_body: serde_json::Value = initiate_reset_response.json().await.unwrap();
    println!(
        "[TEST][initiate_password_reset_should_return_200_if_existing_email] {}",
        response_body
    );
    assert_eq!(
        response_body["message"],
        "If the email exists, a password reset link has been sent."
    );

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn initiate_password_reset_should_return_200_if_non_existing_email() {
    let mut app = RESTTestApp::new().await;
    let non_existing_email = get_random_email();

    Mock::given(path("/email/withTemplate"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let reset_body = json!({ "email": non_existing_email });
    let initiate_reset_response = app.post_initiate_password_reset(&reset_body).await;
    println!(
        "[TEST][initiate_password_reset_should_return_200_if_non_existing_email] {:?}",
        initiate_reset_response
    );
    assert_eq!(initiate_reset_response.status(), 200);

    let response_body: serde_json::Value = initiate_reset_response.json().await.unwrap();
    println!(
        "[TEST][initiate_password_reset_should_return_200_if_non_existing_email] {}",
        response_body
    );
    assert_eq!(
        response_body["message"],
        "If the email exists, a password reset link has been sent."
    );

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn initiate_password_reset_should_return_400_if_invalid_email() {
    let mut app = RESTTestApp::new().await;
    let reset_body = json!({ "email": "not-an-email" });

    Mock::given(path("/email/withTemplate"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let initiate_reset_response = app.post_initiate_password_reset(&reset_body).await;
    println!(
        "[TEST][initiate_password_reset_should_return_200_if_non_existing_email] {:?}",
        initiate_reset_response
    );
    assert_eq!(initiate_reset_response.status(), 400);

    let response_body: serde_json::Value = initiate_reset_response.json().await.unwrap();
    println!(
        "[TEST][initiate_password_reset_should_return_200_if_non_existing_email] {}",
        response_body
    );
    assert_eq!(response_body["error"], "Invalid email address");

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn reset_password_should_return_200_with_cookie_if_valid_token() {
    let mut app = RESTTestApp::new().await;
    let email = get_random_email();

    let signup_body = json!({
        "email": email,
        "password": "P@ssw0rd123",
        "requires2FA": false
    });
    let signup_response = app.post_signup(&signup_body).await;
    println!(
        "[TEST][initiate_password_reset_should_return_400_if_invalid_email] {:?}",
        signup_response
    );
    assert_eq!(signup_response.status(), 201);

    Mock::given(path("/email/withTemplate"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let reset_init_body = json!({ "email": email });
    app.post_initiate_password_reset(&reset_init_body).await;

    // Get the reset token (in a real scenario, this would be sent via email)
    let reset_token = app.get_password_reset_token(&email).await.unwrap();

    let reset_body = json!({
        "token": reset_token,
        "new_password": "NewP@ssw0rd123"
    });
    let reset_response = app.post_reset_password(&reset_body).await;
    println!(
        "[TEST][initiate_password_reset_should_return_400_if_invalid_email] {:?}",
        reset_response
    );
    assert_eq!(reset_response.status(), 200);

    let response_body: serde_json::Value = reset_response.json().await.unwrap();
    println!(
        "[TEST][initiate_password_reset_should_return_400_if_invalid_email] {}",
        response_body
    );
    assert_eq!(
        response_body["message"],
        "Password has been reset successfully.".to_string()
    );

    let login_body = json!({
        "email": email,
        "password": "NewP@ssw0rd123"
    });
    let login_response = app.post_login(&login_body).await;
    println!(
        "[TEST][initiate_password_reset_should_return_400_if_invalid_email] {:?}",
        login_response
    );
    assert_eq!(login_response.status(), 200);

    let auth_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("[ERROR][initiate_password_reset_should_return_400_if_invalid_email] No auth cookie found");

    let token = auth_cookie.value();
    println!(
        "[TEST][initiate_password_reset_should_return_400_if_invalid_email] {}",
        token
    );
    assert!(!token.is_empty());

    let claims = validate_token(app.app_state.banned_token_store.clone(), Secret::new(token.to_string()))
        .await
        .unwrap();
    println!(
        "[TEST][initiate_password_reset_should_return_400_if_invalid_email] {:?}",
        claims
    );
    assert_eq!(claims.sub.expose_secret(), &email);
    assert_eq!(claims.purpose, TokenPurpose::Auth);

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn reset_password_should_return_401_if_invalid_token_structure() {
    let mut app = RESTTestApp::new().await;
    let reset_body = json!({
        "token": "invalid_token",
        "new_password": "NewP@ssw0rd123"
    });

    Mock::given(path("/email/withTemplate"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let reset_response = app.post_reset_password(&reset_body).await;
    println!(
        "[TEST][reset_password_should_return_401_if_invalid_token_structure] {:?}",
        reset_response
    );
    assert_eq!(reset_response.status(), 401);

    let response_body: serde_json::Value = reset_response.json().await.unwrap();
    println!(
        "[TEST][reset_password_should_return_401_if_invalid_token_structure] {:?}",
        response_body
    );
    assert_eq!(response_body["error"], "Invalid auth token");

    app.clean_up().await.unwrap();
}

#[tokio::test]
async fn reset_password_should_return_400_if_weak_password() {
    let mut app = RESTTestApp::new().await;
    let email = get_random_email();

    let signup_body = json!({
        "email": email,
        "password": "P@ssw0rd123",
        "requires2FA": false
    });
    let signup_response = app.post_signup(&signup_body).await;
    println!(
        "[TEST][reset_password_should_return_400_if_weak_password] {:?}",
        signup_response
    );
    assert_eq!(signup_response.status(), 201);

    Mock::given(path("/email/withTemplate"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

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
    println!(
        "[TEST][reset_password_should_return_400_if_weak_password] {:?}",
        reset_response
    );
    assert_eq!(reset_response.status(), 400);

    let response_body: serde_json::Value = reset_response.json().await.unwrap();
    println!(
        "[TEST][reset_password_should_return_400_if_weak_password] {:?}",
        response_body
    );
    let error_msg = response_body["error"].clone();
    println!(
        "[TEST][reset_password_should_return_400_if_weak_password] {:?}",
        error_msg
    );
    println!(
        "[TEST][reset_password_should_return_400_if_weak_password] {:?}",
        error_msg.to_string()
    );
    assert!(error_msg.to_string().contains("Invalid Password"));

    app.clean_up().await.unwrap();
}
