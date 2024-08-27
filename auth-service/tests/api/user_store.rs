use auth_service::domain::{
    data_stores::{UserStore, UserStoreError},
    email::Email,
    password::Password,
    user::NewUser,
};
use secrecy::Secret;

use crate::helpers::RESTTestApp;

fn str_to_valid_email(email: &str) -> Email {
    Email::parse(Secret::new(email.to_string())).unwrap()
}

#[sqlx::test]
async fn test_add_user() {
    let mut app = RESTTestApp::new().await;
    let mut user_store = app.app_state.user_store.write().await;

    let email = str_to_valid_email("test@example.com");
    let password = Password::parse(Secret::new("P@ssw0rd123".to_string())).await.unwrap();
    let new_user = NewUser::new(email.clone(), password.clone(), false);

    let result = user_store.add_user(new_user).await;
    assert!(result.is_ok());

    let new_user = NewUser::new(email, password, false);
    let result = user_store.add_user(new_user).await;
    assert!(matches!(result, Err(UserStoreError::UserAlreadyExists)));

    drop(user_store);
    app.clean_up().await.unwrap();
}

#[sqlx::test]
async fn test_get_user() {
    let mut app = RESTTestApp::new().await;
    let mut user_store = app.app_state.user_store.write().await;

    let email = str_to_valid_email("test@example.com");
    let password = Password::parse(Secret::new("P@ssw0rd123".to_string())).await.unwrap();
    let new_user = NewUser::new(email.clone(), password, false);

    user_store.add_user(new_user).await.unwrap();

    let result = user_store.get_user(&email).await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.email, email);

    let non_existent_email = str_to_valid_email("nonexistent@example.com");
    let result = user_store.get_user(&non_existent_email).await;
    assert!(matches!(result, Err(UserStoreError::UserNotFound)));

    drop(user_store);
    app.clean_up().await.unwrap();
}

#[sqlx::test]
async fn test_update_password() {
    let mut app = RESTTestApp::new().await;
    let mut user_store = app.app_state.user_store.write().await;

    let email = str_to_valid_email("test@example.com");
    let password = Password::parse(Secret::new("P@ssw0rd123".to_string())).await.unwrap();
    let new_user = NewUser::new(email.clone(), password.clone(), false);

    user_store.add_user(new_user).await.unwrap();

    let new_password = Password::parse(Secret::new("NewP@ssw0rd123".to_string()))
        .await
        .unwrap();
    let result = user_store.update_password(&email, new_password.clone()).await;
    assert!(result.is_ok());

    let result = user_store.validate_user(&email, &new_password).await;
    assert!(result.is_ok());

    let old_password = password;
    let result = user_store.validate_user(&email, &old_password).await;
    assert!(matches!(result, Err(e) if e.to_string() == "Failed to verify password hash"));

    let non_existent_email = str_to_valid_email("nonexistent@example.com");
    let result = user_store.update_password(&non_existent_email, new_password).await;
    assert!(matches!(result, Err(UserStoreError::UserNotFound)));

    drop(user_store);
    app.clean_up().await.unwrap();
}

#[sqlx::test]
async fn test_validate_user() {
    let mut app = RESTTestApp::new().await;
    let mut user_store = app.app_state.user_store.write().await;

    let email = str_to_valid_email("test@example.com");
    let password = Password::parse(Secret::new("P@ssw0rd123".to_string())).await.unwrap();
    let new_user = NewUser::new(email.clone(), password.clone(), false);

    user_store.add_user(new_user).await.unwrap();

    let result = user_store.validate_user(&email, &password).await;
    assert!(result.is_ok());

    let wrong_password = Password::parse(Secret::new("WrongP@ssw0rd".to_string())).await.unwrap();
    let result = user_store.validate_user(&email, &wrong_password).await;
    assert!(matches!(result, Err(e) if e.to_string() == "Failed to verify password hash"));

    let non_existent_email = str_to_valid_email("nonexistent@example.com");
    let result = user_store.validate_user(&non_existent_email, &password).await;
    assert!(
        matches!(result, Err(e) if e.to_string() == "no rows returned by a query that expected to return at least one row")
    );

    drop(user_store);
    app.clean_up().await.unwrap();
}
