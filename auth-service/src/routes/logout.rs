use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    domain::error::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn post(jar: CookieJar) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return Err(AuthAPIError::MissingToken),
    };
    let token = cookie.value().to_owned();
    validate_token(&token)
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;
    Ok((jar, StatusCode::OK))
}
