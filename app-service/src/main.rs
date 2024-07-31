use std::env;

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use tower_http::services::ServeDir;

use app_proto::{auth_service_client::AuthServiceClient, VerifyTokenRequest};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/", get(root))
        .route("/protected", get(protected));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    login_link: String,
    logout_link: String,
}

async fn root() -> impl IntoResponse {
    let mut address = env::var("AUTH_SERVICE_URL").unwrap_or("http://localhost:50051".to_owned());
    if address.is_empty() {
        address = "http://localhost:50051".to_string();
    }
    let login_link = address.to_string();
    let logout_link = format!("{}/logout", address);

    let template = IndexTemplate {
        login_link,
        logout_link,
    };
    Html(template.render().unwrap())
}

async fn protected(jar: CookieJar) -> impl IntoResponse {
    let jwt_cookie = match jar.get("jwt") {
        Some(cookie) => cookie,
        None => {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    let auth_hostname =
        env::var("AUTH_SERVICE_HOST_NAME").unwrap_or("http://localhost:50051".to_owned());
    let mut client = AuthServiceClient::connect(auth_hostname).await.unwrap();

    let request = tonic::Request::new(VerifyTokenRequest {
        token: jwt_cookie.value().to_string(),
    });

    match client.verify_token(request).await {
        Ok(_) => Json(ProtectedRouteResponse {
            img_url: "https://i.ibb.co/YP90j68/Light-Live-Bootcamp-Certificate.png".to_owned(),
        })
        .into_response(),
        Err(status) => match status.code() {
            tonic::Code::Unauthenticated | tonic::Code::InvalidArgument => {
                StatusCode::UNAUTHORIZED.into_response()
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
    }
}

#[derive(Serialize)]
pub struct ProtectedRouteResponse {
    pub img_url: String,
}
