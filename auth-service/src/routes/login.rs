use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::State, Json};
use axum_extra::extract::CookieJar;
use log::info;
use serde::Deserialize;

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

pub async fn post<T: UserStore>(
    State(state): State<Arc<AppState<T>>>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    info!("[REST][POST][/signup] Received request: {:?}", payload);

    let email = match Email::parse(payload.email) {
        Ok(email) => email,
        Err(msg) => return (jar, Err(AuthAPIError::InvalidEmail(msg))),
    };

    let password = match Password::parse(payload.password) {
        Ok(password) => password,
        Err(msg) => return (jar, Err(AuthAPIError::InvalidPassword(msg))),
    };

    let user_store = state.user_store.write().await;
    if user_store.validate_user(&email, &password).await.is_err() {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    };

    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    };

    let updated_jar = jar.add(auth_cookie);
    (updated_jar, Ok(StatusCode::OK.into_response()))
}
