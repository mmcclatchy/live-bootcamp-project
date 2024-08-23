use argon2::{Argon2, PasswordHash, PasswordVerifier};

use crate::{
    domain::{email::Email, password::Password},
    utils::auth::async_compute_password_hash,
};

use super::data_stores::UserStoreError;

#[derive(Clone, Debug, PartialEq)]
pub struct NewUser {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub email: Email,
    pub requires_2fa: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DbUser {
    pub email: String,
    pub password_hash: String,
    pub requires_2fa: bool,
}

impl NewUser {
    pub fn new(email: Email, password: Password, requires_2fa: bool) -> Self {
        Self {
            email,
            password,
            requires_2fa,
        }
    }
}

impl DbUser {
    pub fn verify_password(&self, password_attempt: &Password) -> Result<(), UserStoreError> {
        let parsed_hash = PasswordHash::new(&self.password_hash).map_err(|_| UserStoreError::UnexpectedError)?;

        Argon2::default()
            .verify_password(password_attempt.as_ref().as_bytes(), &parsed_hash)
            .map_err(|_| UserStoreError::InvalidCredentials)
    }

    pub fn to_user(&self) -> User {
        User {
            email: Email::parse(self.email.clone()).expect("[ERROR] Invalid email in database"),
            requires_2fa: self.requires_2fa,
        }
    }

    // TODO: This is only used in HashMapUserStore. Remove when gRPC is updated to use PostgresUserStore
    pub async fn update_password(&mut self, password: &Password) -> Result<(), UserStoreError> {
        let new_password_hash = async_compute_password_hash(password.as_ref())
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;
        self.password_hash = new_password_hash;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::password_hash::{PasswordHasher, SaltString};

    fn create_password_hash(password: &str) -> String {
        let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    #[tokio::test]
    async fn test_new_user_creation() {
        let email = Email::parse("test@example.com".to_string()).unwrap();
        let password = Password::parse("P@ssw0rd123".to_string()).await.unwrap();
        let requires_2fa = true;

        let new_user = NewUser::new(email.clone(), password.clone(), requires_2fa);

        assert_eq!(new_user.email, email);
        assert_eq!(new_user.password, password);
        assert_eq!(new_user.requires_2fa, requires_2fa);
    }

    #[tokio::test]
    async fn test_db_user_verify_password_success() {
        let password = "P@ssw0rd123";
        let password_hash = create_password_hash(password);

        let db_user = DbUser {
            email: "test@example.com".to_string(),
            password_hash,
            requires_2fa: false,
        };

        let password_attempt = Password::parse(password.to_string()).await.unwrap();
        assert!(db_user.verify_password(&password_attempt).is_ok());
    }

    #[tokio::test]
    async fn test_db_user_verify_password_failure() {
        let correct_password = "P@ssw0rd123";
        let password_hash = create_password_hash(correct_password);

        let db_user = DbUser {
            email: "test@example.com".to_string(),
            password_hash,
            requires_2fa: false,
        };

        let wrong_password = Password::parse("Wr0ngP@ssw0rd".to_string()).await.unwrap();
        assert_eq!(
            db_user.verify_password(&wrong_password),
            Err(UserStoreError::InvalidCredentials)
        );
    }

    #[tokio::test]
    async fn test_db_user_to_user() {
        let db_user = DbUser {
            email: "test@example.com".to_string(),
            password_hash: "some_hash".to_string(),
            requires_2fa: true,
        };

        let user = db_user.to_user();

        assert_eq!(user.email.as_ref(), "test@example.com");
        assert_eq!(user.requires_2fa, true);
    }

    // #[tokio::test]
    // async fn test_db_user_update_password() {
    //     let mut db_user = DbUser {
    //         email: "test@example.com".to_string(),
    //         password_hash: "old_hash".to_string(),
    //         requires_2fa: false,
    //     };

    //     let new_password = Password::parse("NewP@ssw0rd123".to_string()).await.unwrap();
    //     assert!(db_user.update_password(&new_password).is_ok());
    //     assert_eq!(db_user.password_hash, new_password.to_string());
    // }

    #[tokio::test]
    #[should_panic(expected = "[ERROR] Invalid email in database")]
    async fn test_db_user_to_user_with_invalid_email() {
        let db_user = DbUser {
            email: "invalid_email".to_string(),
            password_hash: "some_hash".to_string(),
            requires_2fa: false,
        };

        db_user.to_user();
    }
}
