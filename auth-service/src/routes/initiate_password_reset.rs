use axum::response::IntoResponse;
use hyper::StatusCode;

pub async fn post() -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED.into_response()
}
