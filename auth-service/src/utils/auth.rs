use core::fmt;
use std::{error::Error, sync::Arc};

use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::{data_stores::BannedTokenStore, email::Email};

use super::constants::{Epoch, JWT_COOKIE_NAME, JWT_SECRET, PASSWORD_RESET_TOKEN_TTL_SECONDS, TOKEN_TTL_SECONDS};

#[derive(Debug, PartialEq)]
pub enum GenerateTokenError {
    TokenError(jsonwebtoken::errors::Error),
    BannedToken,
    UnexpectedError,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum TokenPurpose {
    Auth,
    PasswordReset,
}

impl fmt::Display for TokenPurpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenPurpose::Auth => write!(f, "auth"),
            TokenPurpose::PasswordReset => write!(f, "password reset"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: Epoch,
    pub purpose: TokenPurpose,
}

pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>, GenerateTokenError> {
    let token = generate_auth_token(email)?;
    Ok(create_auth_cookie(token))
}

pub fn create_auth_cookie(token: String) -> Cookie<'static> {
    let cookie = Cookie::build((JWT_COOKIE_NAME, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();
    cookie
}

pub fn generate_auth_token(email: &Email) -> Result<String, GenerateTokenError> {
    let delta = chrono::Duration::try_seconds(TOKEN_TTL_SECONDS).ok_or(GenerateTokenError::UnexpectedError)?;
    let exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::UnexpectedError)?
        .timestamp();
    let exp: Epoch = exp.try_into().map_err(|_| GenerateTokenError::UnexpectedError)?;
    let sub = email.as_ref().to_owned();
    let claims = Claims {
        sub,
        exp,
        purpose: TokenPurpose::Auth,
    };
    create_token(&claims).map_err(GenerateTokenError::TokenError)
}

pub async fn validate_token_structure(token: &str) -> Result<Claims, GenerateTokenError> {
    let data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(error) => {
            error!("[ERROR][validate_token_structure] {:?}", error);
            println!("[ERROR][validate_token_structure] {:?}", error);
            return Err(GenerateTokenError::TokenError(error));
        }
    };
    let claims = data.claims;
    Ok(claims)
}

pub async fn validate_token<T: BannedTokenStore>(
    banned_token_store: Arc<RwLock<T>>,
    token: &str,
) -> Result<Claims, GenerateTokenError> {
    let banned_token_store = banned_token_store.read().await;
    banned_token_store
        .check_token(token.to_string())
        .await
        .map_err(|_| GenerateTokenError::BannedToken)?;
    let claims = validate_token_structure(token).await?;
    Ok(claims)
}

pub fn create_token(claims: &Claims) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
}

pub fn generate_password_reset_token(email: &Email) -> Result<String, GenerateTokenError> {
    let delta =
        chrono::Duration::try_seconds(PASSWORD_RESET_TOKEN_TTL_SECONDS).ok_or(GenerateTokenError::UnexpectedError)?;
    let exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::UnexpectedError)?
        .timestamp();
    let exp: Epoch = exp.try_into().map_err(|_| GenerateTokenError::UnexpectedError)?;
    let sub = email.as_ref().to_owned();
    let claims = Claims {
        sub,
        exp,
        purpose: TokenPurpose::PasswordReset,
    };
    create_token(&claims).map_err(GenerateTokenError::TokenError)
}

pub async fn validate_password_reset_token<T: BannedTokenStore>(
    banned_token_store: Arc<RwLock<T>>,
    token: &str,
) -> Result<(Email, Claims), GenerateTokenError> {
    let claims = validate_token(banned_token_store, token).await?;

    if claims.purpose != TokenPurpose::PasswordReset {
        return Err(GenerateTokenError::TokenError(
            jsonwebtoken::errors::ErrorKind::InvalidToken.into(),
        ));
    }

    let email = Email::parse(claims.sub.clone())
        .map_err(|_| GenerateTokenError::TokenError(jsonwebtoken::errors::ErrorKind::InvalidSubject.into()))?;

    Ok((email, claims))
}

pub async fn async_compute_password_hash(password: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let password = password.to_string();
    let password_hash = tokio::task::spawn_blocking(|| compute_password_hash(password)).await??;

    Ok(password_hash)
}

pub fn compute_password_hash(password: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(15000, 2, 1, None)?);
    let hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();

    Ok(hash)
}

//*******************************  TESTS  *******************************//

#[cfg(test)]
mod tests {
    use crate::services::hashmap_banned_token_store::HashMapBannedTokenStore;

    use super::*;

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        let email = Email::parse("test@example.com".to_owned()).unwrap();
        let cookie = generate_auth_cookie(&email).unwrap();
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        let token = "test_token".to_owned();
        let cookie = create_auth_cookie(token.clone());
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        let email = Email::parse("test@example.com".to_owned()).unwrap();
        let result = generate_auth_token(&email).unwrap();
        assert_eq!(result.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_structure_with_valid_token() {
        let email = Email::parse("test@example.com".to_owned()).unwrap();
        let token = generate_auth_token(&email).unwrap();
        let result = validate_token_structure(&token).await.unwrap();
        assert_eq!(result.sub, "test@example.com");

        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();

        assert!(result.exp > exp as Epoch);
    }

    #[tokio::test]
    async fn test_validate_token_structure_with_invalid_token() {
        let token = "invalid_token".to_owned();
        let result = validate_token_structure(&token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let email = Email::parse("test@example.com".to_owned()).unwrap();
        let token = generate_auth_token(&email).unwrap();
        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));
        let result = validate_token(banned_token_store, &token).await;

        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.sub, "test@example.com");

        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();

        assert!(claims.exp > exp as Epoch);
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let token = "invalid_token".to_owned();
        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));
        let result = validate_token(banned_token_store, &token).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(GenerateTokenError::TokenError(_))));
    }

    #[tokio::test]
    async fn test_validate_token_with_banned_token() {
        let email = Email::parse("test@example.com".to_owned()).unwrap();
        let token = generate_auth_token(&email).unwrap();
        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));

        // Ban the token
        {
            let mut store = banned_token_store.write().await;
            store.add_token(token.clone()).await.unwrap();
        }

        let result = validate_token(banned_token_store, &token).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(GenerateTokenError::BannedToken)));
    }

    #[tokio::test]
    async fn test_generate_password_reset_token() {
        let email = Email::parse("test@example.com".to_string()).unwrap();
        let result = generate_password_reset_token(&email);

        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.split('.').count(), 3); // JWT format: header.payload.signature
    }

    #[tokio::test]
    async fn test_generate_password_reset_token_expiration() {
        let email = Email::parse("test@example.com".to_string()).unwrap();
        let token = generate_password_reset_token(&email).unwrap();

        let claims = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
            &Validation::default(),
        )
        .unwrap()
        .claims;

        let now = Utc::now().timestamp() as Epoch;
        assert!(claims.exp > now);
        assert!(claims.exp <= now + PASSWORD_RESET_TOKEN_TTL_SECONDS as Epoch);
    }

    #[tokio::test]
    async fn test_validate_password_reset_token_valid() {
        let email = Email::parse("test@example.com".to_string()).unwrap();
        let token = generate_password_reset_token(&email).unwrap();
        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));
        let result = validate_token(banned_token_store, &token).await;

        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.sub, email.as_ref());
        assert_eq!(claims.purpose, TokenPurpose::PasswordReset);
    }

    #[tokio::test]
    async fn test_validate_password_reset_token_invalid_purpose() {
        let email = Email::parse("test@example.com".to_string()).unwrap();
        let exp = (Utc::now().timestamp() + 3600) as Epoch;
        let claims = Claims {
            sub: email.as_ref().to_string(),
            exp,
            purpose: TokenPurpose::Auth,
        };
        let token = create_token(&claims).unwrap();

        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));
        let result = validate_password_reset_token(banned_token_store, &token).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GenerateTokenError::TokenError(jsonwebtoken::errors::ErrorKind::InvalidToken.into())
        );
    }

    #[tokio::test]
    async fn test_validate_password_reset_token_expired() {
        let email = Email::parse("test@example.com".to_string()).unwrap();
        let exp = (Utc::now().timestamp() - 3600) as Epoch; // 1 hour in the past
        let claims = Claims {
            sub: email.as_ref().to_string(),
            exp,
            purpose: TokenPurpose::PasswordReset,
        };
        let token = create_token(&claims).unwrap();
        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));
        let result = validate_password_reset_token(banned_token_store, &token).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GenerateTokenError::TokenError(jsonwebtoken::errors::ErrorKind::ExpiredSignature.into())
        );
    }

    #[tokio::test]
    async fn test_validate_password_reset_token_invalid_email() {
        let claims = Claims {
            sub: "invalid_email".to_string(),
            exp: (Utc::now().timestamp() + 3600) as Epoch,
            purpose: TokenPurpose::PasswordReset,
        };
        let token = create_token(&claims).unwrap();
        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));
        let result = validate_password_reset_token(banned_token_store, &token).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GenerateTokenError::TokenError(jsonwebtoken::errors::ErrorKind::InvalidSubject.into())
        );
    }
}
