use auth_proto::auth_service_client::AuthServiceClient;
use auth_proto::auth_service_server::AuthServiceServer;
use auth_service::{
    services::{app_state::AppState, hashmap_user_store::HashmapUserStore},
    AuthServiceImpl,
};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};
use tonic::transport::Channel;
use uuid::Uuid;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(50051);

pub struct TestApp {
    pub address: SocketAddr,
    pub client: AuthServiceClient<Channel>,
    shutdown: Option<oneshot::Sender<()>>,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = HashmapUserStore::new();
        let app_state = AppState::new_arc(user_store);

        for _ in 0..10 {
            // Try up to 10 different ports
            let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
            let addr: SocketAddr = format!("127.0.0.1:{}", port)
                .parse()
                .expect("Failed to parse address");
            let auth_service = AuthServiceImpl::new(app_state.clone());

            let (tx, rx) = oneshot::channel();

            // Start the server in a separate task
            let server_handle = tokio::spawn(async move {
                if let Err(e) = tonic::transport::Server::builder()
                    .add_service(AuthServiceServer::new(auth_service))
                    .serve_with_shutdown(addr, async {
                        rx.await.ok();
                    })
                    .await
                {
                    eprintln!("Server error on port {}: {:?}", port, e);
                } else {
                    println!("Server on port {} shut down gracefully", port);
                }
            });

            // Give the server a moment to start
            sleep(Duration::from_millis(100)).await;

            // Try to connect to the server
            match AuthServiceClient::connect(format!("http://{}", addr)).await {
                Ok(client) => {
                    println!("Successfully connected to server on port {}", port);
                    return TestApp {
                        address: addr,
                        client,
                        shutdown: Some(tx),
                    };
                }
                Err(e) => {
                    eprintln!("Failed to connect on port {}: {:?}", port, e);
                    // Ensure the server is shut down before trying the next port
                    let _ = tx.send(());
                    let _ = server_handle.await;
                }
            }
        }

        panic!("Failed to start the server after multiple attempts");
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}
