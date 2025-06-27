use actix_web::{HttpResponse, Result, web};
use auth_services::middleware::AuthenticatedUser;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

/// Helper function to get user role from database
async fn get_user_role(pool: &PgPool, user_id: &Uuid) -> Result<String, sqlx::Error> {
    let row = sqlx::query!("SELECT role FROM users WHERE id = $1", user_id)
        .fetch_one(pool)
        .await?;
    Ok(row.role.unwrap_or_default())
}

/// System monitoring endpoint for scan execution system (requires authentication)
pub async fn get_scan_system_status(
    pool: web::Data<PgPool>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    // Get scan system statistics
    let stats = get_scan_system_stats(&pool).await?;

    Ok(HttpResponse::Ok().json(stats))
}

/// Force a scan of a specific campground (requires authentication)
pub async fn force_scan_campground(
    _pool: web::Data<PgPool>,
    _user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let campground_id = path.into_inner();

    // TODO: Implement force scan functionality
    // This would trigger an immediate scan of the specified campground

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Force scan initiated for campground {}", campground_id),
        "campground_id": campground_id
    })))
}

/// Get polling job statistics (requires authentication)
pub async fn get_polling_jobs(
    pool: web::Data<PgPool>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let jobs = sqlx::query!(
        r#"
        SELECT 
            pj.campground_id,
            COALESCE(c.name, 'Unknown Campground') as "campground_name!: String",
            COALESCE(pj.active_scan_count, 0) as "active_scan_count!: i32",
            pj.last_polled,
            COALESCE(pj.next_poll_at, NOW()) as "next_poll_at!: DateTime<Utc>",
            COALESCE(pj.poll_frequency_minutes, 15) as "poll_frequency_minutes!: i32",
            COALESCE(pj.consecutive_errors, 0) as "consecutive_errors!: i32",
            COALESCE(pj.is_being_polled, false) as "is_being_polled!: bool",
            COALESCE(pj.priority, 1) as "priority!: i32"
        FROM polling_jobs pj
        LEFT JOIN campgrounds c ON pj.campground_id = c.id
        WHERE pj.active_scan_count > 0
        ORDER BY pj.priority DESC, pj.next_poll_at ASC
        LIMIT 50
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    let jobs = match jobs {
        Ok(jobs) => jobs,
        Err(e) => {
            log::error!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "database_error",
                "message": "Failed to fetch polling jobs"
            })));
        }
    };

    let job_list: Vec<PollingJobInfo> = jobs
        .into_iter()
        .map(|row| PollingJobInfo {
            campground_id: row.campground_id,
            campground_name: row.campground_name,
            active_scan_count: row.active_scan_count,
            last_polled: row.last_polled,
            next_poll_at: row.next_poll_at,
            poll_frequency_minutes: row.poll_frequency_minutes,
            consecutive_errors: row.consecutive_errors,
            is_being_polled: row.is_being_polled,
            priority: row.priority,
        })
        .collect();

    Ok(HttpResponse::Ok().json(job_list))
}

/// Get recent notifications (requires authentication)
pub async fn get_recent_notifications(
    pool: web::Data<PgPool>,
    _user: AuthenticatedUser,
) -> Result<HttpResponse> {
    let notifications = sqlx::query!(
        r#"
        SELECT 
            n.id,
            n.user_id,
            COALESCE(u.email, 'Unknown') as "user_email!: String",
            n.type as notification_type,
            n.recipient,
            n.subject,
            COALESCE(n.status, 'unknown') as "status!: String",
            n.sent_at,
            COALESCE(n.created_at, NOW()) as "created_at!: DateTime<Utc>",
            us.campground_id as "campground_id?: String",
            c.name as "campground_name?: String"
        FROM notifications n
        LEFT JOIN users u ON n.user_id = u.id
        LEFT JOIN user_scans us ON n.user_scan_id = us.id
        LEFT JOIN campgrounds c ON us.campground_id = c.id
        ORDER BY n.created_at DESC
        LIMIT 100
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    let notifications = match notifications {
        Ok(notifications) => notifications,
        Err(e) => {
            log::error!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "database_error",
                "message": "Failed to fetch notifications"
            })));
        }
    };

    let notification_list: Vec<NotificationInfo> = notifications
        .into_iter()
        .map(|row| NotificationInfo {
            id: row.id,
            user_id: row.user_id,
            user_email: row.user_email,
            notification_type: row.notification_type,
            recipient: row.recipient,
            subject: row.subject,
            status: row.status,
            sent_at: row.sent_at,
            created_at: row.created_at,
            campground_id: row.campground_id,
            campground_name: row.campground_name,
        })
        .collect();

    Ok(HttpResponse::Ok().json(notification_list))
}

/// Get system-wide scan statistics
async fn get_scan_system_stats(pool: &PgPool) -> Result<ScanSystemStats> {
    // Get total active scans
    let active_scans = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM user_scans WHERE status = 'active' AND check_out_date >= CURRENT_DATE"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    // Get total polling jobs
    let active_jobs =
        sqlx::query_scalar!("SELECT COUNT(*) FROM polling_jobs WHERE active_scan_count > 0")
            .fetch_one(pool)
            .await
            .unwrap_or(Some(0))
            .unwrap_or(0);

    // Get jobs currently being polled
    let jobs_in_progress =
        sqlx::query_scalar!("SELECT COUNT(*) FROM polling_jobs WHERE is_being_polled = true")
            .fetch_one(pool)
            .await
            .unwrap_or(Some(0))
            .unwrap_or(0);

    // Get error count
    let jobs_with_errors =
        sqlx::query_scalar!("SELECT COUNT(*) FROM polling_jobs WHERE consecutive_errors > 0")
            .fetch_one(pool)
            .await
            .unwrap_or(Some(0))
            .unwrap_or(0);

    // Get recent availability checks
    let recent_checks = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM campground_availability 
        WHERE last_checked > NOW() - INTERVAL '1 hour'
        "#
    )
    .fetch_one(pool)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    // Get notifications sent today
    let notifications_today = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM notifications 
        WHERE sent_at >= CURRENT_DATE AND status = 'sent'
        "#
    )
    .fetch_one(pool)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    Ok(ScanSystemStats {
        active_scans,
        active_polling_jobs: active_jobs,
        jobs_in_progress,
        jobs_with_errors,
        recent_api_checks: recent_checks,
        notifications_sent_today: notifications_today,
        system_status: if jobs_with_errors == 0 {
            "healthy"
        } else {
            "degraded"
        }
        .to_string(),
        last_updated: chrono::Utc::now(),
    })
}

#[derive(Debug, Serialize)]
struct ScanSystemStats {
    active_scans: i64,
    active_polling_jobs: i64,
    jobs_in_progress: i64,
    jobs_with_errors: i64,
    recent_api_checks: i64,
    notifications_sent_today: i64,
    system_status: String,
    last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
struct PollingJobInfo {
    campground_id: String,
    campground_name: String,
    active_scan_count: i32,
    last_polled: Option<chrono::DateTime<chrono::Utc>>,
    next_poll_at: chrono::DateTime<chrono::Utc>,
    poll_frequency_minutes: i32,
    consecutive_errors: i32,
    is_being_polled: bool,
    priority: i32,
}

#[derive(Debug, Serialize)]
struct NotificationInfo {
    id: Uuid,
    user_id: Uuid,
    user_email: String,
    notification_type: String,
    recipient: String,
    subject: Option<String>,
    status: String,
    sent_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    campground_id: Option<String>,
    campground_name: Option<String>,
}
