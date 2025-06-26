use actix_web::{HttpResponse, Result, web};
use bcrypt::hash;
use sqlx::PgPool;
use validator::Validate;

use auth_services::jwt::JwtService;
use auth_services::service::AuthService;
use auth_services::types::*;
use notification_services::service::*;
use notification_services::types::*;

/// Handles user signup by validating the request, creating a new user,
/// generating access and refresh tokens, and returning the user info.
/// Returns a 201 Created response with the user info and tokens.
pub async fn signup(
    pool: web::Data<PgPool>,
    notification_service: web::Data<NotificationService>,
    verification_store: web::Data<VerificationStore>,
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

    // Send verification email with LINK (not code)
    let verification_token = NotificationService::generate_verification_token(); // 32-char token
    let email_key = format!("email_token_{}_{}", user.id, user.email); // Different key format

    store_verification_code(&verification_store, &email_key, &verification_token, 1440); // 24 hours

    // Try to send verification email link (don't fail signup if this fails)
    if let Err(e) = notification_service
        .send_email_verification_link(&user.id, &user.email, &user.name, &verification_token)
        .await
    {
        log::warn!("Failed to send verification email during signup: {}", e);
        // Continue with signup - user can verify later
    }

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
            email_verified: user.email_verified, // Will be false for new users
            phone_verified: user.phone_verified, // Will be false for new users
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
