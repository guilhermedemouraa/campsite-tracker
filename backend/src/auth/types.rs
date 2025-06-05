// src/auth/types.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct SignUpRequest {
    #[validate(length(min = 1, max = 255, message = "Name is required"))]
    pub name: String,

    #[validate(email(message = "Please enter a valid email"))]
    pub email: String,

    #[validate(length(
        min = 10,
        max = 15,
        message = "Phone number must be between 10-15 digits"
    ))]
    pub phone: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,

    pub notification_preferences: NotificationPreferences,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NotificationPreferences {
    pub email: bool,
    pub sms: bool,
}

#[derive(Debug, Serialize)]
pub struct SignUpResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub notification_preferences: NotificationPreferences,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub notification_preferences: NotificationPreferences,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Please enter a valid email"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

// Database model - matching the exact schema
#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub phone: Option<String>, // This is nullable in the DB
    pub password_hash: String,
    pub role: String,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub notification_preferences: serde_json::Value,
    pub timezone: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user ID
    pub email: String,
    pub role: String,
    pub exp: usize, // expiration timestamp
    pub iat: usize, // issued at timestamp
}

// Regex for phone validation
lazy_static::lazy_static! {
    pub static ref PHONE_REGEX: regex::Regex = regex::Regex::new(
        r"^\+?1?[-.\s]?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})$"
    ).unwrap();
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Email already exists")]
    EmailExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid phone number format")]
    InvalidPhoneNumber,

    #[error("User not found")]
    UserNotFound,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Password hashing error: {0}")]
    PasswordHash(#[from] bcrypt::BcryptError),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

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

// Phone validation utility
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
