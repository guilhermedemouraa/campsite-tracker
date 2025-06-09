use actix_web::{HttpResponse, Result, web};
use bcrypt::hash;
use sqlx::{PgPool, Row};
use validator::Validate;

use auth_services::jwt::JwtService;
use auth_services::middleware::AuthenticatedUser;
use auth_services::service::AuthService;
use auth_services::types::*;
use notification_services::service::*;
use notification_services::types::*;

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

/// Handles user profile update by validating the request, updating user info,
pub async fn update_profile(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    request: web::Json<UpdateProfileRequest>,
) -> Result<HttpResponse, AuthError> {
    // Validate the request
    request
        .validate()
        .map_err(|e| AuthError::Validation(format!("Validation error: {}", e)))?;

    let auth_service = AuthService::new(pool.get_ref().clone());

    // Update user profile
    let updated_user = auth_service.update_user_profile(&user.0, &request).await?;

    let notification_prefs = updated_user.to_notification_preferences()?;

    let user_info = UserInfo {
        id: updated_user.id,
        name: updated_user.name,
        email: updated_user.email,
        phone: updated_user.phone.unwrap_or_default(),
        email_verified: updated_user.email_verified,
        phone_verified: updated_user.phone_verified,
        notification_preferences: notification_prefs,
    };

    Ok(HttpResponse::Ok().json(user_info))
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

/// Send email verification code
pub async fn send_email_verification(
    pool: web::Data<PgPool>,
    notification_service: web::Data<NotificationService>,
    verification_store: web::Data<VerificationStore>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, AuthError> {
    let auth_service = AuthService::new(pool.get_ref().clone());
    let user_data = auth_service
        .get_user_by_id(&user.0)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    let verification_code = NotificationService::generate_verification_code();
    let key = format!("email_{}_{}", user.0, user_data.email);

    store_verification_code(&verification_store, &key, &verification_code, 1440); // 24 hours

    notification_service
        .send_email_verification(
            &user.0,
            &user_data.email,
            &user_data.name,
            &verification_code,
        )
        .await
        .map_err(|e| AuthError::Validation(format!("Failed to send email: {}", e)))?;

    Ok(HttpResponse::Ok().json(VerificationResponse {
        message: "Verification email sent successfully".to_string(),
    }))
}

/// Verify email with code
pub async fn verify_email(
    pool: web::Data<PgPool>,
    verification_store: web::Data<VerificationStore>,
    user: AuthenticatedUser,
    request: web::Json<VerifyEmailRequest>,
) -> Result<HttpResponse, AuthError> {
    let auth_service = AuthService::new(pool.get_ref().clone());
    let user_data = auth_service
        .get_user_by_id(&user.0)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    let key = format!("email_{}_{}", user.0, user_data.email);

    match verify_code(&verification_store, &key, &request.code) {
        Ok(true) => {
            // Update user email verification status
            auth_service
                .update_user_verification(&user.0, Some(true), None)
                .await?;

            Ok(HttpResponse::Ok().json(VerificationResponse {
                message: "Email verified successfully!".to_string(),
            }))
        }
        Ok(false) => Err(AuthError::Validation(
            "Invalid verification code".to_string(),
        )),
        Err(err) => Err(AuthError::Validation(err)),
    }
}

/// Send SMS verification code
pub async fn send_sms_verification(
    pool: web::Data<PgPool>,
    notification_service: web::Data<NotificationService>,
    verification_store: web::Data<VerificationStore>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, AuthError> {
    let auth_service = AuthService::new(pool.get_ref().clone());
    let user_data = auth_service
        .get_user_by_id(&user.0)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    let phone = user_data
        .phone
        .ok_or(AuthError::Validation("No phone number on file".to_string()))?;
    let verification_code = NotificationService::generate_verification_code();
    let key = format!("sms_{}_{}", user.0, phone);

    store_verification_code(&verification_store, &key, &verification_code, 10); // 10 minutes

    notification_service
        .send_sms_verification(&user.0, &phone, &verification_code)
        .await
        .map_err(|e| AuthError::Validation(format!("Failed to send SMS: {}", e)))?;

    Ok(HttpResponse::Ok().json(VerificationResponse {
        message: "Verification SMS sent successfully".to_string(),
    }))
}

/// Verify phone with code
pub async fn verify_phone(
    pool: web::Data<PgPool>,
    verification_store: web::Data<VerificationStore>,
    user: AuthenticatedUser,
    request: web::Json<VerifyPhoneRequest>,
) -> Result<HttpResponse, AuthError> {
    let auth_service = AuthService::new(pool.get_ref().clone());
    let user_data = auth_service
        .get_user_by_id(&user.0)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    let phone = user_data
        .phone
        .ok_or(AuthError::Validation("No phone number on file".to_string()))?;
    let key = format!("sms_{}_{}", user.0, phone);

    match verify_code(&verification_store, &key, &request.code) {
        Ok(true) => {
            // Update user phone verification status
            auth_service
                .update_user_verification(&user.0, None, Some(true))
                .await?;

            Ok(HttpResponse::Ok().json(VerificationResponse {
                message: "Phone number verified successfully!".to_string(),
            }))
        }
        Ok(false) => Err(AuthError::Validation(
            "Invalid verification code".to_string(),
        )),
        Err(err) => Err(AuthError::Validation(err)),
    }
}
