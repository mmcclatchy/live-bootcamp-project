use std::env;

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use axum_extra::extract::CookieJar;
use serde::Serialize;
use tower_http::services::ServeDir;

use app_proto::{auth_service_client::AuthServiceClient, VerifyTokenRequest};

#[derive(Clone)]
struct AppConfig {
    rest_auth_service_url: String,
    grpc_auth_service_url: String,
    rest_auth_service_host: String,
    grpc_auth_service_host: String,
}

impl AppConfig {
    fn from_env() -> Self {
        Self {
            rest_auth_service_url: env::var("REST_AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost/auth".to_owned()),
            grpc_auth_service_url: env::var("GRPC_AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://0.0.0.0:50051".to_owned()),
            rest_auth_service_host: env::var("REST_AUTH_SERVICE_HOST")
                .unwrap_or_else(|_| "localhost".to_owned()),
            grpc_auth_service_host: env::var("GRPC_AUTH_SERVICE_HOST")
                .unwrap_or_else(|_| "0.0.0.0:50051".to_owned()),
        }
    }
}

#[tokio::main]
async fn main() {
    let config = AppConfig::from_env();

    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .route("/", get(rest_root))
        .route("/protected", get(rest_protected))
        .route("/auth/login", get(rest_login))
        .route("/auth/logout", get(rest_logout))
        .route("/grpc", get(grpc_root))
        .route("/grpc/protected", get(grpc_protected))
        .route("/grpc/auth/login", get(grpc_login))
        .route("/grpc/auth/logout", get(grpc_logout))
        .with_state(config);

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

fn create_index_template(base_url: &str) -> IndexTemplate {
    IndexTemplate {
        login_link: base_url.to_string(),
        logout_link: format!("{}/logout", base_url),
    }
}

async fn rest_root(
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    let template = create_index_template(&config.rest_auth_service_url);
    Html(template.render().unwrap())
}

async fn grpc_root(
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    let template = create_index_template(&config.grpc_auth_service_url);
    Html(template.render().unwrap())
}

async fn rest_protected(
    jar: CookieJar,
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    let jwt_cookie = match jar.get("jwt") {
        Some(cookie) => cookie,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let api_client = reqwest::Client::builder().build().unwrap();
    let verify_token_body = serde_json::json!({ "token": jwt_cookie.value() });
    let url = format!("http://{}/auth/verify-token", config.rest_auth_service_host);

    let response = match api_client.post(&url).json(&verify_token_body).send().await {
        Ok(response) => response,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match response.status() {
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::BAD_REQUEST => {
            StatusCode::UNAUTHORIZED.into_response()
        }
        reqwest::StatusCode::OK => Json(ProtectedRouteResponse::new()).into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn grpc_protected(
    jar: CookieJar,
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    let jwt_cookie = match jar.get("jwt") {
        Some(cookie) => cookie,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let mut client = match AuthServiceClient::connect(config.grpc_auth_service_host).await {
        Ok(client) => client,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let request = tonic::Request::new(VerifyTokenRequest {
        token: jwt_cookie.value().to_string(),
    });

    match client.verify_token(request).await {
        Ok(_) => Json(ProtectedRouteResponse::new()).into_response(),
        Err(status) => match status.code() {
            tonic::Code::Unauthenticated | tonic::Code::InvalidArgument => {
                StatusCode::UNAUTHORIZED.into_response()
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
    }
}

fn create_redirect(base_url: &str, endpoint: &str) -> impl IntoResponse {
    Redirect::to(&format!("{}/{}", base_url, endpoint))
}

async fn rest_login(
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    create_redirect(&config.rest_auth_service_url, "login")
}

async fn rest_logout(
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    create_redirect(&config.rest_auth_service_url, "logout")
}

async fn grpc_login(
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    create_redirect(&config.grpc_auth_service_url, "login")
}

async fn grpc_logout(
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    create_redirect(&config.grpc_auth_service_url, "logout")
}

#[derive(Serialize)]
pub struct ProtectedRouteResponse {
    pub img_url: String,
}

impl ProtectedRouteResponse {
    fn new() -> Self {
        Self {
            img_url: "https://i.ibb.co/YP90j68/Light-Live-Bootcamp-Certificate.png".to_owned(),
        }
    }
}
