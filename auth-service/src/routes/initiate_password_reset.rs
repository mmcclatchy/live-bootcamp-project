use core::fmt;
use std::sync::{Arc, LazyLock};

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::domain::{
    data_stores::{PasswordResetTokenStore, UserStore},
    email::Email,
    email_client::EmailClient,
    error::AuthAPIError,
};
use crate::services::app_state::{AppServices, AppState};
use crate::utils::auth::generate_password_reset_token;

static INITIATE_PASSWORD_RESPONSE: LazyLock<InitiatePasswordResetResponse> =
    LazyLock::new(|| InitiatePasswordResetResponse {
        message: "If the email exists, a password reset link has been sent.".to_string(),
    });

#[derive(Debug, Deserialize)]
pub struct InitiatePasswordResetRequest {
    email: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct InitiatePasswordResetResponse {
    pub message: String,
}

impl fmt::Display for InitiatePasswordResetResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub async fn post<'a, S: AppServices>(
    State(state): State<Arc<AppState<S>>>,
    Json(payload): Json<InitiatePasswordResetRequest>,
) -> Result<Json<InitiatePasswordResetResponse>, AuthAPIError> {
    let email = Email::parse(payload.email).map_err(AuthAPIError::InvalidEmail)?;

    let user_store = state.user_store.read().await;
    let token = match user_store.get_user(&email).await {
        Err(_) => return Ok(Json(INITIATE_PASSWORD_RESPONSE.clone())),
        Ok(_) => match generate_password_reset_token(&email) {
            Err(_) => return Ok(Json(INITIATE_PASSWORD_RESPONSE.clone())),
            Ok(token) => token,
        },
    };
    let mut token_store = state.password_reset_token_store.write().await;
    token_store
        .add_token(email.clone(), token.clone())
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    state
        .email_client
        .send_email(&email, "Password Reset Link", &token)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    Ok(Json(INITIATE_PASSWORD_RESPONSE.clone()))
}
