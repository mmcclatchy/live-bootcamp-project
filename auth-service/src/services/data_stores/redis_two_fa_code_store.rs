use std::{fmt::Debug, sync::Arc};

use color_eyre::eyre::eyre;
use redis::{aio::ConnectionManager, AsyncCommands};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json};
use tokio::sync::RwLock;

use crate::{
    domain::{
        data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
        email::Email,
    },
    utils::constants::Time,
};

#[derive(Clone)]
pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<ConnectionManager>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<ConnectionManager>>) -> Self {
        Self { conn }
    }
}

impl Debug for RedisTwoFACodeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RedisTwoFACodeStore")
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(&email);
        let two_fa_tuple = TwoFATuple(login_attempt_id.expose_secret_string(), code.expose_secret_string());
        let two_fa_json = json!(two_fa_tuple).to_string();

        conn.set_ex(key, two_fa_json, Time::Minutes10 as u64)
            .await
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(email);
        let mut conn = self.conn.write().await;

        conn.del(key)
            .await
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(email);

        let two_fa_json: String = conn
            .get(key)
            .await
            .map_err(|_| TwoFACodeStoreError::LoginAttemptIdNotFound)?;

        let two_fa_tuple: TwoFATuple =
            from_str(&two_fa_json).map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;

        two_fa_tuple
            .destructure()
            .map_err(|err_msg| TwoFACodeStoreError::UnexpectedError(eyre!(err_msg)))
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

impl TwoFATuple {
    fn destructure(&self) -> Result<(LoginAttemptId, TwoFACode), String> {
        let attempt_id = LoginAttemptId::parse(Secret::new(self.0.clone()))?;
        let code = TwoFACode::parse(Secret::new(self.1.clone()))?;
        Ok((attempt_id, code))
    }
}

const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref().expose_secret())
}
