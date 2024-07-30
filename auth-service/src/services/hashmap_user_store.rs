use std::collections::HashMap;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    user::User,
};

#[derive(Clone)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn new() -> Self {
        HashmapUserStore {
            users: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let email = user.email.clone();
        match self.users.get(&email) {
            Some(_) => Err(UserStoreError::UserAlreadyExists),
            None => {
                self.users.insert(email, user);
                Ok(())
            }
        }
    }

    async fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok((*user).clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            None => Err(UserStoreError::UserNotFound),
            Some(user) => {
                if user.password == password {
                    return Ok(());
                }
                Err(UserStoreError::InvalidCredentials)
            }
        }
    }
}

impl Default for HashmapUserStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_USER_EMAIL: &str = "test@email.com";
    const TEST_USER_PASSWORD: &str = "password123";

    fn create_test_user() -> User {
        User {
            email: String::from(TEST_USER_EMAIL),
            password: String::from(TEST_USER_PASSWORD),
            requires_2fa: false,
        }
    }

    async fn get_store_with_test_user() -> HashmapUserStore {
        let mut store = HashmapUserStore::new();
        let test_user = create_test_user();
        store.add_user(test_user).await.unwrap();
        store
    }

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore::new();
        let test_user = create_test_user();
        store.add_user(test_user.clone()).await.unwrap();
        assert_eq!(store.users.get(&test_user.email), Some(&test_user));
    }

    #[tokio::test]
    async fn test_get_user() {
        let store = get_store_with_test_user().await;
        let test_user = create_test_user();

        let output_user = store.get_user(TEST_USER_EMAIL).await.unwrap();
        assert_eq!(output_user, test_user);
    }

    #[tokio::test]
    async fn test_validate_user_with_valid_input() {
        let store = get_store_with_test_user().await;
        assert!(store
            .validate_user(TEST_USER_EMAIL, TEST_USER_PASSWORD)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_validate_user_raises_error_when_user_not_found() {
        let store = HashmapUserStore::new();
        let result = store
            .validate_user(TEST_USER_EMAIL, TEST_USER_PASSWORD)
            .await;
        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user_raises_error_when_password_does_not_match() {
        let store = get_store_with_test_user().await;
        let result = store
            .validate_user(TEST_USER_EMAIL, "invalid_password")
            .await;
        assert_eq!(result, Err(UserStoreError::InvalidCredentials));
    }
}
