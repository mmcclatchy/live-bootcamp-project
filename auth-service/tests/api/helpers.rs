use auth_proto::auth_service_client::AuthServiceClient;
use auth_service::{
    services::{app_state::AppState, hashmap_user_store::HashmapUserStore},
    GRPCApp, RESTApp,
};
use reqwest;
use std::net::SocketAddr;
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};
use tonic::transport::Channel;
use uuid::Uuid;

pub struct RESTTestApp {
    pub address: String,
    pub client: reqwest::Client,
    shutdown: Option<oneshot::Sender<()>>,
}

pub struct GRPCTestApp {
    pub address: SocketAddr,
    pub client: AuthServiceClient<Channel>,
    shutdown: Option<oneshot::Sender<()>>,
}

impl RESTTestApp {
    pub async fn new() -> Self {
        let user_store = HashmapUserStore::new();
        let app_state = AppState::new_arc(user_store);

        let rest_app = RESTApp::new(app_state);
        let address = rest_app.address.clone();

        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            tokio::select! {
                _ = rest_app.run() => {},
                _ = rx => {},
            }
        });

        // Wait for server to start
        sleep(Duration::from_millis(100)).await;

        let client = reqwest::Client::new();

        RESTTestApp {
            address: format!("http://{}", address),
            client,
            shutdown: Some(tx),
        }
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_error_message(response: reqwest::Response) -> String {
        response
            .json::<serde_json::Value>()
            .await
            .expect("Failed to parse error response")
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error")
            .to_string()
    }

    // Add other REST helper methods here (login, logout, verify_2fa, verify_token)...
}

impl GRPCTestApp {
    pub async fn new() -> Self {
        let user_store = HashmapUserStore::new();
        let app_state = AppState::new_arc(user_store);

        let grpc_app = GRPCApp::new(app_state);
        let address = grpc_app.address;

        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            tokio::select! {
                _ = grpc_app.run() => {},
                _ = rx => {},
            }
        });

        // Wait for server to start and attempt to connect
        let client = loop {
            match AuthServiceClient::connect(format!("http://{}", address)).await {
                Ok(client) => break client,
                Err(_) => {
                    sleep(Duration::from_millis(100)).await;
                }
            }
        };

        GRPCTestApp {
            address,
            client,
            shutdown: Some(tx),
        }
    }

    // Add gRPC helper methods here if needed...
}

impl Drop for RESTTestApp {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for GRPCTestApp {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}
