use auth_service::{
    services::{app_state::AppState, hashmap_user_store::HashmapUserStore},
    Application,
};
use log::{error, info};
use std::io::Write;
use std::{env, net::SocketAddr, panic};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    info!("Log: auth-service/src/main.rs::main invoked");
    std::io::stderr().flush().unwrap();
    std::io::stdout().flush().unwrap();

    panic::set_hook(Box::new(|panic_info| {
        error!("Panic occurred: {:?}", panic_info);
    }));

    info!("Starting auth service");

    let user_store = HashmapUserStore::new();
    info!("User store initialized");

    let app_state = AppState::new_arc(user_store);
    info!("App state created");

    let addr: SocketAddr = env::var("GRPC_LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:50051".to_string())
        .parse()?;
    info!("Address parsed: {}", addr);

    let app = Application::new(addr);
    info!("Application instance created");

    info!("Running the application");
    app.run(app_state).await?;

    // Force an error
    // Err("Forced error for testing".into())
    Ok(())
}
