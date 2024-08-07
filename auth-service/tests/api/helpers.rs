use auth_proto::auth_service_client::AuthServiceClient;
use auth_service::{
    domain::{
        data_stores::{UserStore, UserStoreError},
        email::Email,
        user::User,
    },
    services::{app_state::AppState, hashmap_user_store::HashmapUserStore},
    GRPCApp, RESTApp,
};
use reqwest;
use serde::{Deserialize, Serialize};
use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
};
use tokio::sync::{oneshot, RwLockReadGuard};
use tokio::time::{sleep, Duration};
use tonic::transport::Channel;
use uuid::Uuid;

pub struct RESTTestApp {
    pub address: String,
    pub client: reqwest::Client,
    pub app_state: Arc<AppState<HashmapUserStore>>,
    // _shutdown: Option<oneshot::Sender<()>>,
}

impl RESTTestApp {
    pub async fn new() -> Self {
        let user_store = HashmapUserStore::new();
        let app_state = AppState::new_arc(user_store);
        let address = String::from("127.0.0.1:3001");

        let rest_app = RESTApp::new(app_state.clone(), address);
        let address = rest_app.address.clone();

        // let (tx, rx) = oneshot::channel();
        // tokio::spawn(async move {
        //     tokio::select! {
        //         _ = rest_app.run() => {},
        //         _ = rx => {},
        //     }
        // });

        tokio::spawn(rest_app.run());

        // Wait for server to start
        sleep(Duration::from_millis(100)).await;

        let client = reqwest::Client::new();

        RESTTestApp {
            address: format!("http://{}", address),
            client,
            app_state: app_state.clone(),
            // _shutdown: Some(tx),
        }
    }

    pub async fn post_signup<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        let client_url = format!("{}/signup", &self.address);
        println!("[RESTTestApp][post_signup] Client URL: {}", client_url);
        self.client
            .post(&client_url)
            .json(body)
            .send()
            .await
            .expect("[RESTTestApp][post_signup] Failed to execute request.")
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

    pub async fn log_user_store(&self, fn_name: &str) {
        let user_store = self.app_state.user_store.read().await;
        println!("[TEST][{}] {:?}", fn_name, user_store);
    }
}

pub struct GRPCTestApp {
    pub address: SocketAddr,
    pub client: AuthServiceClient<Channel>,
    pub app_state: Arc<AppState<HashmapUserStore>>,
    // shutdown: Option<oneshot::Sender<()>>,
}

impl GRPCTestApp {
    pub async fn new() -> Self {
        // let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
        // let port = listener.local_addr().unwrap().port();
        // let address = format!("127.0.0.1:{}", port);
        // let address = listener.local_addr().unwrap().to_string();

        let user_store = HashmapUserStore::new();
        let user_store_id = user_store.get_id();
        let app_state = Arc::new(AppState::new(user_store));

        let address = "127.0.0.1:50052".to_string();
        println!(
            "[GRPCTestApp][new] Bound to address: {} with UserStore id: {}",
            address, user_store_id
        );
        let grpc_app = GRPCApp::new(app_state.clone(), address);
        let address = grpc_app.address;

        // let (tx, rx) = oneshot::channel();
        // tokio::spawn(async move {
        //     tokio::select! {
        //         _ = grpc_app.run() => {},
        //         _ = rx => {},
        //     }
        // });

        tokio::spawn(grpc_app.run());

        // Wait for the server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        #[allow(clippy::expect_fun_call)]
        let client = AuthServiceClient::connect("http://127.0.0.1:50052")
            .await
            .expect("Failed to create gRPC client");

        GRPCTestApp {
            address,
            client,
            app_state: app_state.clone(),
            // shutdown: Some(tx),
        }
    }

    pub async fn log_user_store(&self, fn_name: &str) {
        let user_store = self.app_state.user_store.read().await;
        println!("[TEST][{}] {:?}", fn_name, user_store);
    }
}

// impl Drop for GRPCTestApp {
//     fn drop(&mut self) {
//         if let Some(tx) = self.shutdown.take() {
//             let _ = tx.send(());
//         }
//     }
// }

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

pub async fn wait_for_user<'a, T: UserStore>(
    user_store: RwLockReadGuard<'a, T>,
    email: &Email,
    max_retries: u8,
    delay_ms: u64,
) -> Result<User, UserStoreError> {
    for _ in 0..max_retries {
        match user_store.get_user(email).await {
            Ok(user) => return Ok(user),
            Err(UserStoreError::UserNotFound) => {
                sleep(Duration::from_millis(delay_ms)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(UserStoreError::UserNotFound)
}
