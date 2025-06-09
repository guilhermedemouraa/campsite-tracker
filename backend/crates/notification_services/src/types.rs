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

/// A thread-safe store for verification codes, allowing concurrent access.
pub type VerificationStore = Arc<Mutex<HashMap<String, VerificationCode>>>;
