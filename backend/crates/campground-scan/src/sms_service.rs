use std::env;

use async_trait::async_trait;
use reqwest::Client;
use tracing::info;

use crate::{NotificationError, SmsService};

/// AWS SNS SMS service implementation
pub struct AwsSnsService {
    client: Client,
    aws_region: String,
    aws_access_key: String,
    aws_secret_key: String,
}

impl AwsSnsService {
    /// Create a new AWS SNS SMS service
    pub fn new() -> Result<Self, NotificationError> {
        let aws_region = env::var("AWS_REGION").map_err(|_| {
            NotificationError::Sms("AWS_REGION environment variable not set".to_string())
        })?;

        let aws_access_key = env::var("AWS_ACCESS_KEY_ID").map_err(|_| {
            NotificationError::Sms("AWS_ACCESS_KEY_ID environment variable not set".to_string())
        })?;

        let aws_secret_key = env::var("AWS_SECRET_ACCESS_KEY").map_err(|_| {
            NotificationError::Sms("AWS_SECRET_ACCESS_KEY environment variable not set".to_string())
        })?;

        let client = Client::new();

        Ok(Self {
            client,
            aws_region,
            aws_access_key,
            aws_secret_key,
        })
    }
}

#[async_trait]
impl SmsService for AwsSnsService {
    async fn send_sms(&self, to: &str, message: &str) -> Result<String, NotificationError> {
        info!("Sending SMS to {} with message: {}", to, message);

        // For now, just log the SMS and return a mock ID
        // In production, you would implement actual SNS integration
        info!("SMS content:\nTo: {}\nMessage: {}", to, message);

        // Mock successful send
        let mock_id = format!("mock-sms-{}", uuid::Uuid::new_v4());

        Ok(mock_id)
    }
}

/// Mock SMS service for development/testing
pub struct MockSmsService;

#[async_trait]
impl SmsService for MockSmsService {
    async fn send_sms(&self, to: &str, message: &str) -> Result<String, NotificationError> {
        info!("📱 [MOCK SMS] To: {}", to);
        info!("📱 [MOCK SMS] Message: {}", message);

        let mock_id = format!("mock-sms-{}", uuid::Uuid::new_v4());
        Ok(mock_id)
    }
}
