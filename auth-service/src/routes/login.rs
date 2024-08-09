use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::State, Json};
use log::info;
use serde::Deserialize;

use crate::domain::{
    data_stores::UserStore, email::Email, error::AuthAPIError, password::Password,
};
use crate::services::app_state::AppState;

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn post<T: UserStore>(
    State(state): State<Arc<AppState<T>>>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    info!("[REST][POST][/signup] Received request: {:?}", payload);

    let email = Email::parse(payload.email).map_err(AuthAPIError::InvalidEmail)?;
    let password = Password::parse(payload.password).map_err(AuthAPIError::InvalidPassword)?;

    let user_store = state.user_store.write().await;
    if user_store.validate_user(&email, &password).await.is_err() {
        return Err(AuthAPIError::InvalidCredentials);
    }

    Ok(StatusCode::OK.into_response())
}
