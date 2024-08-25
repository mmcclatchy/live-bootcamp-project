use std::{io::Write, sync::Arc};

use auth_service::{
    get_postgres_pool, get_redis_client,
    services::{
        app_state::AppState,
        concrete_app_services::PersistentAppStateType,
        data_stores::{
            postgres_user_store::PostgresUserStore, redis_banned_token_store::RedisBannedTokenStore,
            redis_password_reset_token_store::RedisPasswordResetTokenStore,
            redis_two_fa_code_store::RedisTwoFACodeStore,
        },
        mock_email_client::MockEmailClient,
    },
    utils::{
        constants::{prod, DATABASE_URL, REDIS_HOST_NAME},
        tracing::init_tracing,
    },
    GRPCApp, RESTApp,
};
use log::{error, info};
use sqlx::PgPool;
use tokio::sync::RwLock;

async fn configure_postgresql() -> PgPool {
    #[allow(clippy::to_string_in_format_args, clippy::unnecessary_to_owned)]
    let prod_db_url = format!("{}/rust-bc", DATABASE_URL.to_string());
    println!("[main][configure_postgresql] Attempting to connect to PostgreSQL at: {prod_db_url}");
    let pg_pool = get_postgres_pool(&prod_db_url)
        .await
        .expect("[ERROR][main][configure_postgresql] Failed to create Postgres connection pool!");

    println!("[main][configure_postgresql] Running migrations.");

    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("[ERROR][main][configure_postgresql] Failed to run migrations!");

    println!("[main][configure_postgresql] Connection and migrations successful.");

    pg_pool
}

fn configure_redis() -> Arc<RwLock<redis::Connection>> {
    let conn = get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis Client")
        .get_connection()
        .expect("Failed to get Redis Connection");
    Arc::new(RwLock::new(conn))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    eprintln!("Tracing initialized successfully");
    std::io::stderr().flush().unwrap();
    info!("Starting auth service");

    let pg_pool = configure_postgresql().await;
    let redis_conn = configure_redis();

    let app_state: PersistentAppStateType = AppState::new_arc(
        RedisBannedTokenStore::new(redis_conn.clone()),
        PostgresUserStore::new(pg_pool),
        RedisTwoFACodeStore::new(redis_conn.clone()),
        MockEmailClient,
        RedisPasswordResetTokenStore::new(redis_conn.clone()),
    );

    let address = prod::APP_GRPC_ADDRESS.to_string();
    let grpc_app = GRPCApp::new(app_state.clone(), address)
        .await
        .expect("[ERROR][main] Failed to create GRPCApp");
    let grpc_server = grpc_app.run();

    let address = prod::APP_REST_ADDRESS.to_string();
    let rest_app = RESTApp::new(app_state, address)
        .await
        .expect("[ERROR][main] Failed to create REST server");
    let rest_server = rest_app.run();

    // Run both servers concurrently
    tokio::select! {
        res = grpc_server => {
            if let Err(e) = res {
                error!("gRPC server error: {}", e);
            }
        }
        res = rest_server => {
            if let Err(e) = res {
                error!("REST server error: {}", e);
            }
        }
    }

    Ok(())
}
