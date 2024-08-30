use std::fmt;

use color_eyre::eyre;
use rand::Rng;
use secrecy::{ExposeSecret, Secret};
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use thiserror;
use uuid::Uuid;

use macros::SecretString;

use super::user::{NewUser, User};

use crate::domain::{email::Email, password::Password};

//************************  Traits  ************************//

#[async_trait::async_trait]
pub trait UserStore: Clone + Send + Sync + 'static + fmt::Debug {
    async fn add_user(&mut self, user: NewUser) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn update_password(&mut self, email: &Email, password: Password) -> Result<(), UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password) -> eyre::Result<User>;
}

#[async_trait::async_trait]
pub trait BannedTokenStore: Clone + Send + Sync + 'static + fmt::Debug {
    async fn add_token(&mut self, token: Secret<String>) -> Result<(), TokenStoreError>;
    async fn check_token(&self, token: Secret<String>) -> Result<(), TokenStoreError>;
}

#[async_trait::async_trait]
pub trait TwoFACodeStore: Clone + Send + Sync + 'static + fmt::Debug {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;

    async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[async_trait::async_trait]
pub trait PasswordResetTokenStore: Clone + Send + Sync + 'static + fmt::Debug {
    async fn add_token(&mut self, email: Email, token: String) -> Result<(), TokenStoreError>;
    async fn get_token(&self, email: &Email) -> Result<String, TokenStoreError>;
    async fn remove_token(&mut self, email: &Email) -> Result<(), TokenStoreError>;
}

//************************  Traits  ************************//

//************************  Enums   ************************//

#[derive(Debug, thiserror::Error)]
pub enum UserStoreError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Unexpected error")]
    UnexpectedError(#[source] eyre::Report),
}

#[derive(Debug, thiserror::Error)]
pub enum TokenStoreError {
    #[error("Banned token")]
    BannedToken,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token not found")]
    TokenNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] eyre::Report),
}

#[derive(Debug, thiserror::Error)]
pub enum TwoFACodeStoreError {
    #[error("Login attempt id not found")]
    LoginAttemptIdNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] eyre::Report),
}

//************************  Enums   ************************//

//***********************  Structs  ************************//

#[derive(Clone, Debug, Deserialize, SecretString)]
pub struct LoginAttemptId(Secret<String>);

impl LoginAttemptId {
    pub fn parse(id: Secret<String>) -> Result<Self, String> {
        match Uuid::parse_str(id.expose_secret()) {
            Err(_) => Err(String::from("Invalid Login Attempt Id")),
            Ok(_) => Ok(Self(id)),
        }
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        Self(Secret::new(Uuid::new_v4().to_string()))
    }
}

#[derive(Clone, Debug, Deserialize, SecretString)]
#[secret_string(field_name = "2FACode")]
pub struct TwoFACode(Secret<String>);

impl TwoFACode {
    pub fn parse(code: Secret<String>) -> Result<Self, String> {
        match code.expose_secret().len() == 6 && code.expose_secret().chars().all(char::is_numeric) {
            false => Err("Failed to parse Two-Factor Authorization Code".to_string()),
            true => Ok(Self(code)),
        }
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let mut range = rand::thread_rng();
        let code = (0..6).map(|_| range.gen_range(0..10).to_string()).collect();
        Self(Secret::new(code))
    }
}

//***********************  Structs  ************************//

//***********************   Tests   ************************//

#[cfg(test)]
mod login_attempt_id_tests {
    use super::*;

    #[test]
    fn test_login_attempt_id_parse_valid() {
        let valid_uuid = Uuid::new_v4().to_string();
        let result = LoginAttemptId::parse(Secret::new(valid_uuid.clone()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.expose_secret(), &valid_uuid);
    }

    #[test]
    fn test_login_attempt_id_parse_invalid() {
        let invalid_uuid = "not-a-uuid".to_string();
        let result = LoginAttemptId::parse(Secret::new(invalid_uuid));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid Login Attempt Id");
    }
}

#[cfg(test)]
mod two_fa_code_tests {
    use super::*;

    fn get_2fa_code_secret(code: &str) -> Secret<String> {
        Secret::new(code.to_string())
    }

    #[test]
    fn test_two_fa_code_parse_valid() {
        let valid_code = get_2fa_code_secret("123456");
        let result = TwoFACode::parse(valid_code.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.expose_secret(), valid_code.expose_secret());
    }

    #[test]
    fn test_two_fa_code_parse_invalid_length() {
        let invalid_code = get_2fa_code_secret("12345");
        let result = TwoFACode::parse(invalid_code);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Failed to parse Two-Factor Authorization Code");
    }

    #[test]
    fn test_two_fa_code_parse_invalid_characters() {
        let invalid_code = get_2fa_code_secret("12345a");
        let result = TwoFACode::parse(invalid_code);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Failed to parse Two-Factor Authorization Code");
    }

    #[test]
    fn test_two_fa_code_default() {
        let code = TwoFACode::default();
        assert_eq!(code.0.expose_secret().len(), 6);
        assert!(code.0.expose_secret().chars().all(char::is_numeric));
    }
}
