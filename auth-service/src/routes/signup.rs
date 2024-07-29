use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        data_stores::{UserStore, UserStoreError},
        error::AuthAPIError,
        user::User,
    },
    services::app_state::AppState,
};

pub async fn post<T: UserStore>(
    State(state): State<Arc<AppState<T>>>,
    Json(request): Json<SignupRequest>,
) -> impl IntoResponse {
    let email = request.email;
    let password = request.password;
    if email.is_empty() || !email.contains('@') || password.len() < 8 {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let user = User {
        email,
        password,
        requires_2fa: request.requires_2fa,
    };
    let mut user_store = state.user_store.write().await;
    user_store.add_user(user).await.map_err(|e| match e {
        UserStoreError::UserAlreadyExists => AuthAPIError::UserAlreadyExists,
        _ => AuthAPIError::UnexpectedError,
    })?;

    let response = SignupResponse {
        message: "User created successfully".to_string(),
    };
    Ok((StatusCode::CREATED, Json(response)))
}

#[derive(Deserialize, Debug)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Serialize)]
pub struct SignupResponse {
    pub message: String,
}
