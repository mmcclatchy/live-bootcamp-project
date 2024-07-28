use std::error::Error;

use axum::{routing::post, serve::Serve, Router};
use tower_http::services::ServeDir;

pub mod domain;
pub mod routes;

pub struct Application {
    server: Serve<Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(address: &str) -> Result<Self, Box<dyn Error>> {
        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .nest_service("/signup", post(routes::signup::post))
            .nest_service("/login", post(routes::login::post))
            .nest_service("/logout", post(routes::logout::post))
            .nest_service("/verify-2fa", post(routes::verify_2fa::post))
            .nest_service("/verify-token", post(routes::verify_token::post));
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
