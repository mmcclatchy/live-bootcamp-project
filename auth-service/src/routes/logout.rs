use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{cookie, CookieJar};

use crate::domain::{
    data_stores::{BannedTokenStore, UserStore},
    error::AuthAPIError,
};
use crate::services::app_state::AppState;
use crate::utils::{auth::validate_token, constants::JWT_COOKIE_NAME};

pub async fn post<T: BannedTokenStore, U: UserStore>(
    State(state): State<Arc<AppState<T, U>>>,
    jar: CookieJar,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(cookie) => cookie,
        None => return Err(AuthAPIError::MissingToken),
    };

    let token = cookie.value().to_owned();
    validate_token(&token)
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    let jar = jar.remove(cookie::Cookie::from(JWT_COOKIE_NAME));

    let mut banned_token_store = state.banned_token_store.write().await;
    banned_token_store
        .add_token(token)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    Ok((jar, StatusCode::OK))
}
