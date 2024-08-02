use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::domain::data_stores::UserStoreError;
use crate::domain::{
    data_stores::UserStore, email::Email, error::AuthAPIError, password::Password, user::User,
};
use crate::services::app_state::AppState;

#[derive(Deserialize)]
pub struct SignupRequest {
    email: String,
    password: String,
    requires_2fa: bool,
}

#[derive(Serialize)]
pub struct SignupResponse {
    message: String,
}

pub async fn post<T: UserStore>(
    State(state): State<Arc<AppState<T>>>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<SignupResponse>, AuthAPIError> {
    let email = Email::parse(payload.email).map_err(AuthAPIError::InvalidEmail)?;
    let password = Password::parse(payload.password).map_err(AuthAPIError::InvalidPassword)?;

    let user = User::new(email, password, payload.requires_2fa);

    let mut user_store = state.user_store.write().await;
    user_store.add_user(user).await.map_err(|e| match e {
        UserStoreError::UserAlreadyExists => AuthAPIError::UserAlreadyExists,
        _ => AuthAPIError::UnexpectedError,
    })?;

    Ok(Json(SignupResponse {
        message: "User created successfully".to_string(),
    }))
}
