use core::fmt::Debug;

use super::email::Email;

#[async_trait::async_trait]
pub trait EmailClient: Clone + Send + Sync + Debug + 'static {
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> Result<(), String>;
}
