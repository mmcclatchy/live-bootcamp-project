use auth_service::{
    services::{app_state::AppState, hashmap_user_store::HashmapUserStore},
    Application,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user_store = HashmapUserStore::new();
    let app_state = AppState::new_arc(user_store);

    let addr: SocketAddr = "0.0.0.0:50051".parse()?;
    let app = Application::new(addr);

    app.run(app_state).await
}
