use auth_service::{
    services::{app_state::AppState, hashmap_user_store::HashmapUserStore},
    GRPCApp, RESTApp,
};
use log::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    info!("Starting auth service");

    let user_store = HashmapUserStore::new();
    let app_state = AppState::new_arc(user_store);

    let grpc_app = GRPCApp::new(app_state.clone());
    let grpc_server = grpc_app.run();

    // REST server
    let rest_app = RESTApp::new(app_state);
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
