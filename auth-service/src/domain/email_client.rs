use core::fmt::Debug;

use color_eyre::eyre::Result;
use serde::Serialize;

use crate::{services::postmark_email_client::PostmarkTemplate, utils::constants::REST_AUTH_SERVICE_URL};

use super::email::Email;

#[derive(Serialize, Clone, Debug)]
pub struct TemplateModel {
    product_url: &'static str,
    product_name: &'static str,
    expiration_time: String,
    model_content: String,
}

impl TemplateModel {
    pub fn new(expiration_time: String, model_content: String) -> Self {
        Self {
            product_url: &REST_AUTH_SERVICE_URL,
            product_name: "markmcclatchy.com Auth Service",
            expiration_time,
            model_content,
        }
    }
}

// pub trait EmailTemplate: Clone + Debug + Serialize {
//     fn to_model(&self) -> TemplateModel;
//     fn alias(&self) -> &str;
// }

#[async_trait::async_trait]
pub trait EmailClient: Clone + Send + Sync + Debug + 'static {
    async fn send_email(&self, recipient: &Email, template: PostmarkTemplate) -> Result<()>;
}
