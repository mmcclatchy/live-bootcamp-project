use core::fmt;
use std::sync::Arc;

use axum::{extract::State, Json};
use color_eyre::eyre::eyre;
use lazy_static::lazy_static;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

use crate::domain::{
    data_stores::{PasswordResetTokenStore, UserStore},
    email::Email,
    email_client::EmailClient,
    error::AuthAPIError,
};
use crate::services::app_state::{AppServices, AppState};
use crate::services::postmark_email_client::PostmarkTemplate;
use crate::utils::auth::generate_password_reset_token;
use crate::utils::constants::Time;

lazy_static! {
    static ref INITIATE_PASSWORD_RESPONSE: InitiatePasswordResetResponse = InitiatePasswordResetResponse {
        message: "If the email exists, a password reset link has been sent.".to_string(),
    };
}

#[derive(Debug, Deserialize)]
pub struct InitiatePasswordResetRequest {
    email: Secret<String>,
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
        .add_token(email.clone(), token.expose_secret().to_string())
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;

    let template_model = PostmarkTemplate::PasswordReset(Time::Minutes15, token.expose_secret().to_string());
    state
        .email_client
        .send_email(&email, "Password Reset Link", template_model)
        .await
        .map_err(|err_msg| AuthAPIError::UnexpectedError(eyre!(err_msg)))?;

    Ok(Json(INITIATE_PASSWORD_RESPONSE.clone()))
}
