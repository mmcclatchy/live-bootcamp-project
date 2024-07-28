use std::collections::HashMap;

use crate::domain::user::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[allow(dead_code)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

#[allow(dead_code)]
impl HashmapUserStore {
    fn new() -> Self {
        HashmapUserStore {
            users: HashMap::new(),
        }
    }

    fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let email = user.email.clone();
        match self.users.get(&email) {
            Some(_) => Err(UserStoreError::UserAlreadyExists),
            None => {
                self.users.insert(email, user);
                Ok(())
            }
        }
    }

    fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok((*user).clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_USER_EMAIL: &str = "test@email.com";
    const TEST_USER_PASSWORD: &str = "password123";

    fn get_test_user() -> User {
        User {
            email: String::from(TEST_USER_EMAIL),
            password: String::from(TEST_USER_PASSWORD),
            requires_2fa: false,
        }
    }

    fn get_store_with_test_user() -> HashmapUserStore {
        let mut store = HashmapUserStore::new();
        let test_user = get_test_user();
        store.add_user(test_user).unwrap();
        store
    }

    #[tokio::test]
    async fn test_add_user() {
        let store = get_store_with_test_user();
        let test_user = get_test_user();
        assert_eq!(store.users.get(TEST_USER_EMAIL), Some(&test_user));
    }

    #[tokio::test]
    async fn test_get_user() {
        let store = get_store_with_test_user();
        let test_user = get_test_user();

        let output_user = store.get_user(TEST_USER_EMAIL).unwrap();
        assert_eq!(output_user, test_user);
    }

    #[tokio::test]
    async fn test_validate_user_with_valid_input() {
        let store = get_store_with_test_user();
        store
            .validate_user(TEST_USER_EMAIL, TEST_USER_PASSWORD)
            .unwrap();
    }

    #[tokio::test]
    async fn test_validate_user_raises_error_when_user_not_found() {
        let store = HashmapUserStore::new();
        let result = store.validate_user(TEST_USER_EMAIL, TEST_USER_PASSWORD);
        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user_raises_error_when_password_does_not_match() {
        let store = get_store_with_test_user();
        let result = store.validate_user(TEST_USER_EMAIL, "invalid_password");
        assert_eq!(result, Err(UserStoreError::InvalidCredentials));
    }
}
