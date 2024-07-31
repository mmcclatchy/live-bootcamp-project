use auth_proto::{auth_service_client::AuthServiceClient, auth_service_server::AuthServiceServer};
use auth_service::{
    services::{app_state::AppState, hashmap_user_store::HashmapUserStore},
    AuthServiceImpl,
};
use std::net::SocketAddr;
use tonic::transport::Channel;
use uuid::Uuid;

pub struct TestApp {
    pub address: SocketAddr,
    pub client: AuthServiceClient<Channel>,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = HashmapUserStore::new();
        let app_state = AppState::new_arc(user_store);

        // Use a random port
        let addr: SocketAddr = "127.0.0.1:0".parse().expect("Failed to parse address");
        let auth_service = AuthServiceImpl::new(app_state.clone());

        // Start the server in a separate task
        let (_tx, rx) = tokio::sync::oneshot::channel::<()>();
        tokio::spawn(async move {
            let server = tonic::transport::Server::builder()
                .add_service(AuthServiceServer::new(auth_service))
                .serve_with_shutdown(addr, async {
                    rx.await.ok();
                });
            server.await.expect("Server failed to start");
        });

        // Wait for the server to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let client = AuthServiceClient::connect(format!("http://{}", addr))
            .await
            .expect("Failed to create client");

        TestApp {
            address: addr,
            client,
        }
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}
