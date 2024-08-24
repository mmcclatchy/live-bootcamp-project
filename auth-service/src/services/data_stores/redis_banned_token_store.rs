use std::{fmt, sync::Arc};

use redis::{Commands, Connection};
use tokio::sync::RwLock;

use crate::{
    domain::data_stores::{BannedTokenStore, TokenStoreError},
    utils::constants::TOKEN_TTL_SECONDS,
};

#[derive(Clone)]
pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl fmt::Debug for RedisBannedTokenStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RedisBannedTokenStore")
    }
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<(), TokenStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(&token);
        conn.set_ex(key, true, TOKEN_TTL_SECONDS as u64)
            .map_err(|_| TokenStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn check_token(&self, token: String) -> Result<(), TokenStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(&token);
        match conn.exists(&key).map_err(|_| TokenStoreError::UnexpectedError)? {
            true => Err(TokenStoreError::BannedToken),
            false => Ok(()),
        }
    }
}

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
