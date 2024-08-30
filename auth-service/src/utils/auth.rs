use core::fmt;
use std::sync::Arc;

use argon2::{password_hash::SaltString, Algorithm, Argon2, Params, PasswordHasher, Version};
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use color_eyre::eyre::{eyre, Report, Result};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
use log::error;
use secrecy::{ExposeSecret, Secret};
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::{data_stores::BannedTokenStore, email::Email};

use super::constants::{Epoch, Time, JWT_COOKIE_NAME, JWT_SECRET, TOKEN_TTL_SECONDS};

#[derive(Debug, thiserror::Error)]
pub enum GenerateTokenError {
    #[error("Token error")]
    TokenError(#[source] Report),
    #[error("Banned token")]
    BannedToken,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
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

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub sub: Secret<String>,
    pub exp: Epoch,
    pub purpose: TokenPurpose,
}

impl Serialize for Claims {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Claims", 3)?;
        state.serialize_field("sub", self.sub.expose_secret())?;
        state.serialize_field("exp", &self.exp)?;
        state.serialize_field("purpose", &self.purpose)?;
        state.end()
    }
}

#[tracing::instrument(name = "Generate Auth Cookie", skip_all)]
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>, GenerateTokenError> {
    let token = generate_auth_token(email)?;
    let cookie = Cookie::build((JWT_COOKIE_NAME, token.expose_secret().clone()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();
    Ok(cookie)
}

#[tracing::instrument(name = "Generate Auth Token", skip_all)]
pub fn generate_auth_token(email: &Email) -> Result<Secret<String>, GenerateTokenError> {
    let delta = chrono::Duration::try_seconds(TOKEN_TTL_SECONDS).ok_or(GenerateTokenError::UnexpectedError(eyre!(
        "Failed to obtain chrono duration"
    )))?;
    let exp: Epoch = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::UnexpectedError(eyre!(
            "Failed to generate expiration timestamp"
        )))?
        .timestamp()
        .try_into()
        .map_err(|_| GenerateTokenError::UnexpectedError(eyre!("Failed to convert to Epoch")))?;
    let sub = email.as_ref().to_owned();
    let claims = Claims {
        sub,
        exp,
        purpose: TokenPurpose::Auth,
    };
    let token = create_token(&claims).map_err(|e| GenerateTokenError::TokenError(e.into()))?;

    Ok(token)
}

#[tracing::instrument(name = "Validate Token", skip_all)]
pub async fn validate_token_structure(token: &str) -> Result<Claims, GenerateTokenError> {
    let data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.expose_secret().as_bytes()),
        &Validation::default(),
    ) {
        Ok(data) => data,
        Err(error) => {
            error!("[ERROR][validate_token_structure] {:?}", error);
            println!("[ERROR][validate_token_structure] {:?}", error);
            return Err(GenerateTokenError::TokenError(error.into()));
        }
    };
    let claims = data.claims;
    Ok(claims)
}

#[tracing::instrument(name = "Validate Token and Check if Banned", skip_all)]
pub async fn validate_token<T: BannedTokenStore>(
    banned_token_store: Arc<RwLock<T>>,
    token: Secret<String>,
) -> Result<Claims, GenerateTokenError> {
    let banned_token_store = banned_token_store.read().await;
    banned_token_store
        .check_token(token.clone())
        .await
        .map_err(|_| GenerateTokenError::BannedToken)?;
    let claims = validate_token_structure(token.expose_secret()).await?;
    Ok(claims)
}

#[tracing::instrument(name = "Create Token", skip_all)]
pub fn create_token(claims: &Claims) -> Result<Secret<String>, jsonwebtoken::errors::Error> {
    let token = encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.expose_secret().as_bytes()),
    )?;

    Ok(Secret::new(token))
}

#[tracing::instrument(name = "Generate Password Reset Token", skip_all)]
pub fn generate_password_reset_token(email: &Email) -> Result<Secret<String>, GenerateTokenError> {
    let delta = chrono::Duration::try_seconds(Time::Minutes15 as i64).ok_or(GenerateTokenError::UnexpectedError(
        eyre!("Failed to obtain chrono duration"),
    ))?;
    let exp: Epoch = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::UnexpectedError(eyre!(
            "Failed to generate expiration timestamp"
        )))?
        .timestamp()
        .try_into()
        .map_err(|_| GenerateTokenError::UnexpectedError(eyre!("Failed to convert to Epoch")))?;
    let sub = email.as_ref().to_owned();
    let claims = Claims {
        sub,
        exp,
        purpose: TokenPurpose::PasswordReset,
    };
    create_token(&claims).map_err(|e| GenerateTokenError::TokenError(e.into()))
}

#[tracing::instrument(name = "Validate Password Reset Token", skip_all)]
pub async fn validate_password_reset_token<T: BannedTokenStore>(
    banned_token_store: Arc<RwLock<T>>,
    token: Secret<String>,
) -> Result<(Email, Claims), GenerateTokenError> {
    let claims = validate_token(banned_token_store, token).await?;

    if claims.purpose != TokenPurpose::PasswordReset {
        return Err(GenerateTokenError::TokenError(eyre!("Invalid token type")));
    }

    let email = Email::parse(claims.sub.clone()).map_err(|err_msg| GenerateTokenError::TokenError(eyre!(err_msg)))?;

    Ok((email, claims))
}

#[tracing::instrument(name = "Compute Password Hash", skip_all)]
pub async fn async_compute_password_hash(password: Secret<String>) -> Result<Secret<String>> {
    let password_hash = tokio::task::spawn_blocking(|| compute_password_hash(password)).await??;

    Ok(password_hash)
}

pub fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>> {
    let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(15000, 2, 1, None)?);
    let hash = argon2
        .hash_password(password.expose_secret().as_bytes(), &salt)?
        .to_string();

    Ok(Secret::new(hash))
}

//*******************************  TESTS  *******************************//

#[cfg(test)]
mod tests {
    use crate::{
        services::hashmap_banned_token_store::HashMapBannedTokenStore,
        utils::constants::PASSWORD_RESET_TOKEN_TTL_SECONDS,
    };

    use super::*;

    fn str_to_valid_email(email: &str) -> Email {
        Email::parse(Secret::new(email.to_string())).unwrap()
    }

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        let email = str_to_valid_email("test@example.com");
        let cookie = generate_auth_cookie(&email).unwrap();
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        let email = str_to_valid_email("test@example.com");
        let secret = generate_auth_token(&email).unwrap();
        assert_eq!(secret.expose_secret().split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_structure_with_valid_token() {
        let email = str_to_valid_email("test@example.com");
        let token = generate_auth_token(&email).unwrap();
        let claims = validate_token_structure(token.expose_secret()).await.unwrap();
        assert_eq!(claims.sub.expose_secret(), "test@example.com");

        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();

        assert!(claims.exp > exp as Epoch);
    }

    #[tokio::test]
    async fn test_validate_token_structure_with_invalid_token() {
        let token = "invalid_token".to_owned();
        let result = validate_token_structure(&token).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let email = str_to_valid_email("test@example.com");
        let token = generate_auth_token(&email).unwrap();
        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));
        let result = validate_token(banned_token_store, token).await;

        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.sub.expose_secret(), "test@example.com");

        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();

        assert!(claims.exp > exp as Epoch);
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let token = Secret::new("invalid_token".to_owned());
        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));
        let result = validate_token(banned_token_store, token).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(GenerateTokenError::TokenError(_))));
    }

    #[tokio::test]
    async fn test_validate_token_with_banned_token() {
        let email = str_to_valid_email("test@example.com");
        let token = generate_auth_token(&email).unwrap();
        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));

        // Ban the token
        {
            let mut store = banned_token_store.write().await;
            store.add_token(token.clone()).await.unwrap();
        }

        let result = validate_token(banned_token_store, token).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(GenerateTokenError::BannedToken)));
    }

    #[tokio::test]
    async fn test_generate_password_reset_token() {
        let email = str_to_valid_email("test@example.com");
        let result = generate_password_reset_token(&email);

        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.expose_secret().split('.').count(), 3); // JWT format: header.payload.signature
    }

    #[tokio::test]
    async fn test_generate_password_reset_token_expiration() {
        let email = str_to_valid_email("test@example.com");
        let token = generate_password_reset_token(&email).unwrap();

        let claims = decode::<Claims>(
            token.expose_secret(),
            &DecodingKey::from_secret(JWT_SECRET.expose_secret().as_bytes()),
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
        let email = str_to_valid_email("test@example.com");
        let token = generate_password_reset_token(&email).unwrap();
        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));
        let result = validate_token(banned_token_store, token).await;

        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.sub.expose_secret(), email.as_ref().expose_secret());
        assert_eq!(claims.purpose, TokenPurpose::PasswordReset);
    }

    #[tokio::test]
    async fn test_validate_password_reset_token_invalid_purpose() {
        let email = str_to_valid_email("test@example.com");
        let exp = (Utc::now().timestamp() + 3600) as Epoch;
        let claims = Claims {
            sub: email.as_ref().to_owned(),
            exp,
            purpose: TokenPurpose::Auth,
        };
        let token = create_token(&claims).unwrap();

        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));
        let result = validate_password_reset_token(banned_token_store, token).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Token error");
    }

    #[tokio::test]
    async fn test_validate_password_reset_token_expired() {
        let email = str_to_valid_email("test@example.com");
        let exp = (Utc::now().timestamp() - 3600) as Epoch; // 1 hour in the past
        let claims = Claims {
            sub: email.as_ref().to_owned(),
            exp,
            purpose: TokenPurpose::PasswordReset,
        };
        let token = create_token(&claims).unwrap();
        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));
        let result = validate_password_reset_token(banned_token_store, token).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Token error");
    }

    #[tokio::test]
    async fn test_validate_password_reset_token_invalid_email() {
        let claims = Claims {
            sub: Secret::new("invalid_email".to_string()),
            exp: (Utc::now().timestamp() + 3600) as Epoch,
            purpose: TokenPurpose::PasswordReset,
        };
        let token = create_token(&claims).unwrap();
        let banned_token_store = Arc::new(RwLock::new(HashMapBannedTokenStore::new()));
        let result = validate_password_reset_token(banned_token_store, token).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Token error");
    }
}
