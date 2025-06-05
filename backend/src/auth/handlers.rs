use actix_web::{web, HttpResponse, Result};
use bcrypt::hash;
use sqlx::{PgPool, Row};
use validator::Validate;

use super::jwt::JwtService;
use super::middleware::AuthenticatedUser;
use super::service::AuthService;
use super::types::{AuthError, AuthResponse, LoginRequest, SignUpRequest, UserInfo};

/// Handles user signup by validating the request, creating a new user,
/// generating access and refresh tokens, and returning the user info.
/// Returns a 201 Created response with the user info and tokens.
pub async fn signup(
    pool: web::Data<PgPool>,
    request: web::Json<SignUpRequest>,
) -> Result<HttpResponse, AuthError> {
    // Validate the request
    request
        .validate()
        .map_err(|e| AuthError::Validation(format!("Validation error: {}", e)))?;

    let auth_service = AuthService::new(pool.get_ref().clone());
    let jwt_service = JwtService::new();

    // Create the user
    let user = auth_service.create_user(&request).await?;

    // Generate tokens
    let access_token = jwt_service.generate_access_token(&user)?;
    let refresh_token = jwt_service.generate_refresh_token(&user.id)?;

    // Hash and store the refresh token
    let refresh_token_hash = hash(&refresh_token, bcrypt::DEFAULT_COST)?;
    let _session_id = auth_service
        .create_session(&user.id, &refresh_token_hash)
        .await?;

    // Prepare response
    let notification_prefs = user.to_notification_preferences()?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        user: UserInfo {
            id: user.id,
            name: user.name,
            email: user.email,
            phone: user.phone.unwrap_or_default(),
            email_verified: user.email_verified,
            phone_verified: user.phone_verified,
            notification_preferences: notification_prefs,
        },
    };

    Ok(HttpResponse::Created().json(response))
}

/// Handles user login by validating the request, verifying credentials,
/// generating access and refresh tokens, and returning the user info.
pub async fn login(
    pool: web::Data<PgPool>,
    request: web::Json<LoginRequest>,
) -> Result<HttpResponse, AuthError> {
    // Validate the request
    request
        .validate()
        .map_err(|e| AuthError::Validation(format!("Validation error: {}", e)))?;

    let auth_service = AuthService::new(pool.get_ref().clone());
    let jwt_service = JwtService::new();

    // Verify credentials
    let user = auth_service
        .verify_password(&request.email, &request.password)
        .await?;

    // Generate tokens
    let access_token = jwt_service.generate_access_token(&user)?;
    let refresh_token = jwt_service.generate_refresh_token(&user.id)?;

    // Hash and store the refresh token
    let refresh_token_hash = hash(&refresh_token, bcrypt::DEFAULT_COST)?;
    let _session_id = auth_service
        .create_session(&user.id, &refresh_token_hash)
        .await?;

    // Prepare response
    let notification_prefs = user.to_notification_preferences()?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        user: UserInfo {
            id: user.id,
            name: user.name,
            email: user.email,
            phone: user.phone.unwrap_or_default(),
            email_verified: user.email_verified,
            phone_verified: user.phone_verified,
            notification_preferences: notification_prefs,
        },
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Handles user profile retrieval by fetching user info based on the authenticated user.
pub async fn get_profile(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, AuthError> {
    let auth_service = AuthService::new(pool.get_ref().clone());

    let user = auth_service
        .get_user_by_id(&user.0)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    let notification_prefs = user.to_notification_preferences()?;

    let user_info = UserInfo {
        id: user.id,
        name: user.name,
        email: user.email,
        phone: user.phone.unwrap_or_default(),
        email_verified: user.email_verified,
        phone_verified: user.phone_verified,
        notification_preferences: notification_prefs,
    };

    Ok(HttpResponse::Ok().json(user_info))
}

// Helper function to validate signup request manually if needed
fn _validate_signup_request(request: &SignUpRequest) -> Result<(), AuthError> {
    use validator::Validate;

    request
        .validate()
        .map_err(|e| AuthError::Validation(format!("Validation error: {}", e)))?;

    Ok(())
}

/// Health check endpoint for auth service
pub async fn auth_health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "service": "auth",
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    })))
}

/// Development endpoint to list users (remove in production!)
pub async fn list_users(pool: web::Data<PgPool>) -> Result<HttpResponse, AuthError> {
    let rows = sqlx::query(
        "SELECT id, name, email, phone, email_verified, phone_verified, created_at FROM users ORDER BY created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await?;

    let users: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.get::<uuid::Uuid, _>("id"),
                "name": row.get::<String, _>("name"),
                "email": row.get::<String, _>("email"),
                "phone": row.get::<Option<String>, _>("phone"),
                "email_verified": row.get::<bool, _>("email_verified"),
                "phone_verified": row.get::<bool, _>("phone_verified"),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at")
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "users": users,
        "count": users.len()
    })))
}
