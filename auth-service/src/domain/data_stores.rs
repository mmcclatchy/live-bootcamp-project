use std::fmt;

use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::user::{NewUser, User};

use crate::domain::{email::Email, password::Password};

//************************  Traits  ************************//

#[async_trait::async_trait]
pub trait UserStore: Clone + Send + Sync + 'static + fmt::Debug {
    async fn add_user(&mut self, user: NewUser) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn update_password(&mut self, email: &Email, password: Password) -> Result<(), UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password) -> Result<User, UserStoreError>;
}

#[async_trait::async_trait]
pub trait BannedTokenStore: Clone + Send + Sync + 'static + fmt::Debug {
    async fn add_token(&mut self, token: String) -> Result<(), TokenStoreError>;
    async fn check_token(&self, token: String) -> Result<(), TokenStoreError>;
}

#[async_trait::async_trait]
pub trait TwoFACodeStore: Clone + Send + Sync + 'static {
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

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Debug, PartialEq)]
pub enum TokenStoreError {
    BannedToken,
    InvalidToken,
    TokenNotFound,
    UnexpectedError,
}

#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}

//************************  Enums   ************************//

//***********************  Structs  ************************//

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, String> {
        match Uuid::parse_str(&id) {
            Err(_) => Err(String::from("Invalid Login Attempt Id")),
            Ok(_) => Ok(Self(id)),
        }
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LoginAttemptId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TwoFACode(String);

impl TwoFACode {
    pub fn parse(code: String) -> Result<Self, String> {
        match code.len() == 6 && code.chars().all(char::is_numeric) {
            false => Err("Failed to parse Two-Factor Authorization Code".to_string()),
            true => Ok(Self(code)),
        }
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let mut range = rand::thread_rng();
        let code = (0..6).map(|_| range.gen_range(0..10).to_string()).collect();
        Self(code)
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TwoFACode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
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
        let result = LoginAttemptId::parse(valid_uuid.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, valid_uuid);
    }

    #[test]
    fn test_login_attempt_id_parse_invalid() {
        let invalid_uuid = "not-a-uuid".to_string();
        let result = LoginAttemptId::parse(invalid_uuid);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid Login Attempt Id");
    }

    #[test]
    fn test_login_attempt_id_default() {
        let id = LoginAttemptId::default();
        assert!(Uuid::parse_str(&id.as_ref()).is_ok());
    }
}

#[cfg(test)]
mod two_fa_code_tests {
    use super::*;

    #[test]
    fn test_two_fa_code_parse_valid() {
        let valid_code = "123456".to_string();
        let result = TwoFACode::parse(valid_code.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0, valid_code);
    }

    #[test]
    fn test_two_fa_code_parse_invalid_length() {
        let invalid_code = "12345".to_string();
        let result = TwoFACode::parse(invalid_code);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Failed to parse Two-Factor Authorization Code");
    }

    #[test]
    fn test_two_fa_code_parse_invalid_characters() {
        let invalid_code = "12345a".to_string();
        let result = TwoFACode::parse(invalid_code);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Failed to parse Two-Factor Authorization Code");
    }

    #[test]
    fn test_two_fa_code_default() {
        let code = TwoFACode::default();
        assert_eq!(code.0.len(), 6);
        assert!(code.0.chars().all(char::is_numeric));
    }
}
