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
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLockReadGuard;
use tokio::time::{sleep, Duration};
use tonic::transport::Channel;
use uuid::Uuid;

pub struct RESTTestApp {
    pub address: String,
    pub client: reqwest::Client,
    pub app_state: Arc<AppState<HashmapUserStore>>,
}

impl RESTTestApp {
    pub async fn new() -> Self {
        let user_store = HashmapUserStore::new();
        let user_store_id = user_store.get_id();
        let app_state = AppState::new_arc(user_store);
        let address = String::from("127.0.0.1:0");

        println!(
            "[GRPCTestApp][new] Bound to address: {address} with UserStore id: {user_store_id}"
        );

        let rest_app = RESTApp::new(app_state.clone(), address)
            .await
            .expect("should create rest app");
        let address = rest_app.address.clone();

        tokio::spawn(rest_app.run());

        sleep(Duration::from_millis(100)).await;

        let client = reqwest::Client::new();

        RESTTestApp {
            address: format!("http://{}", address),
            client,
            app_state: app_state.clone(),
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

    pub async fn log_user_store(&self, fn_name: &str) {
        let user_store = self.app_state.user_store.read().await;
        println!("[TEST][{}] {:?}", fn_name, user_store);
    }
}

pub struct GRPCTestApp {
    pub address: String,
    pub client: AuthServiceClient<Channel>,
    pub app_state: Arc<AppState<HashmapUserStore>>,
}

impl GRPCTestApp {
    pub async fn new() -> Self {
        let user_store = HashmapUserStore::new();
        let user_store_id = user_store.get_id();
        let app_state = Arc::new(AppState::new(user_store));
        let address = String::from("127.0.0.1:0");

        let grpc_app = GRPCApp::new(app_state.clone(), address.clone())
            .await
            .expect("[ERROR][GRPCTestApp][new] Failed to create GRPCApp");
        let address = grpc_app.address;
        println!(
            "[GRPCTestApp][new] Bound to address: {address} with UserStore id: {user_store_id}"
        );

        tokio::spawn(grpc_app.run());

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let address = format!("http://{address}");
        let client = AuthServiceClient::connect(address.clone())
            .await
            .expect("[ERROR][GRPCTestApp][new] Failed to create gRPC client");

        GRPCTestApp {
            address,
            client,
            app_state: app_state.clone(),
        }
    }

    pub async fn log_user_store(&self, fn_name: &str) {
        let user_store = self.app_state.user_store.read().await;
        println!("[TEST][{}] {:?}", fn_name, user_store);
    }
}

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
