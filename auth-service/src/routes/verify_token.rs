use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{domain::error::AuthAPIError, utils::auth::validate_token};

#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    token: String,
}

pub async fn post(Json(request): Json<VerifyTokenRequest>) -> impl IntoResponse {
    if validate_token(&request.token).await.is_err() {
        return Err(AuthAPIError::InvalidCredentials);
    }
    Ok(StatusCode::OK)
}
