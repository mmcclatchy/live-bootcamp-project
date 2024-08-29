use dotenvy::dotenv;
use lazy_static::lazy_static;
use secrecy::Secret;
use std::env as std_env;

lazy_static! {
    pub static ref DATABASE_URL: Secret<String> = Secret::new(set_required_env_var(env::DATABASE_URL_ENV_VAR));
    pub static ref JWT_SECRET: Secret<String> = Secret::new(set_required_env_var(env::JWT_SECRET_ENV_VAR));
    pub static ref POSTMARK_AUTH_TOKEN: Secret<String> =
        Secret::new(set_required_env_var(env::POSTMARK_AUTH_TOKEN_ENV_VAR));
    pub static ref REDIS_HOST_NAME: String = set_default_env_var(env::REDIS_HOST_NAME_ENV_VAR, DEFAULT_REDIS_HOST_NAME);
    pub static ref REST_AUTH_SERVICE_URL: String =
        set_default_env_var(env::REST_AUTH_SERVICE_URL_ENV_VAR, "http://localhost/auth");
}

fn set_default_env_var(var_name: &str, default_value: &str) -> String {
    dotenv().ok();
    std_env::var(var_name).unwrap_or_else(|_| default_value.to_string())
}

fn set_required_env_var(var_name: &str) -> String {
    println!("Attempting to obtain {var_name} environment variable");
    dotenv().ok();
    println!("dotenv Ok");
    let var_value = std_env::var(var_name).expect("{var_name} must be set.");
    println!("{var_name}:  {var_value}");
    if var_value.is_empty() {
        panic!("{var_name} must not be empty.");
    }
    var_value
}

pub mod env {
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const POSTMARK_AUTH_TOKEN_ENV_VAR: &str = "POSTMARK_AUTH_TOKEN";
    pub const REST_AUTH_SERVICE_URL_ENV_VAR: &str = "REST_AUTH_SERVICE_URL";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
}

pub mod prod {
    pub const APP_REST_ADDRESS: &str = "0.0.0.0:3000";
    pub const APP_GRPC_ADDRESS: &str = "0.0.0.0:50051";

    pub mod email_client {
        use std::time::Duration;

        pub const BASE_URL: &str = "https://api.postmarkapp.com/email";
        pub const SENDER: &str = "do-not-reply@markmcclatchy.com";
        pub const TIMEOUT: Duration = std::time::Duration::from_secs(10);
    }
}

pub mod test {
    pub const APP_REST_ADDRESS: &str = "127.0.0.1:0";
    pub const APP_GRPC_ADDRESS: &str = "127.0.0.1:0";
    pub const DATABASE_URL: &str = "postgres://postgres:password@localhost:5432";

    pub mod email_client {
        use std::time::Duration;

        pub const SENDER: &str = "test@email.com";
        pub const TIMEOUT: Duration = std::time::Duration::from_millis(200);
    }
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const TOKEN_TTL_SECONDS: i64 = 600;
pub const PASSWORD_RESET_TOKEN_TTL_SECONDS: i64 = 3600;
pub const DEFAULT_REDIS_HOST_NAME: &str = "127.0.0.1";

pub type Epoch = u32;
