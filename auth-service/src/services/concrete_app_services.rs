use std::sync::Arc;

use super::{
    app_state::{AppServices, AppState},
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

pub type MemoryAppStateType = Arc<AppState<MemoryServices>>;
