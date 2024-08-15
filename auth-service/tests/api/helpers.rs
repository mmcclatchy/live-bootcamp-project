use auth_proto::auth_service_client::AuthServiceClient;
use auth_service::{
    domain::{
        data_stores::{BannedTokenStore, TwoFACodeStore, UserStore, UserStoreError},
        email::Email,
        email_client::EmailClient,
        user::User,
    },
    services::{
        app_state::AppState, hashmap_banned_token_store::HashMapBannedTokenStore,
        hashmap_two_fa_code_store::HashMapTwoFACodeStore, hashmap_user_store::HashmapUserStore,
        mock_email_client::MockEmailClient,
    },
    utils::constants::{test, JWT_COOKIE_NAME},
    GRPCApp, RESTApp,
};
use reqwest::cookie::Jar;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLockReadGuard;
use tokio::time::{sleep, Duration};
use tonic::transport::Channel;
use uuid::Uuid;

pub struct RESTTestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub client: reqwest::Client,
    pub app_state: Arc<
        AppState<HashMapBannedTokenStore, HashmapUserStore, HashMapTwoFACodeStore, MockEmailClient>,
    >,
}

impl RESTTestApp {
    pub async fn new() -> Self {
        let banned_token_store = HashMapBannedTokenStore::new();
        let user_store = HashmapUserStore::new();
        let user_store_id = user_store.get_id();
        let two_factor_code_store = HashMapTwoFACodeStore::new();
        let email_client = MockEmailClient;
        let app_state = AppState::new_arc(
            banned_token_store,
            user_store,
            two_factor_code_store,
            email_client,
        );
        let address = String::from(test::APP_REST_ADDRESS);

        println!(
            "[GRPCTestApp][new] Bound to address: {address} with UserStore id: {user_store_id}"
        );

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
        }
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
}

pub struct GRPCTestApp {
    pub address: String,
    pub client: AuthServiceClient<Channel>,
    pub app_state: Arc<
        AppState<HashMapBannedTokenStore, HashmapUserStore, HashMapTwoFACodeStore, MockEmailClient>,
    >,
}

impl GRPCTestApp {
    pub async fn new() -> Self {
        let banned_token_store = HashMapBannedTokenStore::new();
        let user_store = HashmapUserStore::new();
        let user_store_id = user_store.get_id();
        let two_factor_code_store = HashMapTwoFACodeStore::new();
        let email_client = MockEmailClient;
        let app_state = Arc::new(AppState::new(
            banned_token_store,
            user_store,
            two_factor_code_store,
            email_client,
        ));
        let address = String::from(test::APP_GRPC_ADDRESS);

        let grpc_app = GRPCApp::new(app_state.clone(), address.clone())
            .await
            .expect("[ERROR][GRPCTestApp][new] Failed to create GRPCApp");
        let address = grpc_app.address;
        println!(
            "[GRPCTestApp][new] Bound to address: {address} with UserStore id: {user_store_id}"
        );

        tokio::spawn(grpc_app.run());

        sleep(Duration::from_millis(100)).await;

        let address = format!("http://{address}");
        let client = AuthServiceClient::connect(address.clone())
            .await
            .expect("[ERROR][GRPCTestApp][new] Failed to create gRPC client");

        Self {
            address,
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
    let signup_body = json!({
        "email": "test@example.com",
        "password": "P@ssw0rd",
        "requires2FA": false,
    });
    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status(), 201);
    let login_body = json!({
        "email": "test@example.com",
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

pub fn print_app_state<T: BannedTokenStore, U: UserStore, V: TwoFACodeStore, W: EmailClient>(
    app_state: &AppState<T, U, V, W>,
    prefix: &str,
) {
    println!("\n------------ AppState ------------");
    println!("{prefix} {:?}", app_state.banned_token_store);
    println!("{prefix} {:?}", app_state.user_store);
    println!("{prefix} {:?}", app_state.two_fa_code_store);
    println!("----------------------------------\n");
}
