use crate::domain::{
    data_stores::{PasswordResetTokenStore, TokenStoreError},
    email::Email,
};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct HashMapPasswordResetTokenStore {
    tokens: HashMap<Email, String>,
}

impl HashMapPasswordResetTokenStore {
    pub fn new() -> Self {
        Self { tokens: HashMap::new() }
    }
}

impl Default for HashMapPasswordResetTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl PasswordResetTokenStore for HashMapPasswordResetTokenStore {
    async fn add_token(&mut self, email: Email, token: String) -> Result<(), TokenStoreError> {
        self.tokens.insert(email, token);
        Ok(())
    }

    async fn get_token(&self, email: &Email) -> Result<String, TokenStoreError> {
        match self.tokens.get(email) {
            Some(token) => Ok(token.to_string()),
            None => Err(TokenStoreError::TokenNotFound),
        }
    }

    async fn remove_token(&mut self, email: &Email) -> Result<(), TokenStoreError> {
        self.tokens.remove(email).ok_or(TokenStoreError::TokenNotFound)?;
        Ok(())
    }
}
