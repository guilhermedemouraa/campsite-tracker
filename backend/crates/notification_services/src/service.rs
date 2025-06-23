use crate::types::*;
use aws_config::BehaviorVersion;
use aws_sdk_ses::Client as SesClient;
use aws_sdk_sns::Client as SnsClient;
use chrono::{Duration, Utc};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

/// Notification service for sending emails and SMS messages.
#[derive(Debug, Clone)]
pub struct NotificationService {
    ses_client: SesClient,
    sns_client: SnsClient,
    from_email: String,
}

impl NotificationService {
    /// Creates a new instance of the NotificationService with AWS clients initialized.
    pub async fn new() -> Result<Self, NotificationError> {
        let config = aws_config::defaults(BehaviorVersion::latest()).load().await;

        let ses_client = SesClient::new(&config);
        let sns_client = SnsClient::new(&config);

        let from_email = std::env::var("FROM_EMAIL")
            .unwrap_or_else(|_| "noreplycampsitetracker@gmail.com".to_string());

        Ok(Self {
            ses_client,
            sns_client,
            from_email,
        })
    }

    /// Sends an email verification LINK to the user (NEW FUNCTION)
    pub async fn send_email_verification_link(
        &self,
        user_id: &Uuid,
        email: &str,
        name: &str,
        verification_token: &str,
    ) -> Result<(), NotificationError> {
        log::info!(
            "📧 Sending verification link to {} for user {}",
            email,
            user_id
        );

        // Build the verification URL
        let verification_url = format!(
            "http://localhost:8080/verify-email?token={}",
            verification_token
        );

        let subject = "Verify your CampTracker email";
        let html_body = format!(
            r#"
            <html>
            <body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto;">
                <div style="background: linear-gradient(135deg, #2c3e50 0%, #4a6741 100%); padding: 20px; text-align: center;">
                    <h1 style="color: white; margin: 0;">🏕️ CampTracker</h1>
                </div>
                <div style="padding: 30px; background: white;">
                    <h2 style="color: #2c3e50;">Hi {}!</h2>
                    <p style="font-size: 16px; line-height: 1.6; color: #374151;">
                        Welcome to CampTracker! Please verify your email address to complete your account setup.
                    </p>
                    <div style="text-align: center; margin: 30px 0;">
                        <a href="{}" style="
                            display: inline-block;
                            background: #4a6741;
                            color: white;
                            text-decoration: none;
                            padding: 12px 24px;
                            border-radius: 8px;
                            font-weight: bold;
                            font-size: 16px;
                        ">Verify Email Address</a>
                    </div>
                    <p style="font-size: 14px; color: #6b7280;">
                        This link will expire in 24 hours. If you didn't create this account, you can safely ignore this email.
                    </p>
                </div>
                <div style="background: #f9fafb; padding: 20px; text-align: center; color: #6b7280; font-size: 12px;">
                    <p>© 2025 CampTracker. Never miss a campsite!</p>
                </div>
            </body>
            </html>
            "#,
            name, verification_url
        );

        let text_body = format!(
            "Hi {}!\n\nWelcome to CampTracker!\n\nPlease verify your email by visiting this link:\n{}\n\nThis link will expire in 24 hours.\n\n© 2025 CampTracker",
            name, verification_url
        );

        // Rest is the same as your existing send_email_verification function
        let subject_content = aws_sdk_ses::types::Content::builder()
            .data(subject)
            .build()
            .map_err(|e| {
                log::error!("❌ Failed to build subject content: {}", e);
                NotificationError::SesError(format!("Failed to build subject: {}", e))
            })?;

        let html_content = aws_sdk_ses::types::Content::builder()
            .data(html_body)
            .build()
            .map_err(|e| {
                log::error!("❌ Failed to build HTML content: {}", e);
                NotificationError::SesError(format!("Failed to build HTML body: {}", e))
            })?;

        let text_content = aws_sdk_ses::types::Content::builder()
            .data(text_body)
            .build()
            .map_err(|e| {
                log::error!("❌ Failed to build text content: {}", e);
                NotificationError::SesError(format!("Failed to build text body: {}", e))
            })?;

        let body = aws_sdk_ses::types::Body::builder()
            .html(html_content)
            .text(text_content)
            .build();

        let message = aws_sdk_ses::types::Message::builder()
            .subject(subject_content)
            .body(body)
            .build();

        let destination = aws_sdk_ses::types::Destination::builder()
            .to_addresses(email)
            .build();

        log::info!("📧 Sending email via AWS SES...");

        let result = self
            .ses_client
            .send_email()
            .source(&self.from_email)
            .destination(destination)
            .message(message)
            .send()
            .await;

        match result {
            Ok(output) => {
                log::info!(
                    "✅ Email verification link sent to {} for user {}",
                    email,
                    user_id
                );
                let message_id = output.message_id();
                log::info!("📧 SES Message ID: {}", message_id);
                Ok(())
            }
            Err(e) => {
                log::error!("❌ AWS SES error: {:#?}", e);
                let error_msg = if let Some(service_error) = e.as_service_error() {
                    format!("AWS SES service error: {:?}", service_error)
                } else {
                    format!("AWS SES error: {}", e)
                };
                Err(NotificationError::SesError(error_msg))
            }
        }
    }

    /// Sends an SMS verification message to the user.
    pub async fn send_sms_verification(
        &self,
        user_id: &Uuid,
        phone: &str,
        verification_code: &str,
    ) -> Result<(), NotificationError> {
        // Ensure phone number is in E.164 format
        let formatted_phone = if phone.starts_with('+') {
            phone.to_string()
        } else {
            format!("+{}", phone.replace(['(', ')', '-', ' ', '.'], ""))
        };

        let message = format!(
            "Your CampTracker verification code is: {}\n\nThis code expires in 10 minutes.\n\nIf you didn't request this, ignore this message.",
            verification_code
        );

        self.sns_client
            .publish()
            .phone_number(&formatted_phone)
            .message(&message)
            .send()
            .await
            .map_err(|e| NotificationError::SnsError(e.to_string()))?;

        log::info!(
            "SMS verification sent to {} for user {}",
            formatted_phone,
            user_id
        );
        Ok(())
    }

    /// Generates a random 6-digit verification code (for SMS).
    pub fn generate_verification_code() -> String {
        use rand::Rng;
        let mut rng = rand::rng();
        format!("{:06}", rng.random_range(100000..=999999))
    }

    /// Generate secure token for email links
    pub fn generate_verification_token() -> String {
        use rand::Rng;
        let mut rng = rand::rng();
        (0..32)
            .map(|_| {
                let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
                chars[rng.random_range(0..chars.len())] as char
            })
            .collect()
    }
}

/// A thread-safe store for verification codes, allowing concurrent access.
pub fn create_verification_store() -> VerificationStore {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Represents a verification code for user actions like phone number or email verification.
pub fn store_verification_code(
    store: &VerificationStore,
    key: &str,
    code: &str,
    expires_in_minutes: i64,
) {
    let verification = VerificationCode {
        code: code.to_string(),
        expires_at: Utc::now() + Duration::minutes(expires_in_minutes),
        attempts: 0,
    };

    store.lock().unwrap().insert(key.to_string(), verification);
}

/// Verifies the provided code against the stored verification code.
pub fn verify_code(
    store: &VerificationStore,
    key: &str,
    provided_code: &str,
) -> Result<bool, String> {
    let mut store = store.lock().unwrap();

    let verification = store.get_mut(key).ok_or("Verification code not found")?;

    if verification.expires_at < Utc::now() {
        store.remove(key);
        return Err("Verification code has expired".to_string());
    }

    verification.attempts += 1;

    if verification.attempts > 3 {
        store.remove(key);
        return Err("Too many verification attempts".to_string());
    }

    if verification.code == provided_code {
        store.remove(key);
        Ok(true)
    } else {
        Ok(false)
    }
}
