use std::collections::HashMap;

use color_eyre::eyre::{self, eyre};

use crate::{
    domain::{
        data_stores::{UserStore, UserStoreError},
        email::Email,
        password::Password,
        user::{DbUser, NewUser, User},
    },
    utils::auth::async_compute_password_hash,
};

#[derive(Clone, Debug)]
pub struct HashmapUserStore {
    // id: String,
    users: HashMap<Email, DbUser>,
}

impl HashmapUserStore {
    pub fn new() -> Self {
        HashmapUserStore {
            // id: uuid::Uuid::new_v4().to_string(),
            users: HashMap::new(),
        }
    }

    // pub fn get_id(&self) -> String {
    //     self.id.clone()
    // }
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: NewUser) -> Result<(), UserStoreError> {
        println!("[HashmapUserStore][add_user] {:?}", self);
        println!("[HashmapUserStore][add_user] {:?}", user);
        let email = user.email.clone();
        match self.users.get(&email) {
            Some(_) => Err(UserStoreError::UserAlreadyExists),
            None => {
                let password_hash = async_compute_password_hash(user.password.as_ref())
                    .await
                    .map_err(UserStoreError::UnexpectedError)?;
                let user = DbUser {
                    email: email.to_string(),
                    password_hash,
                    requires_2fa: user.requires_2fa,
                };
                self.users.insert(email, user);
                Ok(())
            }
        }
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        println!("[HashmapUserStore][get_user] {:?}", self);
        println!("[HashmapUserStore][get_user] {:?}", email);
        match self.users.get(email) {
            Some(user) => Ok((*user).clone().to_user()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn update_password(&mut self, email: &Email, password: Password) -> Result<(), UserStoreError> {
        let mut user = match self.users.get(email) {
            Some(user) => (*user).clone(),
            None => return Err(UserStoreError::UserNotFound),
        };
        user.update_password(&password)
            .await
            .map_err(|_| UserStoreError::UserNotFound)?;
        self.users.insert((*email).clone(), user);
        Ok(())
    }

    async fn validate_user(&self, email: &Email, password: &Password) -> eyre::Result<User> {
        let db_user = match self.users.get(email) {
            None => Err(eyre!("User Not Found")),
            // None => Err(UserStoreError::UserNotFound),
            Some(db_user) => Ok((*db_user).clone()),
        }?;
        db_user.verify_password(password)?;

        Ok(db_user.to_user())
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

    async fn get_test_password() -> Password {
        Password::parse("P@assword123".to_string()).await.unwrap()
    }

    async fn create_new_user() -> NewUser {
        NewUser {
            email: get_test_email(),
            password: get_test_password().await,
            requires_2fa: false,
        }
    }

    async fn create_db_user() -> DbUser {
        DbUser {
            email: get_test_email().to_string(),
            password_hash: get_test_password().await.to_string(),
            requires_2fa: false,
        }
    }

    async fn get_store_with_test_user() -> HashmapUserStore {
        let mut store = HashmapUserStore::new();
        let test_user = create_new_user().await;
        store.add_user(test_user).await.unwrap();
        store
    }

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore::new();
        let test_user = create_new_user().await;

        store.add_user(test_user.clone()).await.unwrap();

        let stored_user = match store.users.get(&test_user.email) {
            None => panic!("[ERROR] Failed to get user"),
            Some(db_user) => (*db_user).clone(),
        };

        let original_password = test_user.password.clone();

        println!("[test_add_user] Stored user: {:?}", stored_user);
        println!("[test_add_user] Test user: {:?}", test_user);
        println!("[test_add_user] Original password: {:?}", original_password);

        assert_eq!(stored_user.email, test_user.email.to_string());
        assert_eq!(stored_user.requires_2fa, test_user.requires_2fa);

        // assert!(stored_user.verify_password(&original_password).is_ok());
    }

    #[tokio::test]
    async fn test_get_user() {
        let store = get_store_with_test_user().await;
        let email = get_test_email();
        let output_user = store.get_user(&email).await.unwrap();
        let test_db_user = create_db_user().await;
        assert_eq!(output_user.email.to_string(), test_db_user.email);
        assert_eq!(output_user.requires_2fa, test_db_user.requires_2fa);
    }

    #[tokio::test]
    async fn test_validate_user_with_valid_input() {
        let store = get_store_with_test_user().await;
        let email = get_test_email();
        let password = get_test_password().await;
        assert!(store.validate_user(&email, &password).await.is_ok());
    }

    #[tokio::test]
    async fn test_validate_user_raises_error_when_user_not_found() {
        let store = HashmapUserStore::new();
        let email = get_test_email();
        let password = get_test_password().await;
        let result = store.validate_user(&email, &password).await;
        // assert_eq!(result, Err(UserStoreError::UserNotFound));
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn test_validate_user_raises_error_when_password_does_not_match() {
        let store = get_store_with_test_user().await;
        let email = get_test_email();
        let incorrect_password = Password::parse("Inc0rrect!".to_string()).await.unwrap();
        let result = store.validate_user(&email, &incorrect_password).await;
        // assert_eq!(result, Err(UserStoreError::InvalidCredentials));
        assert!(result.is_err())
    }
}
