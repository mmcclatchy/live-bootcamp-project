use std::sync::Arc;

use auth_service::services::data_stores::redis_banned_token_store::RedisBannedTokenStore;
use auth_service::services::data_stores::redis_password_reset_token_store::RedisPasswordResetTokenStore;
use auth_service::services::data_stores::redis_two_fa_code_store::RedisTwoFACodeStore;
use reqwest::cookie::Jar;
use serde::Serialize;
use serde_json::json;
use tokio::sync::RwLockReadGuard;
use tokio::time::{sleep, Duration};
use tonic::transport::Channel;
use uuid::Uuid;

use auth_proto::auth_service_client::AuthServiceClient;
use auth_service::{
    domain::{
        data_stores::{PasswordResetTokenStore, UserStore, UserStoreError},
        email::Email,
        user::User,
    },
    services::{
        app_state::AppState,
        concrete_app_services::{MemoryAppStateType, PersistentAppStateType},
        data_stores::postgres_user_store::PostgresUserStore,
        hashmap_banned_token_store::HashMapBannedTokenStore,
        hashmap_password_reset_token_store::HashMapPasswordResetTokenStore,
        hashmap_two_fa_code_store::HashMapTwoFACodeStore,
        hashmap_user_store::HashmapUserStore,
        mock_email_client::MockEmailClient,
    },
    utils::constants::{test, JWT_COOKIE_NAME},
    GRPCApp, RESTApp,
};

use crate::db::{configure_postgresql, configure_redis, delete_database};

pub struct RESTTestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub client: reqwest::Client,
    pub app_state: PersistentAppStateType,
    pub test_db_name: String,
    clean_up_called: bool,
}

impl RESTTestApp {
    pub async fn new() -> Self {
        let (pg_pool, db_name) = configure_postgresql().await;
        let user_store = PostgresUserStore::new(pg_pool);
        let redis_conn = configure_redis();
        let app_state = AppState::new_arc(
            RedisBannedTokenStore::new(redis_conn.clone()),
            user_store,
            RedisTwoFACodeStore::new(redis_conn.clone()),
            MockEmailClient,
            RedisPasswordResetTokenStore::new(redis_conn.clone()),
        );
        let address = String::from(test::APP_REST_ADDRESS);

        println!("[RESTTestApp][new] Bound to address: {address} with DbName: {db_name}");

        let rest_app = RESTApp::new(app_state.clone(), address)
            .await
            .expect("[ERROR][RESTTestApp][new] Failed to create RESTApp");
        let address = rest_app.address.clone();

        tokio::spawn(rest_app.run());

        sleep(Duration::from_millis(100)).await;

        let cookie_jar = Arc::new(Jar::default());
        let client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        Self {
            address: format!("http://{address}"),
            cookie_jar,
            client,
            app_state: app_state.clone(),
            test_db_name: db_name.to_string(),
            clean_up_called: false,
        }
    }

    pub async fn clean_up(&mut self) -> Result<(), String> {
        delete_database(&self.test_db_name).await?;
        self.clean_up_called = true;
        Ok(())
    }

    pub async fn post_signup<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        let client_url = format!("{}/signup", &self.address);
        println!("[RESTTestApp][post_signup] Client URL: {client_url}");
        self.client
            .post(&client_url)
            .json(body)
            .send()
            .await
            .expect("[RESTTestApp][post_signup] Failed to execute request.")
    }

    pub async fn post_login<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        let client_url = format!("{}/login", &self.address);
        println!("[RESTTestApp][post_login] Client URL: {client_url}");
        self.client
            .post(client_url)
            .json(body)
            .send()
            .await
            .expect("[ERROR][RESTTestApp][post_login] Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        let client_url = format!("{}/logout", &self.address);
        println!("[RESTTestApp][post_logout] Client URL: {client_url}");
        self.client
            .post(client_url)
            .send()
            .await
            .expect("[ERROR][RESTTestApp][post_logout] Failed to execute request.")
    }

    pub async fn post_verify_token<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        let client_url = format!("{}/verify-token", &self.address);
        println!("[RESTTestApp][post_verify_token] Client URL: {client_url}");
        self.client
            .post(client_url)
            .json(body)
            .send()
            .await
            .expect("[ERROR][RESTTestApp][post_verify_token] Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        let client_url = format!("{}/verify-2fa", &self.address);
        self.client
            .post(client_url)
            .json(body)
            .send()
            .await
            .expect("[ERROR][RESTTestApp][post_verify_2fa] Failed to execute request.")
    }

    pub async fn log_user_store(&self, fn_name: &str) {
        let user_store = self.app_state.user_store.read().await;
        println!("[{}] {:?}", fn_name, user_store);
    }

    pub async fn post_initiate_password_reset<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        let client_url = format!("{}/initiate-password-reset", &self.address);
        println!("[RESTTestApp][post_initiate_password_reset] Client URL: {client_url}");
        self.client
            .post(&client_url)
            .json(body)
            .send()
            .await
            .expect("[RESTTestApp][post_initiate_password_reset] Failed to execute request.")
    }

    pub async fn post_reset_password<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        let client_url = format!("{}/reset-password", &self.address);
        println!("[RESTTestApp][post_reset_password] Client URL: {client_url}");
        self.client
            .post(&client_url)
            .json(body)
            .send()
            .await
            .expect("[RESTTestApp][post_reset_password] Failed to execute request.")
    }

    pub async fn get_password_reset_token(&self, email: &str) -> Option<String> {
        let email = Email::parse(email.to_string()).ok()?;
        let token_store = self.app_state.password_reset_token_store.read().await;
        token_store.get_token(&email).await.ok()
    }
}

impl Drop for RESTTestApp {
    fn drop(&mut self) {
        if self.clean_up_called == false {
            panic!("RESTTestApp clean_up not called")
        }
    }
}

pub struct GRPCTestApp {
    // pub address: String,
    pub client: AuthServiceClient<Channel>,
    pub app_state: MemoryAppStateType,
}

impl GRPCTestApp {
    pub async fn new() -> Self {
        let user_store = HashmapUserStore::new();
        // let user_store_id = user_store.get_id();
        let app_state = Arc::new(AppState::new(
            HashMapBannedTokenStore::new(),
            user_store,
            HashMapTwoFACodeStore::new(),
            MockEmailClient,
            HashMapPasswordResetTokenStore::new(),
        ));
        let address = String::from(test::APP_GRPC_ADDRESS);

        let grpc_app = GRPCApp::new(app_state.clone(), address.clone())
            .await
            .expect("[ERROR][GRPCTestApp][new] Failed to create GRPCApp");
        let address = grpc_app.address;
        // println!(
        //     "[GRPCTestApp][new] Bound to address: {address} with UserStore id: {user_store_id}"
        // );

        tokio::spawn(grpc_app.run());

        sleep(Duration::from_millis(100)).await;

        let address = format!("http://{address}");
        let client = AuthServiceClient::connect(address.clone())
            .await
            .expect("[ERROR][GRPCTestApp][new] Failed to create gRPC client");

        Self {
            // address,
            client,
            app_state: app_state.clone(),
        }
    }

    pub async fn log_user_store(&self, fn_name: &str) {
        let user_store = self.app_state.user_store.read().await;
        println!("[{}] {:?}", fn_name, user_store);
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

pub async fn create_app_with_logged_in_token() -> (RESTTestApp, String) {
    let app = RESTTestApp::new().await;
    let email = get_random_email();
    let signup_body = json!({
        "email": email,
        "password": "P@ssw0rd",
        "requires2FA": false,
    });
    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status(), 201);
    let login_body = json!({
        "email": email,
        "password": "P@ssw0rd",
    });
    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status(), 200);
    let cookie = login_response
        .cookies()
        .find(|c| c.name() == JWT_COOKIE_NAME)
        .expect("[ERROR][Test Helper][create_app_with_logged_in_token] No auth cookie returned");
    assert!(!cookie.value().is_empty());
    let token = cookie.value().to_string();

    (app, token)
}

// pub fn print_app_state<S: AppServices>(app_state: &AppState<S>, prefix: &str) {
//     println!("\n------------ AppState ------------");
//     println!("{prefix} {:?}", app_state.banned_token_store);
//     println!("{prefix} {:?}", app_state.user_store);
//     println!("{prefix} {:?}", app_state.two_fa_code_store);
//     println!("{prefix} {:?}", app_state.password_reset_token_store);
//     println!("----------------------------------\n");
// }
