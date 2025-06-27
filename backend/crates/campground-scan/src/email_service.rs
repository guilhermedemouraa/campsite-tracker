use std::env;

use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use tracing::info;

use crate::{EmailService, NotificationError};

/// AWS SES email service implementation
pub struct AwsSesEmailService {
    client: Client,
    from_email: String,
    aws_region: String,
    aws_access_key: String,
    aws_secret_key: String,
}

#[derive(Debug, Serialize)]
struct SesRequest {
    #[serde(rename = "Source")]
    source: String,
    #[serde(rename = "Destination")]
    destination: SesDestination,
    #[serde(rename = "Message")]
    message: SesMessage,
}

#[derive(Debug, Serialize)]
struct SesDestination {
    #[serde(rename = "ToAddresses")]
    to_addresses: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SesMessage {
    #[serde(rename = "Subject")]
    subject: SesContent,
    #[serde(rename = "Body")]
    body: SesBody,
}

#[derive(Debug, Serialize)]
struct SesContent {
    #[serde(rename = "Data")]
    data: String,
    #[serde(rename = "Charset")]
    charset: String,
}

#[derive(Debug, Serialize)]
struct SesBody {
    #[serde(rename = "Text")]
    text: SesContent,
}

impl AwsSesEmailService {
    /// Create a new AWS SES email service
    pub fn new() -> Result<Self, NotificationError> {
        let from_email = env::var("FROM_EMAIL").map_err(|_| {
            NotificationError::Email("FROM_EMAIL environment variable not set".to_string())
        })?;

        let aws_region = env::var("AWS_REGION").map_err(|_| {
            NotificationError::Email("AWS_REGION environment variable not set".to_string())
        })?;

        let aws_access_key = env::var("AWS_ACCESS_KEY_ID").map_err(|_| {
            NotificationError::Email("AWS_ACCESS_KEY_ID environment variable not set".to_string())
        })?;

        let aws_secret_key = env::var("AWS_SECRET_ACCESS_KEY").map_err(|_| {
            NotificationError::Email(
                "AWS_SECRET_ACCESS_KEY environment variable not set".to_string(),
            )
        })?;

        let client = Client::new();

        Ok(Self {
            client,
            from_email,
            aws_region,
            aws_access_key,
            aws_secret_key,
        })
    }
}

#[async_trait]
impl EmailService for AwsSesEmailService {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<String, NotificationError> {
        info!("Sending email to {} with subject: {}", to, subject);

        // For now, just log the email and return a mock ID
        // In production, you would implement actual SES integration
        info!(
            "Email content:\nTo: {}\nSubject: {}\nBody: {}",
            to, subject, body
        );

        // Mock successful send
        let mock_id = format!("mock-email-{}", uuid::Uuid::new_v4());

        Ok(mock_id)
    }
}

/// Mock email service for development/testing
pub struct MockEmailService;

#[async_trait]
impl EmailService for MockEmailService {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<String, NotificationError> {
        info!("📧 [MOCK EMAIL] To: {}", to);
        info!("📧 [MOCK EMAIL] Subject: {}", subject);
        info!("📧 [MOCK EMAIL] Body:\n{}", body);

        let mock_id = format!("mock-email-{}", uuid::Uuid::new_v4());
        Ok(mock_id)
    }
}
