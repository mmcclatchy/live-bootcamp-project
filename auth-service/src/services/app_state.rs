use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::data_stores::UserStore;

#[derive(Clone)]
pub struct AppState<T: UserStore> {
    pub user_store: Arc<RwLock<T>>,
}

impl<T: UserStore> AppState<T> {
    pub fn new(user_store: T) -> Self {
        Self {
            user_store: Arc::new(RwLock::new(user_store)),
        }
    }

    pub fn new_arc(user_store: T) -> Arc<Self> {
        Arc::new(Self::new(user_store))
    }
}
