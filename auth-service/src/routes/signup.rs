use std::sync::Arc;

use axum::http::StatusCode;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::domain::user::NewUser;
use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    email::Email,
    error::AuthAPIError,
    password::Password,
};
use crate::services::app_state::{AppServices, AppState};

#[derive(Deserialize, Debug)]
pub struct SignupRequest {
    email: String,
    password: String,
    #[serde(rename = "requires2FA")]
    requires_2fa: bool,
}

#[derive(Serialize)]
pub struct SignupResponse {
    message: String,
}

#[tracing::instrument(name = "Signup", skip_all, err(Debug))]
pub async fn post<S: AppServices>(
    State(state): State<Arc<AppState<S>>>,
    Json(payload): Json<SignupRequest>,
) -> Result<(StatusCode, Json<SignupResponse>), AuthAPIError> {
    let email = Email::parse(payload.email).map_err(AuthAPIError::InvalidEmail)?;
    let password = Password::parse(payload.password)
        .await
        .map_err(AuthAPIError::InvalidPassword)?;

    let user = NewUser::new(email, password, payload.requires_2fa);

    let mut user_store = state.user_store.write().await;
    user_store.add_user(user).await.map_err(|e| match e {
        UserStoreError::UserAlreadyExists => AuthAPIError::UserAlreadyExists,
        _ => AuthAPIError::UnexpectedError,
    })?;

    Ok((
        StatusCode::CREATED,
        Json(SignupResponse {
            message: "User created successfully".to_string(),
        }),
    ))
}
