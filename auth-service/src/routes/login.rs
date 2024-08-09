use axum::{http::StatusCode, response::IntoResponse};

pub async fn post() -> impl IntoResponse {
    StatusCode::OK.into_response()
}

pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub struct LoginResponse {
    pub message: String,
}
