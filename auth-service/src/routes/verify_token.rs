use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use secrecy::Secret;
use serde::Deserialize;

use crate::domain::error::AuthAPIError;
use crate::services::app_state::{AppServices, AppState};
use crate::utils::auth::validate_token;

#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    token: Secret<String>,
}

#[tracing::instrument(name = "Verify Auth Token POST Request")]
pub async fn post<S: AppServices>(
    State(state): State<Arc<AppState<S>>>,
    Json(request): Json<VerifyTokenRequest>,
) -> impl IntoResponse {
    match validate_token(state.banned_token_store.clone(), request.token).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(AuthAPIError::InvalidCredentials),
    }
}
