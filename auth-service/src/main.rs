use std::io::Write;

use auth_service::{
    services::{
        app_state::AppState, hashmap_banned_token_store::HashMapBannedTokenStore,
        hashmap_two_fa_code_store::HashMapTwoFACodeStore, hashmap_user_store::HashmapUserStore,
    },
    utils::constants::prod,
    GRPCApp, RESTApp,
};
use log::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .try_init()
        .unwrap_or_else(|e| eprintln!("Failed to initialize tracing: {:?}", e));

    eprintln!("Tracing initialized successfully");
    std::io::stderr().flush().unwrap();
    info!("Starting auth service");

    let banned_token_store = HashMapBannedTokenStore::new();
    let user_store = HashmapUserStore::new();
    let two_fa_code_store = HashMapTwoFACodeStore::new();
    let app_state = AppState::new_arc(banned_token_store, user_store, two_fa_code_store);

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
