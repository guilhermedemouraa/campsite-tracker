use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Request structure for user sign-up
#[derive(Debug, Deserialize, Validate)]
pub struct SignUpRequest {
    /// Name of the user
    #[validate(length(min = 1, max = 255, message = "Name is required"))]
    pub name: String,

    /// Email address of the user
    #[validate(email(message = "Please enter a valid email"))]
    pub email: String,

    /// Phone number of the user
    #[validate(length(
        min = 10,
        max = 15,
        message = "Phone number must be between 10-15 digits"
    ))]
    pub phone: String,

    /// Password for the user account
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    /// Notification preferences for the user
    pub notification_preferences: NotificationPreferences,
}

/// Preferences for user notifications
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NotificationPreferences {
    /// Whether the user wants to receive email notifications
    pub email: bool,
    /// Whether the user wants to receive SMS notifications
    pub sms: bool,
}

/// Request structure for verifying email
#[derive(serde::Deserialize)]
pub struct VerifyEmailRequest {
    /// Verification code sent to the user's email
    pub code: String,
}

/// Request structure for verifying phone number
#[derive(serde::Deserialize)]
pub struct VerifyPhoneRequest {
    /// Verification code sent to the user's phone
    pub code: String,
}

/// Response structure for verification actions
#[derive(serde::Serialize)]
pub struct VerificationResponse {
    /// Message indicating the result of the verification
    pub message: String,
}

/// Response structure for user sign-up
#[derive(Debug, Serialize)]
pub struct SignUpResponse {
    /// Unique identifier for the user
    pub id: Uuid,
    /// Name of the user
    pub name: String,
    /// Email address of the user
    pub email: String,
    /// Phone number of the user
    pub phone: String,
    /// Whether the user's email is verified
    pub email_verified: bool,
    /// Whether the user's phone number is verified
    pub phone_verified: bool,
    /// User's notification preferences
    pub notification_preferences: NotificationPreferences,
    /// Time at which the user was created
    pub created_at: DateTime<Utc>,
}

/// Response structure for user authentication
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    /// Access token for the user
    pub access_token: String,
    /// Refresh token for the user
    pub refresh_token: String,
    /// User information
    pub user: UserInfo,
}

/// Information about the user, used in responses
#[derive(Debug, Serialize)]
pub struct UserInfo {
    /// Unique identifier for the user
    pub id: Uuid,
    /// Name of the user
    pub name: String,
    /// Email address of the user
    pub email: String,
    /// Phone number of the user
    pub phone: String,
    /// Whether the user's email is verified
    pub email_verified: bool,
    /// Whether the user's phone number is verified
    pub phone_verified: bool,
    /// User's notification preferences
    pub notification_preferences: NotificationPreferences,
}

/// Request structure for updating user profile
#[derive(serde::Deserialize, validator::Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, max = 255, message = "Name is required"))]
    /// Name of the user
    pub name: String,

    /// Email address of the user
    #[validate(email(message = "Please enter a valid email"))]
    pub email: String,

    /// Phone number of the user
    #[validate(length(
        min = 10,
        max = 15,
        message = "Phone number must be between 10-15 digits"
    ))]
    pub phone: String,

    /// Notification preferences for the user
    pub notification_preferences: NotificationPreferences,
}

/// Request structure for user login
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    /// Email address of the user
    #[validate(email(message = "Please enter a valid email"))]
    pub email: String,

    /// Password for the user account
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// User model representing the database schema
#[derive(Debug, sqlx::FromRow)]
pub struct User {
    /// Unique identifier for the user
    pub id: Uuid,
    /// Email address of the user
    pub email: String,
    /// Name of the user
    pub name: String,
    /// Phone number of the user (nullable)
    pub phone: Option<String>, // This is nullable in the DB
    /// Hashed password of the user
    pub password_hash: String,
    /// Role of the user (e.g., "user", "admin")
    pub role: String,
    /// Whether the user's email is verified
    pub email_verified: bool,
    /// Whether the user's phone number is verified
    pub phone_verified: bool,
    /// User's notification preferences stored as JSON
    pub notification_preferences: serde_json::Value,
    /// Timezone of the user
    pub timezone: String,
    /// Whether the user account is active
    pub is_active: bool,
    /// Timestamp when the user was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the user was last updated
    pub updated_at: DateTime<Utc>,
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject of the token, typically the user ID
    pub sub: String, // user ID
    /// Email address of the user
    pub email: String,
    /// Role of the user (e.g., "user", "admin")
    pub role: String,
    /// Expiration timestamp of the token
    pub exp: usize, // expiration timestamp
    /// Issued at timestamp of the token
    pub iat: usize, // issued at timestamp
}

/// Custom error type for authentication-related errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// The email address already exists in the system
    #[error("Email already exists")]
    EmailExists,

    /// The provided credentials are invalid
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// The phone number format is invalid
    #[error("Invalid phone number format")]
    InvalidPhoneNumber,

    /// The user was not found in the system
    #[error("User not found")]
    UserNotFound,

    /// An internal server error occurred
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// An error occurred while hashing the password
    #[error("Password hashing error: {0}")]
    PasswordHash(#[from] bcrypt::BcryptError),

    /// An error occurred while serializing or deserializing JSON
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    /// An error occurred while validating input data
    #[error("Validation error: {0}")]
    Validation(String),
}

impl actix_web::ResponseError for AuthError {
    fn error_response(&self) -> actix_web::HttpResponse {
        use actix_web::HttpResponse;

        match self {
            AuthError::EmailExists => HttpResponse::Conflict().json(serde_json::json!({
                "error": "email_exists",
                "message": "An account with this email already exists"
            })),
            AuthError::InvalidCredentials => HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "invalid_credentials",
                "message": "Invalid email or password"
            })),
            AuthError::UserNotFound => HttpResponse::NotFound().json(serde_json::json!({
                "error": "user_not_found",
                "message": "User not found"
            })),
            AuthError::Validation(msg) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "validation_error",
                "message": msg
            })),
            AuthError::InvalidPhoneNumber => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_phone_number",
                "message": "Please enter a valid US phone number"
            })),
            _ => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "An internal error occurred"
            })),
        }
    }
}

/// Validates a US phone number format
pub fn validate_phone_number(phone: &str) -> bool {
    // Remove all non-digit characters
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

    // US phone numbers should be 10 digits, or 11 if they include the country code (1)
    match digits.len() {
        10 => true,
        11 => digits.starts_with('1'),
        _ => false,
    }
}
