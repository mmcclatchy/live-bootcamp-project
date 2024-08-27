use argon2::{Argon2, PasswordHash, PasswordVerifier};
use color_eyre::eyre::{Context, Result};
use secrecy::{ExposeSecret, Secret};

use crate::{
    domain::{email::Email, password::Password},
    utils::auth::async_compute_password_hash,
};

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

#[derive(Clone, Debug)]
pub struct DbUser {
    pub email: Secret<String>,
    pub password_hash: Secret<String>,
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
    #[tracing::instrument(name = "Verify User Password", skip_all)]
    pub fn verify_password(&self, password_attempt: &Password) -> Result<()> {
        let parsed_hash = PasswordHash::new(self.password_hash.expose_secret())?;

        Argon2::default()
            .verify_password(password_attempt.as_ref().expose_secret().as_bytes(), &parsed_hash)
            .wrap_err("Failed to verify password hash")
    }

    pub fn to_user(&self) -> User {
        User {
            email: Email::parse(self.email.clone()).expect("[ERROR] Invalid email in database"),
            requires_2fa: self.requires_2fa,
        }
    }

    // TODO: This is only used in HashMapUserStore. Remove when gRPC is updated to use PostgresUserStore
    pub async fn update_password(&mut self, password: &Password) -> Result<()> {
        let new_password_hash = async_compute_password_hash(password.as_ref().clone()).await?;
        self.password_hash = new_password_hash;
        Ok(())
    }
}

impl PartialEq for DbUser {
    fn eq(&self, other: &Self) -> bool {
        self.password_hash.expose_secret() == other.password_hash.expose_secret()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::password_hash::{PasswordHasher, SaltString};
    use color_eyre::eyre;

    fn str_to_valid_email(email: &str) -> Email {
        Email::parse(Secret::new(email.to_string())).unwrap()
    }

    fn str_to_email_secret(email: &str) -> Secret<String> {
        str_to_valid_email(email).as_ref().clone()
    }

    async fn str_to_password_result(password: &str) -> eyre::Result<Password> {
        Password::parse(Secret::new(password.to_string())).await
    }

    async fn str_to_valid_password(password: &str) -> Password {
        str_to_password_result(password).await.unwrap()
    }

    fn create_password_hash(password: &str) -> Secret<String> {
        let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        Secret::new(password_hash)
    }

    #[tokio::test]
    async fn test_new_user_creation() {
        let email = str_to_valid_email("test@example.com");
        let password = str_to_valid_password("P@ssw0rd123").await;
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
            email: str_to_email_secret("test@example.com"),
            password_hash,
            requires_2fa: false,
        };

        let password_attempt = Password::parse(Secret::new(password.to_string())).await.unwrap();
        assert!(db_user.verify_password(&password_attempt).is_ok());
    }

    #[tokio::test]
    async fn test_db_user_verify_password_failure() {
        let correct_password = "P@ssw0rd123";
        let password_hash = create_password_hash(correct_password);

        let db_user = DbUser {
            email: str_to_email_secret("test@example.com"),
            password_hash,
            requires_2fa: false,
        };

        let wrong_password = Password::parse(Secret::new("Wr0ngP@ssw0rd".to_string())).await.unwrap();
        let result = db_user.verify_password(&wrong_password);

        println!("Verify Password Result: {:?}", result);
        assert!(result.is_err_and(|report| report.to_string() == "Failed to verify password hash"));
    }

    #[tokio::test]
    async fn test_db_user_to_user() {
        let db_user = DbUser {
            email: str_to_email_secret("test@example.com"),
            password_hash: Secret::new("some_hash".to_string()),
            requires_2fa: true,
        };

        let user = db_user.to_user();

        assert_eq!(user.email.as_ref().expose_secret(), "test@example.com");
        assert_eq!(user.requires_2fa, true);
    }

    #[tokio::test]
    #[should_panic(expected = "[ERROR] Invalid email in database")]
    async fn test_db_user_to_user_with_invalid_email() {
        let db_user = DbUser {
            email: Secret::new("invalid_email".to_string()),
            password_hash: Secret::new("some_hash".to_string()),
            requires_2fa: false,
        };

        db_user.to_user();
    }
}
