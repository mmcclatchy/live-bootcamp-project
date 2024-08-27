use secrecy::ExposeSecret;

use crate::domain::{email::Email, email_client::EmailClient};

#[derive(Clone, Debug)]
pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> Result<(), String> {
        println!(
            "Sending email to {} with subject: {subject} and content {content}",
            recipient.as_ref().expose_secret()
        );

        Ok(())
    }
}
