use actix_web::{HttpResponse, Result, web};

use auth_services::middleware::AuthenticatedUser;
use auth_services::service::AuthService;
use auth_services::types::*;
use notification_services::service::*;
use notification_services::types::*;
use sqlx::PgPool;

/// Send email verification link
pub async fn send_email_verification_link(
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

    let verification_token = NotificationService::generate_verification_token(); // 32-char token
    let key = format!("email_token_{}_{}", user.0, user_data.email); // Updated key format

    store_verification_code(&verification_store, &key, &verification_token, 1440); // 24 hours

    notification_service
        .send_email_verification_link(
            // Updated function name
            &user.0,
            &user_data.email,
            &user_data.name,
            &verification_token, // Token instead of code
        )
        .await
        .map_err(|e| AuthError::Validation(format!("Failed to send email: {}", e)))?;

    Ok(HttpResponse::Ok().json(VerificationResponse {
        message: "Verification email sent successfully".to_string(),
    }))
}

/// Verify email with token (from email link)
pub async fn verify_email_with_token(
    pool: web::Data<PgPool>,
    verification_store: web::Data<VerificationStore>,
    query: web::Query<EmailVerificationQuery>,
) -> Result<HttpResponse, AuthError> {
    let token = &query.token;

    // Look for verification token in store
    let store = verification_store.lock().unwrap();
    let mut found_key = None;
    let mut user_id = None;

    for (key, verification) in store.iter() {
        if key.starts_with("email_token_") && verification.code == *token {
            if verification.expires_at > chrono::Utc::now() {
                found_key = Some(key.clone());
                // Extract user_id from key format: "email_token_{user_id}_{email}"
                if let Some(id_part) = key.split('_').nth(2) {
                    if let Ok(parsed_id) = uuid::Uuid::parse_str(id_part) {
                        user_id = Some(parsed_id);
                    }
                }
            }
            break;
        }
    }

    drop(store);

    if let (Some(key), Some(uid)) = (found_key, user_id) {
        // Remove the token
        verification_store.lock().unwrap().remove(&key);

        // Update user verification status
        let auth_service = AuthService::new(pool.get_ref().clone());
        auth_service
            .update_user_verification(&uid, Some(true), None)
            .await?;

        // Return success page HTML
        Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(EMAIL_VERIFICATION_SUCCESS_HTML))
    } else {
        Ok(HttpResponse::BadRequest()
            .content_type("text/html")
            .body(EMAIL_VERIFICATION_ERROR_HTML))
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
