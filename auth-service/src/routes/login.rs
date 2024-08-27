use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::State, Json};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use log::info;
use secrecy::Secret;
use serde::{Deserialize, Serialize};

use crate::domain::data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore};
use crate::domain::email_client::EmailClient;
use crate::domain::{data_stores::UserStore, email::Email, error::AuthAPIError, password::Password};
use crate::services::app_state::{AppServices, AppState};
use crate::utils::auth::generate_auth_cookie;

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub email: Secret<String>,
    pub password: Secret<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

pub async fn post<S: AppServices>(
    State(state): State<Arc<AppState<S>>>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    info!("[REST][POST][/signup] Received request: {:?}", payload);

    let email = Email::parse(payload.email).map_err(AuthAPIError::InvalidEmail)?;
    let password = Password::parse(payload.password)
        .await
        .map_err(AuthAPIError::InvalidPassword)?;
    let user_store = state.user_store.write().await;

    let user = user_store
        .validate_user(&email, &password)
        .await
        .map_err(|_| AuthAPIError::InvalidCredentials)?;

    match user.requires_2fa {
        false => handle_no_2fa(&email, jar).await,
        true => handle_2fa(&email, &state, jar).await,
    }
}

#[tracing::instrument(name = "Handle no 2fa path")]
async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    let auth_cookie = generate_auth_cookie(email).map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;
    let updated_jar = jar.add(auth_cookie);
    Ok((updated_jar, (StatusCode::OK, Json(LoginResponse::RegularAuth))))
}

#[tracing::instrument(name = "Handle 2fa path")]
async fn handle_2fa<S: AppServices>(
    email: &Email,
    state: &AppState<S>,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    let email = (*email).clone();
    let mut two_fa_code_store = state.two_fa_code_store.write().await;
    two_fa_code_store
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;

    let response = TwoFactorAuthResponse {
        message: "2FA required".to_string(),
        login_attempt_id: login_attempt_id.to_string(),
    };

    if state
        .email_client
        .send_email(&email, "Rust Live Boot-camp Authentication Code", two_fa_code.as_ref())
        .await
        .is_err()
    {
        tracing::info!("Error sending 2FA email");
        return Err(AuthAPIError::UnexpectedError(eyre!("Failed to send 2FA email")));
    };

    Ok((
        jar,
        (
            StatusCode::PARTIAL_CONTENT,
            Json(LoginResponse::TwoFactorAuth(response)),
        ),
    ))
}
