use std::{fmt, sync::Arc};

use redis::{aio::ConnectionManager, AsyncCommands};
use secrecy::{ExposeSecret, Secret};
use tokio::sync::RwLock;

use crate::{
    domain::data_stores::{BannedTokenStore, TokenStoreError},
    utils::constants::TOKEN_TTL_SECONDS,
};

#[derive(Clone)]
pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<ConnectionManager>>,
}

impl fmt::Debug for RedisBannedTokenStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RedisBannedTokenStore")
    }
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<ConnectionManager>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(name = "RedisBannedTokenStore Add Token")]
    async fn add_token(&mut self, token: Secret<String>) -> Result<(), TokenStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(&token);

        conn.set_ex(key, true, TOKEN_TTL_SECONDS as u64)
            .await
            .map_err(|e| TokenStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "RedisBannedTokenStore Check Token")]
    async fn check_token(&self, token: Secret<String>) -> Result<(), TokenStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(&token);

        match conn
            .exists(&key)
            .await
            .map_err(|e| TokenStoreError::UnexpectedError(e.into()))?
        {
            true => Err(TokenStoreError::BannedToken),
            false => Ok(()),
        }
    }
}

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &Secret<String>) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token.expose_secret())
}
