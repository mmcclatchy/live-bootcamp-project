use std::collections::HashMap;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    email::Email,
    password::Password,
    user::User,
};

#[derive(Clone, Debug)]
pub struct HashmapUserStore {
    id: String,
    users: HashMap<Email, User>,
}

impl HashmapUserStore {
    pub fn new() -> Self {
        HashmapUserStore {
            id: uuid::Uuid::new_v4().to_string(),
            users: HashMap::new(),
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        println!("[HashmapUserStore][add_user] {:?}", self);
        println!("[HashmapUserStore][add_user] {:?}", user);
        let email = user.email.clone();
        match self.users.get(&email) {
            Some(_) => Err(UserStoreError::UserAlreadyExists),
            None => {
                self.users.insert(email, user);
                Ok(())
            }
        }
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        println!("[HashmapUserStore][get_user] {:?}", self);
        println!("[HashmapUserStore][get_user] {:?}", email);
        match self.users.get(email) {
            Some(user) => Ok((*user).clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            None => Err(UserStoreError::UserNotFound),
            Some(user) => {
                if user.password == *password {
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

    fn get_test_email() -> Email {
        Email::parse("test@email.com".to_string()).unwrap()
    }

    fn get_test_password() -> Password {
        Password::parse("P@assword123".to_string()).unwrap()
    }

    fn create_test_user() -> User {
        User {
            email: get_test_email(),
            password: get_test_password(),
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
        let email = get_test_email();
        let output_user = store.get_user(&email).await.unwrap();
        assert_eq!(output_user, test_user);
    }

    #[tokio::test]
    async fn test_validate_user_with_valid_input() {
        let store = get_store_with_test_user().await;
        let email = get_test_email();
        let password = get_test_password();
        assert!(store.validate_user(&email, &password).await.is_ok());
    }

    #[tokio::test]
    async fn test_validate_user_raises_error_when_user_not_found() {
        let store = HashmapUserStore::new();
        let email = get_test_email();
        let password = get_test_password();
        let result = store.validate_user(&email, &password).await;
        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user_raises_error_when_password_does_not_match() {
        let store = get_store_with_test_user().await;
        let email = get_test_email();
        let incorrect_password = Password::parse("Inc0rrect!".to_string()).unwrap();
        let result = store.validate_user(&email, &incorrect_password).await;
        assert_eq!(result, Err(UserStoreError::InvalidCredentials));
    }
}
