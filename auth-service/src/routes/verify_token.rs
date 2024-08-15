use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::domain::{
    data_stores::{BannedTokenStore, TwoFACodeStore, UserStore},
    email_client::EmailClient,
    error::AuthAPIError,
};
use crate::services::app_state::AppState;
use crate::utils::auth::validate_token;

#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    token: String,
}

pub async fn post<T: BannedTokenStore, U: UserStore, V: TwoFACodeStore, W: EmailClient>(
    State(state): State<Arc<AppState<T, U, V, W>>>,
    Json(request): Json<VerifyTokenRequest>,
) -> impl IntoResponse {
    match validate_token(state.banned_token_store.clone(), &request.token).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(AuthAPIError::InvalidCredentials),
    }
}
