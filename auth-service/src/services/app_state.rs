use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::data_stores::{BannedTokenStore, TwoFACodeStore, UserStore};

#[derive(Clone)]
pub struct AppState<T: BannedTokenStore, U: UserStore, V: TwoFACodeStore> {
    pub banned_token_store: Arc<RwLock<T>>,
    pub user_store: Arc<RwLock<U>>,
    pub two_fa_code_store: Arc<RwLock<V>>,
}

impl<T: BannedTokenStore, U: UserStore, V: TwoFACodeStore> AppState<T, U, V> {
    pub fn new(banned_token_store: T, user_store: U, two_factor_code_store: V) -> Self {
        Self {
            banned_token_store: Arc::new(RwLock::new(banned_token_store)),
            user_store: Arc::new(RwLock::new(user_store)),
            two_fa_code_store: Arc::new(RwLock::new(two_factor_code_store)),
        }
    }

    pub fn new_arc(banned_token_store: T, user_store: U, two_factor_code_store: V) -> Arc<Self> {
        Arc::new(Self::new(
            banned_token_store,
            user_store,
            two_factor_code_store,
        ))
    }
}
