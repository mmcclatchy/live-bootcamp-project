use std::{env, panic};

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
    env_logger::init();
    println!("Starting app-service");
    panic::set_hook(Box::new(|panic_info| {
        println!("Panic occurred: {:?}", panic_info);
    }));

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
    println!("[rest_root] Called");
    let template = create_index_template(&config.rest_auth_service_url);
    Html(template.render().unwrap())
}

async fn grpc_root(
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    println!("[grpc_root] Called");
    let template = create_index_template(&config.grpc_auth_service_url);
    Html(template.render().unwrap())
}

async fn rest_protected(
    jar: CookieJar,
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    println!("[rest_protected] Called");

    let jwt_cookie = match jar.get("jwt") {
        Some(cookie) => {
            println!("[rest_protected] JWT cookie found: {}", cookie.value());
            cookie
        }
        None => {
            println!("[rest_protected] No JWT cookie found");
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    let api_client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            println!("[rest_protected] Failed to build API client: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let verify_token_body = serde_json::json!({ "token": jwt_cookie.value() });
    let url = format!("http://{}/verify-token", config.rest_auth_service_host);
    println!("[rest_protected] Verify Token URL: {}", url);

    let response = match api_client.post(&url).json(&verify_token_body).send().await {
        Ok(response) => {
            println!("[rest_protected] Verify Token Response Ok");
            response
        }
        Err(e) => {
            println!(
                "[rest_protected] Failed to send request to auth service: {:?}",
                e
            );
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match response.status() {
        reqwest::StatusCode::OK => {
            println!("[rest_protected] Token validation succeeded");
            Json(ProtectedRouteResponse::new()).into_response()
        }
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::BAD_REQUEST => {
            println!(
                "[rest_protected] Token validation failed: {:?}",
                response.status()
            );
            StatusCode::UNAUTHORIZED.into_response()
        }
        _ => {
            println!(
                "[rest_protected] Unexpected response from auth service: {:?}",
                response.status()
            );
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());
            println!("[rest_protected] Response body: {}", body);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn grpc_protected(
    jar: CookieJar,
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    println!("[grpc_protected] Called");

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
    println!("[create_redirect] Called");
    let redirect_uri = &format!("{}/{}", base_url, endpoint);
    println!("[create_redirect] Redirect URI: {redirect_uri}");
    Redirect::to(redirect_uri)
}

async fn rest_login(
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    println!("[rest_login] Called");
    create_redirect(&config.rest_auth_service_url, "login")
}

async fn rest_logout(
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    println!("[rest_logout] Called");
    create_redirect(&config.rest_auth_service_url, "logout")
}

async fn grpc_login(
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    println!("[grpc_login] Called");
    create_redirect(&config.grpc_auth_service_url, "login")
}

async fn grpc_logout(
    axum::extract::State(config): axum::extract::State<AppConfig>,
) -> impl IntoResponse {
    println!("[grpc_logout] Called");
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
