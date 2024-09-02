use std::{ops::Deref, sync::Arc};

use redis::aio::ConnectionManager;
use reqwest::Client;
use sqlx::PgPool;
use tokio::sync::RwLock;

use auth_service::{
    domain::email::Email,
    get_postgres_pool, get_redis_client,
    services::{
        app_state::AppState,
        concrete_app_services::PersistentAppStateType,
        data_stores::{
            postgres_user_store::PostgresUserStore, redis_banned_token_store::RedisBannedTokenStore,
            redis_password_reset_token_store::RedisPasswordResetTokenStore,
            redis_two_fa_code_store::RedisTwoFACodeStore,
        },
        postmark_email_client::PostmarkEmailClient,
    },
    utils::{
        constants::{prod, DATABASE_URL, POSTMARK_AUTH_TOKEN, REDIS_HOST_NAME, REDIS_PASSWORD},
        tracing::init_tracing,
    },
    GRPCApp, RESTApp,
};

#[tracing::instrument(name = "Configure PostgreSQL")]
async fn configure_postgresql() -> PgPool {
    #[allow(clippy::to_string_in_format_args, clippy::unnecessary_to_owned)]
    let pg_pool = get_postgres_pool(DATABASE_URL.deref())
        .await
        .expect("Failed to create Postgres connection pool!");

    tracing::info!("Running migrations.");

    sqlx::migrate!().run(&pg_pool).await.expect("Failed to run migrations!");

    tracing::info!("Connection and migrations successful.");

    pg_pool
}

#[tracing::instrument(name = "Configure Redis")]
async fn configure_redis() -> Arc<RwLock<ConnectionManager>> {
    let conn = get_redis_client(REDIS_HOST_NAME.to_owned(), Some(REDIS_PASSWORD.to_owned()))
        .await
        .expect("Failed to get Redis ConnectionManager");
    Arc::new(RwLock::new(conn))
}

fn configure_postmark_email_client() -> PostmarkEmailClient {
    let http_client = Client::builder()
        .timeout(prod::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(
        prod::email_client::BASE_URL.to_owned(),
        Email::parse(prod::email_client::SENDER.to_owned().into()).unwrap(),
        POSTMARK_AUTH_TOKEN.to_owned(),
        http_client,
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");

    tracing::info!("Tracing initialized successfully");
    tracing::info!("Starting auth service");

    let pg_pool = configure_postgresql().await;
    let redis_conn = configure_redis().await;

    let app_state: PersistentAppStateType = AppState::new_arc(
        RedisBannedTokenStore::new(redis_conn.clone()),
        PostgresUserStore::new(pg_pool),
        RedisTwoFACodeStore::new(redis_conn.clone()),
        configure_postmark_email_client(),
        RedisPasswordResetTokenStore::new(redis_conn.clone()),
    );

    let address = prod::APP_GRPC_ADDRESS.to_string();
    let grpc_app = GRPCApp::new(app_state.clone(), address)
        .await
        .expect("Failed to create GRPCApp");
    let grpc_server = grpc_app.run();

    let address = prod::APP_REST_ADDRESS.to_string();
    let rest_app = RESTApp::new(app_state, address)
        .await
        .expect("Failed to create REST server");
    let rest_server = rest_app.run();

    // Run both servers concurrently
    tokio::select! {
        res = grpc_server => {
            if let Err(e) = res {
                tracing::error!("gRPC server error: {}", e);
            }
        }
        res = rest_server => {
            if let Err(e) = res {
                tracing::error!("REST server error: {}", e);
            }
        }
    }

    Ok(())
}
