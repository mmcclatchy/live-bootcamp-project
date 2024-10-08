use std::{fmt, sync::Arc};

use tokio::sync::RwLock;

use crate::domain::{
    data_stores::{BannedTokenStore, PasswordResetTokenStore, TwoFACodeStore, UserStore},
    email_client::EmailClient,
};

pub trait AppServices: fmt::Debug {
    type BannedTokenStore: BannedTokenStore + fmt::Debug + 'static;
    type UserStore: UserStore + fmt::Debug + 'static;
    type TwoFACodeStore: TwoFACodeStore + fmt::Debug + 'static;
    type PasswordResetTokenStore: PasswordResetTokenStore + fmt::Debug + 'static;
    type EmailClient: EmailClient + fmt::Debug + 'static;
}

#[derive(Clone, Debug)]
pub struct AppState<S: AppServices> {
    pub banned_token_store: Arc<RwLock<S::BannedTokenStore>>,
    pub user_store: Arc<RwLock<S::UserStore>>,
    pub two_fa_code_store: Arc<RwLock<S::TwoFACodeStore>>,
    pub email_client: Arc<S::EmailClient>,
    pub password_reset_token_store: Arc<RwLock<S::PasswordResetTokenStore>>,
}

impl<S: AppServices> AppState<S> {
    pub fn new(
        banned_token_store: S::BannedTokenStore,
        user_store: S::UserStore,
        two_factor_code_store: S::TwoFACodeStore,
        email_client: S::EmailClient,
        password_reset_token_store: S::PasswordResetTokenStore,
    ) -> Self {
        Self {
            banned_token_store: Arc::new(RwLock::new(banned_token_store)),
            user_store: Arc::new(RwLock::new(user_store)),
            two_fa_code_store: Arc::new(RwLock::new(two_factor_code_store)),
            email_client: Arc::new(email_client),
            password_reset_token_store: Arc::new(RwLock::new(password_reset_token_store)),
        }
    }

    pub fn new_arc(
        banned_token_store: S::BannedTokenStore,
        user_store: S::UserStore,
        two_factor_code_store: S::TwoFACodeStore,
        email_client: S::EmailClient,
        password_reset_token_store: S::PasswordResetTokenStore,
    ) -> Arc<Self> {
        Arc::new(Self::new(
            banned_token_store,
            user_store,
            two_factor_code_store,
            email_client,
            password_reset_token_store,
        ))
    }
}
