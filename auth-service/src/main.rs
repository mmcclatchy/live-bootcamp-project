use std::io::Write;

use auth_service::{
    get_postgres_pool,
    services::{
        app_state::AppState,
        concrete_app_services::PersistentAppStateType,
        data_stores::postgres_user_store::PostgresUserStore,
        hashmap_banned_token_store::HashMapBannedTokenStore,
        hashmap_password_reset_token_store::HashMapPasswordResetTokenStore,
        hashmap_two_fa_code_store::HashMapTwoFACodeStore,
        // hashmap_user_store::HashmapUserStore,
        mock_email_client::MockEmailClient,
    },
    utils::constants::{prod, DATABASE_URL},
    GRPCApp, RESTApp,
};
use log::{error, info};
use sqlx::PgPool;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

async fn configure_postgresql() -> PgPool {
    #[allow(clippy::to_string_in_format_args, clippy::unnecessary_to_owned)]
    let prod_db_url = format!("{}/rust-bc", DATABASE_URL.to_string());
    let pg_pool = get_postgres_pool(&prod_db_url)
        .await
        .expect("[ERROR][main][configure_postgresql] Failed to create Postgres connection pool!");

    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("[ERROR][main][configure_postgresql] Failed to run migrations!");

    pg_pool
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .try_init()
        .unwrap_or_else(|e| {
            eprintln!(
                "[ERROR][tracing_subscriber::registry] Failed to initialize tracing: {:?}",
                e
            )
        });

    eprintln!("Tracing initialized successfully");
    std::io::stderr().flush().unwrap();
    info!("Starting auth service");

    let pg_pool = configure_postgresql().await;

    let app_state: PersistentAppStateType = AppState::new_arc(
        HashMapBannedTokenStore::new(),
        PostgresUserStore::new(pg_pool),
        HashMapTwoFACodeStore::new(),
        MockEmailClient,
        HashMapPasswordResetTokenStore::new(),
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
