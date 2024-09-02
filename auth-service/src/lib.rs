use redis::{aio::ConnectionManager, Client, RedisResult};
use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgPoolOptions, PgPool};

pub mod api;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

pub use api::grpc::GRPCApp;
pub use api::rest::RESTApp;

pub async fn get_postgres_pool(url: &Secret<String>) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(url.expose_secret())
        .await
}

pub async fn get_redis_client(
    redis_hostname: String,
    redis_password: Option<Secret<String>>,
) -> RedisResult<ConnectionManager> {
    let redis_url = match redis_password.clone() {
        Some(redis_password) => {
            tracing::debug!("Redis URL: redis://REDACTED@{redis_hostname}:6379");
            format!("redis://{}@{redis_hostname}:6379", redis_password.expose_secret())
        }
        None => {
            tracing::debug!("Redis URL: redis://{redis_hostname}:6379");
            format!("redis://{redis_hostname}:6379")
        }
    };
    let client = Client::open(redis_url)?;
    let mut manager = ConnectionManager::new(client).await?;

    match redis_password {
        None => Ok(manager),
        Some(password) => {
            // Explicitly authenticate with the password
            let result: RedisResult<String> = redis::cmd("AUTH")
                .arg(password.expose_secret())
                .query_async(&mut manager)
                .await;

            match result {
                Ok(_) => {
                    tracing::info!("Successfully authenticated with Redis");
                    Ok(manager)
                }
                Err(e) => {
                    tracing::error!("Failed to authenticate with Redis: {:?}", e);
                    Err(e)
                }
            }
        }
    }
}
