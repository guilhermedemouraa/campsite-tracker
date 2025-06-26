use actix_web::{HttpResponse, Result, web};
use sqlx::PgPool;
use validator::Validate;

use auth_services::middleware::AuthenticatedUser;
use auth_services::service::AuthService;
use auth_services::types::*;

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
