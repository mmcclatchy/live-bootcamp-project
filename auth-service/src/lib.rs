use std::{error::Error, sync::Arc};

use axum::{routing::post, serve::Serve, Router};
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
    pub async fn build(app_state: Arc<AppState>, address: &str) -> Result<Self, Box<dyn Error>> {
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
