use std::sync::Arc;

use super::{
    app_state::{AppServices, AppState},
    data_stores::postgres_user_store::PostgresUserStore,
    hashmap_banned_token_store::HashMapBannedTokenStore,
    hashmap_password_reset_token_store::HashMapPasswordResetTokenStore,
    hashmap_two_fa_code_store::HashMapTwoFACodeStore,
    hashmap_user_store::HashmapUserStore,
    mock_email_client::MockEmailClient,
};

pub struct MemoryServices;

impl AppServices for MemoryServices {
    type BannedTokenStore = HashMapBannedTokenStore;
    type UserStore = HashmapUserStore;
    type TwoFACodeStore = HashMapTwoFACodeStore;
    type PasswordResetTokenStore = HashMapPasswordResetTokenStore;
    type EmailClient = MockEmailClient;
}

#[derive(Debug)]
pub struct PersistentServices;

impl AppServices for PersistentServices {
    type BannedTokenStore = HashMapBannedTokenStore;
    type UserStore = PostgresUserStore;
    type TwoFACodeStore = HashMapTwoFACodeStore;
    type PasswordResetTokenStore = HashMapPasswordResetTokenStore;
    type EmailClient = MockEmailClient;
}

pub type MemoryAppStateType = Arc<AppState<MemoryServices>>;
pub type PersistentAppStateType = Arc<AppState<PersistentServices>>;
