use std::collections::HashMap;

use secrecy::{ExposeSecret, Secret};

use crate::domain::data_stores::{BannedTokenStore, TokenStoreError};
use crate::utils::{auth::validate_token_structure, constants::Epoch};

#[derive(Clone, Debug)]
pub struct HashMapBannedTokenStore {
    tokens: HashMap<String, Epoch>,
}

impl HashMapBannedTokenStore {
    pub fn new() -> Self {
        Self { tokens: HashMap::new() }
    }
}

impl Default for HashMapBannedTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for HashMapBannedTokenStore {
    async fn add_token(&mut self, token: Secret<String>) -> Result<(), TokenStoreError> {
        let claims = validate_token_structure(token.expose_secret())
            .await
            .map_err(|_| TokenStoreError::InvalidToken)?;
        self.tokens.insert(token.expose_secret().clone(), claims.exp);
        Ok(())
    }

    async fn check_token(&self, token: Secret<String>) -> Result<(), TokenStoreError> {
        let response = match self.tokens.get(token.expose_secret()) {
            Some(_) => Err(TokenStoreError::BannedToken),
            None => Ok(()),
        };
        response
    }
}

#[cfg(test)]
mod tests {
    use secrecy::Secret;

    use crate::{domain::email::Email, utils::auth::generate_auth_token};

    use super::*;

    fn create_token(email: &str) -> Secret<String> {
        let email = Email::parse(Secret::new(email.to_string())).unwrap();
        generate_auth_token(&email).unwrap()
    }

    #[tokio::test]
    async fn test_add_token() {
        let mut store = HashMapBannedTokenStore::new();
        let token = create_token("test@example.com");
        store.add_token(token.clone()).await.unwrap();

        assert_eq!(store.tokens.len(), 1);
        assert!(store.tokens.get(token.expose_secret()).is_some());
    }

    #[tokio::test]
    async fn test_check_token_banned() {
        let mut store = HashMapBannedTokenStore::new();
        let token = create_token("test@example.com");
        store.add_token(token.clone()).await.unwrap();

        let result = store.check_token(token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_check_token_not_banned() {
        let store = HashMapBannedTokenStore::new();
        let token = create_token("test@example.com");

        let result = store.check_token(token).await;
        assert!(result.is_ok());
    }
}
