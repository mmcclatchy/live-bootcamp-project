use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn signup_should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": random_email,
            "password": "password123",
        }),
        serde_json::json!({
            "email": random_email,
            "requires2FA": true,
        }),
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
async fn signup_should_return_201_if_valid_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let signup_request = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true,
    });
    let response = app.post_signup(&signup_request).await;
    assert_eq!(
        response.status().as_u16(),
        201,
        "Failed for input: {:?}",
        signup_request
    );
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let test_cases = [
        serde_json::json!({
            "email": "",
            "password": "password123",
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": "random_email",
            "password": "password123",
            "requires2FA": true,
        }),
        serde_json::json!({
            "email": "test@email.com",
            "password": "invalid",
            "requires2FA": true,
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let signup_request = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true,
    });
    app.post_signup(&signup_request).await;
    let response = app.post_signup(&signup_request).await;
    assert_eq!(
        response.status().as_u16(),
        409,
        "Failed for input: {:?}",
        signup_request
    );
}
