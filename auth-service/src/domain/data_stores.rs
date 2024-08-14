use std::fmt;

use rand::Rng;
use uuid::Uuid;

use super::user::User;

use crate::domain::{email::Email, password::Password};

#[async_trait::async_trait]
pub trait UserStore: Clone + Send + Sync + 'static {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<User, UserStoreError>;
}

#[async_trait::async_trait]
pub trait BannedTokenStore: Clone + Send + Sync + 'static {
    async fn add_token(&mut self, token: String) -> Result<(), TokenStoreError>;
    async fn check_token(&self, token: String) -> Result<(), TokenStoreError>;
    async fn expire_tokens(&mut self) -> Result<(), TokenStoreError>;
}

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
    UnexpectedError,
}

#[async_trait::async_trait]
pub trait TwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub struct TwoFACode(String);

impl TwoFACode {
    pub fn parse(code: String) -> Result<Self, String> {
        match code.len() != 6 && !code.chars().all(char::is_numeric) {
            false => Err("Incorrect Two-Factor Authorization Code".to_string()),
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
