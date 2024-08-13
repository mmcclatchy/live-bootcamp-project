use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::State, Json};
use axum_extra::extract::CookieJar;
use log::info;
use serde::{Deserialize, Serialize};

use crate::domain::data_stores::BannedTokenStore;
use crate::domain::{
    data_stores::UserStore, email::Email, error::AuthAPIError, password::Password,
};
use crate::services::app_state::AppState;
use crate::utils::auth::generate_auth_cookie;

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

pub async fn post<T: BannedTokenStore, U: UserStore>(
    State(state): State<Arc<AppState<T, U>>>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    info!("[REST][POST][/signup] Received request: {:?}", payload);

    let email = Email::parse(payload.email).map_err(AuthAPIError::InvalidEmail)?;
    let password = Password::parse(payload.password).map_err(AuthAPIError::InvalidPassword)?;
    let user_store = state.user_store.write().await;

    let user = user_store
        .validate_user(&email, &password)
        .await
        .map_err(|_| AuthAPIError::InvalidCredentials)?;

    match user.requires_2fa {
        false => handle_no_2fa(&email, jar).await,
        true => handle_2fa(jar).await,
    }
}

async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    let auth_cookie = generate_auth_cookie(email).map_err(|_| AuthAPIError::UnexpectedError)?;
    let updated_jar = jar.add(auth_cookie);
    Ok((
        updated_jar,
        (StatusCode::OK, Json(LoginResponse::RegularAuth)),
    ))
}

async fn handle_2fa(
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    let response = TwoFactorAuthResponse {
        message: "2FA required".to_string(),
        login_attempt_id: "123456".to_string(),
    };
    Ok((
        jar,
        (
            StatusCode::PARTIAL_CONTENT,
            Json(LoginResponse::TwoFactorAuth(response)),
        ),
    ))
}
