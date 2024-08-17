use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
    pub static ref REST_AUTH_SERVICE_URL: String = set_rest_auth_service_url();
}

fn set_token() -> String {
    dotenv().ok();
    let secret = std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.");
    }
    secret
}

fn set_rest_auth_service_url() -> String {
    dotenv().ok();
    std_env::var(env::REST_AUTH_SERVICE_URL_ENV_VAR)
        .unwrap_or_else(|_| "http://localhost/auth".to_string())
}

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const REST_AUTH_SERVICE_URL_ENV_VAR: &str = "REST_AUTH_SERVICE_URL";
}

pub mod prod {
    pub const APP_REST_ADDRESS: &str = "0.0.0.0:3000";
    pub const APP_GRPC_ADDRESS: &str = "0.0.0.0:50051";
}

pub mod test {
    pub const APP_REST_ADDRESS: &str = "127.0.0.1:0";
    pub const APP_GRPC_ADDRESS: &str = "127.0.0.1:0";
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const TOKEN_TTL_SECONDS: i64 = 600;
pub const PASSWORD_RESET_TOKEN_TTL_SECONDS: i64 = 3600;

pub type Epoch = u32;
