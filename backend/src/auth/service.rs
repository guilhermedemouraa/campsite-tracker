use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::types::{
    validate_phone_number, AuthError, NotificationPreferences, SignUpRequest, User,
};

pub struct AuthService {
    pool: PgPool,
}

impl AuthService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

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

    fn format_phone_number(&self, phone: &str) -> String {
        // Remove all non-digit characters
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

        // Add +1 if it's a 10-digit US number
        if digits.len() == 10 {
            format!("+1{}", digits)
        } else if digits.len() == 11 && digits.starts_with('1') {
            format!("+{}", digits)
        } else {
            format!("+{}", digits)
        }
    }
}

impl User {
    pub fn to_notification_preferences(&self) -> Result<NotificationPreferences, AuthError> {
        serde_json::from_value(self.notification_preferences.clone()).map_err(|e| {
            AuthError::Validation(format!(
                "Invalid notification preferences in database: {}",
                e
            ))
        })
    }
}
