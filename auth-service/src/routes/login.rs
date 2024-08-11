use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::State, Json};
use axum_extra::extract::CookieJar;
use log::info;
use serde::Deserialize;

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

pub async fn post<T: BannedTokenStore, U: UserStore>(
    State(state): State<Arc<AppState<T, U>>>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    info!("[REST][POST][/signup] Received request: {:?}", payload);

    let email = Email::parse(payload.email).map_err(AuthAPIError::InvalidEmail)?;
    let password = Password::parse(payload.password).map_err(AuthAPIError::InvalidPassword)?;
    let user_store = state.user_store.write().await;

    if user_store.validate_user(&email, &password).await.is_err() {
        return Err(AuthAPIError::InvalidCredentials);
    };

    let auth_cookie = generate_auth_cookie(&email).map_err(|_| AuthAPIError::UnexpectedError)?;
    let updated_jar = jar.add(auth_cookie);

    Ok((updated_jar, StatusCode::OK.into_response()))
}
