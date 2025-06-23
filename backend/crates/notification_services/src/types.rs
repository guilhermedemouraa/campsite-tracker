use chrono::{DateTime, Utc};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Types for notifications (email and SMS).
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    /// Simple email service (SES) errors.
    #[error("AWS SES error: {0}")]
    SesError(String),

    /// Simple notification service (SNS) errors.
    #[error("AWS SNS error: {0}")]
    SnsError(String),

    /// Invalid phone number format.
    #[error("Invalid phone number format")]
    InvalidPhoneNumber,

    /// Invalid email format.
    #[error("Invalid email format")]
    InvalidEmail,
}

/// Represents a verification code for user actions like phone number or email verification.
#[derive(Clone)]
pub struct VerificationCode {
    /// The verification code itself, a 6-digit number.
    pub code: String,
    /// The expiration time of the verification code.
    pub expires_at: DateTime<Utc>,
    /// The number of attempts made to verify this code.
    pub attempts: u32,
}

/// Request structure for sending email verification
#[derive(serde::Deserialize)]
pub struct EmailVerificationQuery {
    /// Email address to which the verification link will be sent
    pub token: String,
}

/// Request structure for listing users
#[derive(serde::Deserialize)]
pub struct DeleteUserQuery {
    /// Email address of the user to be deleted
    pub email: String,
}

/// HTML template for email verification success
pub const EMAIL_VERIFICATION_SUCCESS_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Email Verified - CampTracker</title>
    <style>
        body { font-family: Arial, sans-serif; text-align: center; padding: 50px; background: #f0f9ff; }
        .container { max-width: 500px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }
        .success { color: #059669; font-size: 48px; margin-bottom: 20px; }
        h1 { color: #2c3e50; }
        .button { background: #4a6741; color: white; padding: 12px 24px; text-decoration: none; border-radius: 8px; display: inline-block; margin-top: 20px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="success">&#x2705;</div>
        <h1>Email Verified Successfully!</h1>
        <p>Your email has been verified. You can now receive campsite availability notifications.</p>
        <a href="/" class="button">Return to CampTracker</a>
    </div>
</body>
</html>
"#;

/// HTML template for email verification error
pub const EMAIL_VERIFICATION_ERROR_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Verification Error - CampTracker</title>
    <style>
        body { font-family: Arial, sans-serif; text-align: center; padding: 50px; background: #fef2f2; }
        .container { max-width: 500px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }
        .error { color: #dc2626; font-size: 48px; margin-bottom: 20px; }
        h1 { color: #2c3e50; }
        .button { background: #4a6741; color: white; padding: 12px 24px; text-decoration: none; border-radius: 8px; display: inline-block; margin-top: 20px; }
    </style>
</head>
<body>
    <div class="container">
        <div class="error">&#x274C;</div>
        <h1>Verification Link Invalid</h1>
        <p>This verification link has expired or is invalid. Please request a new verification email from your profile.</p>
        <a href="/" class="button">Return to CampTracker</a>
    </div>
</body>
</html>
"#;

/// A thread-safe store for verification codes, allowing concurrent access.
pub type VerificationStore = Arc<Mutex<HashMap<String, VerificationCode>>>;
