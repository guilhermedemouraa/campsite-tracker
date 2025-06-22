use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::types::{
    AuthError, NotificationPreferences, SignUpRequest, UpdateProfileRequest, User,
    validate_phone_number,
};

/// A service for handling user authentication operations such as creating users,
/// retrieving user information, verifying credentials, and managing sessions.
pub struct AuthService {
    pool: PgPool,
}

impl AuthService {
    /// Creates a new instance of `AuthService` with the provided database connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new user in the database with the provided sign-up request.
    pub async fn create_user(&self, request: &SignUpRequest) -> Result<User, AuthError> {
        // Validate phone number format
        if !validate_phone_number(&request.phone) {
            return Err(AuthError::InvalidPhoneNumber);
        }

        // Check if email already exists
        let existing_user = sqlx::query("SELECT id FROM users WHERE email = $1")
            .bind(request.email.to_lowercase())
            .fetch_optional(&self.pool)
            .await?;

        if existing_user.is_some() {
            return Err(AuthError::EmailExists);
        }

        // Hash the password
        let password_hash = hash(&request.password, DEFAULT_COST)?;

        // Format phone number to E.164 format
        let formatted_phone = self.format_phone_number(&request.phone);

        // Serialize notification preferences to JSON
        let notification_prefs =
            serde_json::to_value(&request.notification_preferences).map_err(|e| {
                AuthError::Validation(format!("Invalid notification preferences: {}", e))
            })?;

        // Insert the new user
        let row = sqlx::query(
            r#"
            INSERT INTO users (
                email, name, phone, password_hash, notification_preferences
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING 
                id, email, name, phone, password_hash, role, 
                email_verified, phone_verified, notification_preferences,
                timezone, is_active, created_at, updated_at
            "#,
        )
        .bind(request.email.to_lowercase().trim())
        .bind(request.name.trim())
        .bind(&formatted_phone)
        .bind(&password_hash)
        .bind(&notification_prefs)
        .fetch_one(&self.pool)
        .await?;

        let user = User {
            id: row.get("id"),
            email: row.get("email"),
            name: row.get("name"),
            phone: row.get("phone"),
            password_hash: row.get("password_hash"),
            role: row.get("role"),
            email_verified: row.get("email_verified"),
            phone_verified: row.get("phone_verified"),
            notification_preferences: row.get("notification_preferences"),
            timezone: row.get("timezone"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(user)
    }

    /// Retrieves a user by their email address, returning `None` if not found or inactive.
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, email, name, phone, password_hash, role,
                email_verified, phone_verified, notification_preferences,
                timezone, is_active, created_at, updated_at
            FROM users 
            WHERE email = $1 AND is_active = true
            "#,
        )
        .bind(email.to_lowercase())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let user = User {
                id: row.get("id"),
                email: row.get("email"),
                name: row.get("name"),
                phone: row.get("phone"),
                password_hash: row.get("password_hash"),
                role: row.get("role"),
                email_verified: row.get("email_verified"),
                phone_verified: row.get("phone_verified"),
                notification_preferences: row.get("notification_preferences"),
                timezone: row.get("timezone"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Retrieves a user by their ID, returning `None` if not found or inactive.
    /// This method is useful for fetching user details without exposing sensitive information.
    pub async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, email, name, phone, password_hash, role,
                email_verified, phone_verified, notification_preferences,
                timezone, is_active, created_at, updated_at
            FROM users 
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let user = User {
                id: row.get("id"),
                email: row.get("email"),
                name: row.get("name"),
                phone: row.get("phone"),
                password_hash: row.get("password_hash"),
                role: row.get("role"),
                email_verified: row.get("email_verified"),
                phone_verified: row.get("phone_verified"),
                notification_preferences: row.get("notification_preferences"),
                timezone: row.get("timezone"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// Verifies the user's password against the stored hash.
    pub async fn verify_password(&self, email: &str, password: &str) -> Result<User, AuthError> {
        let user = self
            .get_user_by_email(email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        let is_valid = verify(password, &user.password_hash)?;

        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        Ok(user)
    }

    /// Creates a new session for the user with a refresh token hash
    pub async fn create_session(
        &self,
        user_id: &Uuid,
        refresh_token_hash: &str,
    ) -> Result<Uuid, AuthError> {
        let row = sqlx::query(
            r#"
            INSERT INTO user_sessions (user_id, refresh_token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(refresh_token_hash)
        .bind(Utc::now() + chrono::Duration::days(30)) // 30 day expiry
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    /// Updates the user's email and/or phone verification status
    pub async fn update_user_verification(
        &self,
        user_id: &Uuid,
        email_verified: Option<bool>,
        phone_verified: Option<bool>,
    ) -> Result<(), AuthError> {
        if email_verified.is_some() {
            sqlx::query("UPDATE users SET email_verified = $1, updated_at = NOW() WHERE id = $2")
                .bind(email_verified.unwrap())
                .bind(user_id)
                .execute(&self.pool)
                .await?;
        }

        if phone_verified.is_some() {
            sqlx::query("UPDATE users SET phone_verified = $1, updated_at = NOW() WHERE id = $2")
                .bind(phone_verified.unwrap())
                .bind(user_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    /// Updates the user's profile information
    pub async fn update_user_profile(
        &self,
        user_id: &Uuid,
        request: &UpdateProfileRequest,
    ) -> Result<User, AuthError> {
        // Get current user to compare changes
        let current_user = self
            .get_user_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        // Check if email or phone changed
        let email_changed = current_user.email != request.email;
        let phone_changed = current_user.phone.as_deref() != Some(&request.phone);

        // Determine new verification status
        let new_email_verified = if email_changed {
            false
        } else {
            current_user.email_verified
        };
        let new_phone_verified = if phone_changed {
            false
        } else {
            current_user.phone_verified
        };

        // Serialize notification preferences to JSON and map error to AuthError
        let notification_prefs =
            serde_json::to_value(&request.notification_preferences).map_err(|e| {
                AuthError::Validation(format!("Invalid notification preferences: {}", e))
            })?;

        // Build the update query and manually construct the User
        let row = sqlx::query(
            r#"
            UPDATE users 
            SET name = $1, 
                email = $2, 
                phone = $3,
                email_verified = $4,
                phone_verified = $5,
                notification_preferences = $6,
                updated_at = NOW()
            WHERE id = $7
            RETURNING 
                id, email, name, phone, password_hash, role, 
                email_verified, phone_verified, notification_preferences,
                timezone, is_active, created_at, updated_at
            "#,
        )
        .bind(request.name.trim())
        .bind(request.email.to_lowercase().trim())
        .bind(&request.phone)
        .bind(new_email_verified)
        .bind(new_phone_verified)
        .bind(&notification_prefs)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let updated_user = User {
            id: row.get("id"),
            email: row.get("email"),
            name: row.get("name"),
            phone: row.get("phone"),
            password_hash: row.get("password_hash"),
            role: row.get("role"),
            email_verified: row.get("email_verified"),
            phone_verified: row.get("phone_verified"),
            notification_preferences: row.get("notification_preferences"),
            timezone: row.get("timezone"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

        Ok(updated_user)
    }

    fn format_phone_number(&self, phone: &str) -> String {
        // Remove all non-digit characters
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

        // Add +1 if it's a 10-digit US number
        if digits.len() == 10 {
            format!("+1{}", digits)
        } else {
            // For 11-digit numbers starting with 1, or any other format, just add +
            format!("+{}", digits)
        }
    }
}

impl User {
    /// Converts the user's notification preferences from JSON to a structured type.
    pub fn to_notification_preferences(&self) -> Result<NotificationPreferences, AuthError> {
        serde_json::from_value(self.notification_preferences.clone()).map_err(|e| {
            AuthError::Validation(format!(
                "Invalid notification preferences in database: {}",
                e
            ))
        })
    }
}
