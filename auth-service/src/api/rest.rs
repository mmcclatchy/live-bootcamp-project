use std::error::Error;
use std::io::Write;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::{from_fn, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    serve::Serve,
    Json, Router,
};
use hyper::Method;
use log::info;
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};

use crate::domain::error::AuthAPIError;
use crate::routes;
use crate::services::app_state::{AppServices, AppState};

async fn log_request(req: Request<Body>, next: Next) -> impl IntoResponse {
    info!("Received request: {} {}", req.method(), req.uri());
    next.run(req).await
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

pub struct RESTApp {
    pub address: String,
    pub server: Serve<Router, Router>,
}

impl RESTApp {
    pub async fn new<S: AppServices + 'static>(
        app_state: Arc<AppState<S>>,
        address: String,
    ) -> Result<Self, Box<dyn Error>> {
        let allowed_origins = [
            "http://localhost".parse()?,
            "https://rust-bc.markmcclatchy.com/app".parse()?,
        ];

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST])
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .layer(TraceLayer::new_for_http())
            .layer(from_fn(log_request))
            .nest_service("/", ServeDir::new("assets"))
            .route("/health", post(health_check))
            .route("/signup", post(routes::signup::post))
            .route("/login", post(routes::login::post))
            .route("/logout", post(routes::logout::post))
            .route("/verify-2fa", post(routes::verify_2fa::post))
            .route("/verify-token", post(routes::verify_token::post))
            .route("/initiate-password-reset", post(routes::initiate_password_reset::post))
            .route("/reset-password", post(routes::reset_password::post))
            .route("/reset-password", get(routes::reset_password::get))
            .with_state(app_state)
            .layer(cors);

        let listener = tokio::net::TcpListener::bind(&address).await?;
        let address = listener.local_addr()?.to_string();
        Ok(Self {
            address,
            server: axum::serve(listener, router),
        })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        eprintln!("About to start REST server on {}", self.address);
        std::io::stderr().flush().unwrap();

        info!("REST server listening on {}", self.address);

        eprintln!("Listener bound successfully");
        std::io::stderr().flush().unwrap();

        self.server.await
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists".to_string()),
            AuthAPIError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()),
            AuthAPIError::InvalidEmail(msg) => (StatusCode::BAD_REQUEST, msg),
            AuthAPIError::InvalidPassword(msg) => (StatusCode::BAD_REQUEST, msg),
            AuthAPIError::UserNotFound => (StatusCode::NOT_FOUND, "User not found".to_string()),
            AuthAPIError::UnexpectedError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred".to_string(),
            ),
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Missing auth token".to_string()),
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid auth token".to_string()),
            AuthAPIError::InvalidLoginAttemptId => (StatusCode::BAD_REQUEST, "Invalid auth id".to_string()),
            AuthAPIError::InvalidTwoFactorAuthCode => (StatusCode::BAD_REQUEST, "Invalid auth code".to_string()),
        };

        let body = Json(ErrorResponse { error: error_message });

        (status, body).into_response()
    }
}
