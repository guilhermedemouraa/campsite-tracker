use actix_web::{HttpResponse, Result, web};
use notification_services::types::DeleteUserQuery;
use sqlx::{PgPool, Row};

use auth_services::types::AuthError;

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

/// Handles user deletion by email, deleting user sessions first to avoid foreign key constraints.
/// Returns a 200 OK response with a message if successful, or a 404 Not Found
pub async fn delete_user_by_email(
    pool: web::Data<PgPool>,
    query: web::Query<DeleteUserQuery>,
) -> Result<HttpResponse, AuthError> {
    let email = &query.email;

    log::warn!("🚨 DELETING USER WITH EMAIL: {}", email);

    // Delete user sessions first (foreign key constraint)
    sqlx::query(
        "DELETE FROM user_sessions WHERE user_id IN (SELECT id FROM users WHERE email = $1)",
    )
    .bind(email)
    .execute(pool.get_ref())
    .await?;

    // Delete the user
    let result = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": format!("User with email {} deleted successfully", email),
            "deleted": true
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "message": format!("No user found with email {}", email),
            "deleted": false
        })))
    }
}
