use std::{fmt::Debug, sync::Arc};

use color_eyre::eyre::eyre;
use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json};
use tokio::sync::RwLock;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Clone)]
pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
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
        let two_fa_tuple = TwoFATuple(login_attempt_id.to_string(), code.to_string());
        let two_fa_json = json!(two_fa_tuple).to_string();

        println!("[Redis2FACodeStore][add_code] {key}");
        println!("[Redis2FACodeStore][add_code] {two_fa_json}");

        conn.set_ex(key, two_fa_json, TEN_MINUTES_IN_SECONDS)
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;

        println!("[Redis2FACodeStore][add_code] 2FA Code Set");

        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(email);
        let mut conn = self.conn.write().await;
        conn.del(key)
            .map_err(|e| TwoFACodeStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    async fn get_code(&self, email: &Email) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(email);
        let two_fa_json: String = conn.get(key).map_err(|_| TwoFACodeStoreError::LoginAttemptIdNotFound)?;
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
        let attempt_id = LoginAttemptId::parse(self.0.clone())?;
        let code = TwoFACode::parse(self.1.clone())?;
        Ok((attempt_id, code))
    }
}

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
