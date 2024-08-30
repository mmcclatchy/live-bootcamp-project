use color_eyre::eyre::Result;
use secrecy::ExposeSecret;
use serde_json::json;

use crate::domain::{email::Email, email_client::EmailClient};

use super::postmark_email_client::PostmarkTemplate;

#[derive(Clone, Debug)]
pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(&self, recipient: &Email, subject: &str, template: PostmarkTemplate) -> Result<()> {
        println!(
            "Sending email to {} with subject: {subject} and content {}",
            recipient.as_ref().expose_secret(),
            json!(template)
        );

        Ok(())
    }
}
