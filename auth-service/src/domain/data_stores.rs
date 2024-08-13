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
