use std::sync::Arc;

use redis::{aio::ConnectionManager, AsyncCommands};
use secrecy::ExposeSecret;
use tokio::sync::RwLock;

use crate::domain::{
    data_stores::{PasswordResetTokenStore, TokenStoreError},
    email::Email,
};

#[derive(Clone)]
pub struct RedisPasswordResetTokenStore {
    conn: Arc<RwLock<ConnectionManager>>,
}

impl RedisPasswordResetTokenStore {
    pub fn new(conn: Arc<RwLock<ConnectionManager>>) -> Self {
        Self { conn }
    }
}

impl std::fmt::Debug for RedisPasswordResetTokenStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RedisPasswordResetTokenStore")
    }
}

#[async_trait::async_trait]
impl PasswordResetTokenStore for RedisPasswordResetTokenStore {
    async fn add_token(&mut self, email: Email, token: String) -> Result<(), TokenStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(&email);

        conn.set_ex(key, token, TEN_MINUTES_IN_SECONDS)
            .await
            .map_err(|e| TokenStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    async fn remove_token(&mut self, email: &Email) -> Result<(), TokenStoreError> {
        let key = get_key(email);
        let mut conn = self.conn.write().await;

        conn.del(key)
            .await
            .map_err(|e| TokenStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    async fn get_token(&self, email: &Email) -> Result<String, TokenStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(email);
        let token: String = conn.get(key).await.map_err(|_| TokenStoreError::TokenNotFound)?;

        Ok(token)
    }
}

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const PASSWORD_RESET_PREFIX: &str = "password_reset:";

fn get_key(email: &Email) -> String {
    format!("{}{}", PASSWORD_RESET_PREFIX, email.as_ref().expose_secret())
}
