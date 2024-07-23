use axum::{http::StatusCode, response::IntoResponse};

pub async fn post() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
