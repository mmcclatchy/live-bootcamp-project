use std::collections::HashMap;

use chrono::Utc;

use crate::domain::data_stores::{BannedTokenStore, TokenStoreError};
use crate::utils::{auth::validate_token, constants::Epoch};

#[derive(Clone, Debug)]
pub struct HashMapBannedTokenStore {
    tokens: HashMap<String, Epoch>,
}

impl HashMapBannedTokenStore {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }
}

impl Default for HashMapBannedTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for HashMapBannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<(), TokenStoreError> {
        let claims = validate_token(&token)
            .await
            .map_err(|_| TokenStoreError::InvalidToken)?;
        self.tokens.insert(token.clone(), claims.exp);
        Ok(())
    }

    async fn check_token(&self, token: String) -> Result<(), TokenStoreError> {
        let response = match self.tokens.get(&token) {
            Some(_) => Err(TokenStoreError::BannedToken),
            None => Ok(()),
        };
        response
    }

    async fn expire_tokens(&mut self) -> Result<(), TokenStoreError> {
        let epoch = Utc::now().timestamp() as Epoch;
        let expired_tokens: Vec<String> = self
            .tokens
            .iter()
            .filter_map(|(token, &exp)| {
                if exp <= epoch {
                    Some(token.clone())
                } else {
                    None
                }
            })
            .collect();
        for token in &expired_tokens {
            self.tokens.remove(token.as_str());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{domain::email::Email, utils::auth::generate_auth_token};

    use super::*;
    use chrono::Utc;

    fn create_token(email: &str) -> String {
        let email = Email::parse(email.to_string()).unwrap();
        let token = generate_auth_token(&email).unwrap();
        token
    }

    #[tokio::test]
    async fn test_add_token() {
        let mut store = HashMapBannedTokenStore::new();
        let token = create_token("test@example.com");
        store.add_token(token.clone()).await.unwrap();

        assert_eq!(store.tokens.len(), 1);
        assert!(store.tokens.get(&token).is_some());
    }

    #[tokio::test]
    async fn test_check_token_banned() {
        let mut store = HashMapBannedTokenStore::new();
        let token = create_token("test@example.com");
        store.add_token(token.clone()).await.unwrap();

        let result = store.check_token(token).await;
        assert_eq!(result, Err(TokenStoreError::BannedToken));
    }

    #[tokio::test]
    async fn test_check_token_not_banned() {
        let store = HashMapBannedTokenStore::new();
        let token = create_token("test@example.com");

        let result = store.check_token(token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_expire_tokens() {
        let mut store = HashMapBannedTokenStore::new();
        let expired_token = create_token("expired@example.com");
        let valid_token = create_token("valid@example.com");

        store.tokens.insert(
            expired_token.clone(),
            Utc::now().timestamp() as Epoch - 3600,
        );
        store
            .tokens
            .insert(valid_token.clone(), Utc::now().timestamp() as Epoch + 3600);

        println!("Before Expiring: {:?}", store);
        store.expire_tokens().await.unwrap();
        println!("After Expiring:  {:?}", store);

        assert_eq!(store.tokens.len(), 1);
        assert!(!store.tokens.contains_key(&expired_token));
        assert!(store.tokens.contains_key(&valid_token));
    }

    #[tokio::test]
    async fn test_expire_tokens_empty_store() {
        let mut store = HashMapBannedTokenStore::new();

        let result = store.expire_tokens().await;

        assert!(result.is_ok());
        assert_eq!(store.tokens.len(), 0);
    }

    #[tokio::test]
    async fn test_expire_tokens_all_valid() {
        let mut store = HashMapBannedTokenStore::new();
        let valid_token1 = create_token("test1@example.com");
        let valid_token2 = create_token("test2@example.com");

        // Manually set expiration times in the future
        store
            .tokens
            .insert(valid_token1.clone(), Utc::now().timestamp() as Epoch + 3600);
        store
            .tokens
            .insert(valid_token2.clone(), Utc::now().timestamp() as Epoch + 7200);

        store.expire_tokens().await.unwrap();

        assert_eq!(store.tokens.len(), 2);
        assert!(store.tokens.contains_key(&valid_token1));
        assert!(store.tokens.contains_key(&valid_token2));
    }
}
