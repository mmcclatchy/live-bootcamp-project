use std::{error::Error, sync::Arc};

use axum::{http::StatusCode, response::IntoResponse, routing::post, serve::Serve, Json, Router};
use domain::{data_stores::UserStore, error::AuthAPIError};
use serde::{Deserialize, Serialize};
use services::app_state::AppState;
use tower_http::services::ServeDir;

pub mod domain;
pub mod routes;
pub mod services;

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build<T: UserStore>(
        app_state: Arc<AppState<T>>,
        address: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(routes::signup::post))
            .route("/login", post(routes::login::post))
            .route("/logout", post(routes::logout::post))
            .route("/verify-2fa", post(routes::verify_2fa::post))
            .route("/verify-token", post(routes::verify_token::post))
            .with_state(app_state);
        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::UnexpectedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected Error")
            }
        };
        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });
        (status, body).into_response()
    }
}
