use redis::{Client, RedisResult};
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

pub fn get_redis_client(redis_hostname: String) -> RedisResult<Client> {
    let redis_url = format!("redis://{redis_hostname}/");
    redis::Client::open(redis_url)
}
