use color_eyre::eyre::Result;
use reqwest::{Client, Url};
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

use crate::{
    domain::{
        data_stores::TwoFACode,
        email::Email,
        email_client::{EmailClient, TemplateModel},
    },
    utils::{
        auth::PasswordResetToken,
        constants::{Time, REST_AUTH_SERVICE_URL},
    },
};

const MESSAGE_STREAM: &str = "outbound";
const POSTMARK_AUTH_HEADER: &str = "X-Postmark-Server-Token";

#[derive(Debug, Clone)]
pub struct PostmarkEmailClient {
    http_client: Client,
    base_url: String,
    sender: Email,
    authorization_token: Secret<String>,
}

impl PostmarkEmailClient {
    pub fn new(base_url: String, sender: Email, authorization_token: Secret<String>, http_client: Client) -> Self {
        Self {
            base_url,
            sender,
            authorization_token,
            http_client,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub enum PostmarkTemplate {
    PasswordReset(Time, PasswordResetToken),
    TwoFACode(Time, TwoFACode),
}

impl PostmarkTemplate {
    fn to_model(&self) -> TemplateModel {
        match self {
            Self::PasswordReset(time, token) => {
                let auth_base_url = REST_AUTH_SERVICE_URL.to_string();
                let url = format!("{auth_base_url}/reset-password?token={}", token.expose_secret_string());
                TemplateModel::new(time.to_string(), url)
            }
            Self::TwoFACode(time, two_fa_code) => {
                let model_content = two_fa_code.expose_secret_string();
                TemplateModel::new(time.to_string(), model_content)
            }
        }
    }

    fn alias(&self) -> &str {
        match self {
            Self::PasswordReset(_, _) => "password-reset",
            Self::TwoFACode(_, _) => "two-fa-code",
        }
    }
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    message_stream: &'a str,
    template_alias: &'a str,
    template_model: TemplateModel,
}

#[async_trait::async_trait]
impl EmailClient for PostmarkEmailClient {
    #[tracing::instrument(name = "Sending email", skip_all)]
    async fn send_email(&self, recipient: &Email, subject: &str, template: PostmarkTemplate) -> Result<()> {
        let base = Url::parse(&self.base_url)?;
        let url = base.join("/email")?;

        let request_body = SendEmailRequest {
            from: self.sender.as_ref().expose_secret(),
            to: recipient.as_ref().expose_secret(),
            subject,
            message_stream: MESSAGE_STREAM,
            template_alias: template.alias(),
            template_model: template.to_model(),
        };

        let request = self
            .http_client
            .post(url)
            .header(POSTMARK_AUTH_HEADER, self.authorization_token.expose_secret())
            .json(&request_body);

        request.send().await?.error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::constants::test;

    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Sentence;
    use fake::{Fake, Faker};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    use super::PostmarkEmailClient;

    // Helper function to generate a test subject
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    // Helper function to generate test content
    fn template() -> PostmarkTemplate {
        PostmarkTemplate::TwoFACode(Time::Minutes10, TwoFACode::default())
    }

    // Helper function to generate a test email
    fn email() -> Email {
        Email::parse(Secret::new(SafeEmail().fake())).unwrap()
    }

    // Helper function to create a test email client
    fn email_client(base_url: String) -> PostmarkEmailClient {
        println!("[email_client] base_url: {base_url}");
        let http_client = Client::builder().timeout(test::email_client::TIMEOUT).build().unwrap();
        let client = PostmarkEmailClient::new(base_url, email(), Secret::new(Faker.fake()), http_client);
        println!("[email_client] Email Client: {:?}", client);
        client
    }

    // Custom matcher to validate the email request body
    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("TemplateAlias").is_some()
                    && body.get("MessageStream").is_some()
                    && body.get("TemplateModel").is_some()
            } else {
                false
            }
        }
    }

    // Test to ensure the email client sends the expected request
    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // Set up the mock server to expect a specific request
        Mock::given(header_exists(POSTMARK_AUTH_HEADER))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        println!(
            "[send_email_sends_the_expected_request] Mock Server:  {:?}",
            mock_server
        );
        println!(
            "[send_email_sends_the_expected_request] Email Client: {:?}",
            email_client
        );

        // Execute the send_email function and check the outcome
        let outcome = email_client.send_email(&email(), &subject(), template()).await;
        println!("[send_email_sends_the_expected_request] Outcome: {:?}", outcome);
        assert!(outcome.is_ok());
    }

    // Test to handle server error responses
    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // Set up the mock server to respond with a 500 error
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Execute the send_email function and check the outcome
        let outcome = email_client.send_email(&email(), &subject(), template()).await;

        assert!(outcome.is_err());
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client.send_email(&email(), &subject(), template()).await;

        assert!(outcome.is_err());
    }
}
