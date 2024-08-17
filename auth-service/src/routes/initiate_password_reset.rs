use std::sync::Arc;

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

const INITIATE_PASSWORD_RESPONSE_MESSAGE: &str =
    "If the email exists, a password reset link has been sent.";
const INITIATE_PASSWORD_RESPONSE: Result<Json<InitiatePasswordResetResponse>, AuthAPIError> =
    Ok(Json(InitiatePasswordResetResponse {
        message: INITIATE_PASSWORD_RESPONSE_MESSAGE,
    }));

#[derive(Deserialize)]
pub struct InitiatePasswordResetRequest {
    email: String,
}

#[derive(Serialize)]
pub struct InitiatePasswordResetResponse {
    message: &'static str,
}

pub async fn post<S: AppServices>(
    State(state): State<Arc<AppState<S>>>,
    Json(payload): Json<InitiatePasswordResetRequest>,
) -> Result<Json<InitiatePasswordResetResponse>, AuthAPIError> {
    let email = Email::parse(payload.email).map_err(AuthAPIError::InvalidEmail)?;

    let user_store = state.user_store.read().await;
    let token = match user_store.get_user(&email).await {
        Err(_) => return INITIATE_PASSWORD_RESPONSE,
        Ok(_) => match generate_password_reset_token(&email) {
            Err(_) => return INITIATE_PASSWORD_RESPONSE,
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
        .send_email(&email, "Password Reset", &token)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    INITIATE_PASSWORD_RESPONSE
}
