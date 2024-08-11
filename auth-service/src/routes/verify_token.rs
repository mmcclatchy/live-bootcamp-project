use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    _token: String,
}

pub async fn post(Json(_token): Json<VerifyTokenRequest>) -> impl IntoResponse {
    StatusCode::OK.into_response()
}
