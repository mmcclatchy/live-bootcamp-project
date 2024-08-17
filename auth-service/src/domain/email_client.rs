use core::fmt;

use super::email::Email;

#[async_trait::async_trait]
pub trait EmailClient: Clone + Send + Sync + 'static + fmt::Debug {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), String>;
}
