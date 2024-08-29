use std::{fmt, sync::Arc};

use axum::response::{Html, IntoResponse};
use axum::{extract::State, Json};
use axum_extra::extract::CookieJar;
use hyper::StatusCode;
use secrecy::Secret;
use serde::{Deserialize, Serialize};

use crate::services::app_state::{AppServices, AppState};
use crate::utils::auth::validate_password_reset_token;
use crate::{
    domain::{
        data_stores::{PasswordResetTokenStore, UserStore},
        error::AuthAPIError,
        password::Password,
    },
    utils::auth::{generate_auth_cookie, TokenPurpose},
};

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    token: Secret<String>,
    new_password: Secret<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ResetPasswordResponse {
    pub message: String,
}

impl fmt::Display for ResetPasswordResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[tracing::instrument(name = "Reset Password GET Request")]
pub async fn get() -> impl IntoResponse {
    Html(include_str!("../../assets/index.html"))
}

#[tracing::instrument(name = "Reset Password POST Request")]
pub async fn post<S: AppServices>(
    State(state): State<Arc<AppState<S>>>,
    jar: CookieJar,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<(CookieJar, (StatusCode, Json<ResetPasswordResponse>)), AuthAPIError> {
    let new_password = Password::parse(payload.new_password)
        .await
        .map_err(AuthAPIError::InvalidPassword)?;
    let (email, claims) = validate_password_reset_token(state.banned_token_store.clone(), payload.token)
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    if claims.purpose != TokenPurpose::PasswordReset {
        return Err(AuthAPIError::InvalidToken);
    }

    let mut token_store = state.password_reset_token_store.write().await;
    token_store
        .remove_token(&email)
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;

    let mut user_store = state.user_store.write().await;
    user_store
        .update_password(&email, new_password)
        .await
        .map_err(|_| AuthAPIError::UserNotFound)?;

    let auth_cookie = generate_auth_cookie(&email).map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;
    let updated_jar = jar.add(auth_cookie);

    let response = ResetPasswordResponse {
        message: "Password has been reset successfully.".to_string(),
    };
    Ok((updated_jar, (StatusCode::OK, Json(response))))
}
