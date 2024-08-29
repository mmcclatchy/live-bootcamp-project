use std::{sync::Arc, time::Duration};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use secrecy::Secret;
use serde::Deserialize;
use tokio::time::timeout;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore},
    email::Email,
    error::AuthAPIError,
};
use crate::services::app_state::{AppServices, AppState};
use crate::utils::auth::generate_auth_cookie;

#[derive(Debug, Deserialize)]
pub struct Verify2FARequest {
    email: Secret<String>,
    #[serde(rename = "loginAttemptId")]
    login_attempt_id: Secret<String>,
    #[serde(rename = "2FACode")]
    two_factor_code: Secret<String>,
}

#[tracing::instrument(name = "Verify 2FA POST Request")]
pub async fn post<S: AppServices>(
    State(state): State<Arc<AppState<S>>>,
    jar: CookieJar,
    Json(payload): Json<Verify2FARequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    println!("[verify-2fa][post] Invoked");
    println!("[verify-2fa][post] {:?}", payload);

    let email = Email::parse(payload.email).map_err(AuthAPIError::InvalidEmail)?;
    let login_attempt_id =
        LoginAttemptId::parse(payload.login_attempt_id).map_err(|_| AuthAPIError::InvalidLoginAttemptId)?;
    let two_factor_code =
        TwoFACode::parse(payload.two_factor_code).map_err(|_| AuthAPIError::InvalidTwoFactorAuthCode)?;

    println!("[verify-2fa][post] payload successfully parsed");

    // Use a timeout when acquiring the lock to prevent indefinite waiting
    let mut two_fa_code_store = match timeout(Duration::from_secs(5), state.two_fa_code_store.write()).await {
        Ok(guard) => guard,
        Err(e) => return Err(AuthAPIError::UnexpectedError(e.into())),
    };

    let (stored_attempt_id, stored_2fa_code) = match two_fa_code_store.get_code(&email).await {
        Ok(result) => result,
        Err(_) => return Err(AuthAPIError::InvalidCredentials),
    };
    println!("[verify-2fa][post] Two factor code retrieved from store");

    if login_attempt_id != stored_attempt_id || two_factor_code != stored_2fa_code {
        println!("[verify-2fa][post] Incorrect login_attempt_id and two_factor_code");
        return Err(AuthAPIError::InvalidCredentials);
    }

    two_fa_code_store
        .remove_code(&email)
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;
    println!("[verify-2fa][post] Two factor auth code successfully removed from store");

    drop(two_fa_code_store);

    let auth_cookie = generate_auth_cookie(&email).map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;
    let updated_jar = jar.add(auth_cookie);
    println!("[verify-2fa][post] Auth cookie successfully created");

    Ok((updated_jar, StatusCode::OK.into_response()))
}
