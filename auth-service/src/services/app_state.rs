use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::{
    data_stores::{BannedTokenStore, TwoFACodeStore, UserStore},
    email_client::EmailClient,
};

#[derive(Clone, Debug)]
pub struct AppState<T: BannedTokenStore, U: UserStore, V: TwoFACodeStore, W: EmailClient> {
    pub banned_token_store: Arc<RwLock<T>>,
    pub user_store: Arc<RwLock<U>>,
    pub two_fa_code_store: Arc<RwLock<V>>,
    pub email_client: Arc<RwLock<W>>,
}

impl<T: BannedTokenStore, U: UserStore, V: TwoFACodeStore, W: EmailClient> AppState<T, U, V, W> {
    pub fn new(
        banned_token_store: T,
        user_store: U,
        two_factor_code_store: V,
        email_client: W,
    ) -> Self {
        Self {
            banned_token_store: Arc::new(RwLock::new(banned_token_store)),
            user_store: Arc::new(RwLock::new(user_store)),
            two_fa_code_store: Arc::new(RwLock::new(two_factor_code_store)),
            email_client: Arc::new(RwLock::new(email_client)),
        }
    }

    pub fn new_arc(
        banned_token_store: T,
        user_store: U,
        two_factor_code_store: V,
        email_client: W,
    ) -> Arc<Self> {
        Arc::new(Self::new(
            banned_token_store,
            user_store,
            two_factor_code_store,
            email_client,
        ))
    }
}
