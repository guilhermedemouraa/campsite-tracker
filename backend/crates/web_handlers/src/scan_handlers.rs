use actix_web::{HttpResponse, Result, web};
use validator::Validate;

use crate::scan_service::ScanService;
use crate::scan_types::*;
use auth_services::middleware::AuthenticatedUser;

/// Creates a new campground scan for the authenticated user
pub async fn create_scan(
    pool: web::Data<sqlx::PgPool>,
    user: AuthenticatedUser,
    request: web::Json<CreateScanRequest>,
) -> Result<HttpResponse, ScanError> {
    // Validate the request
    request
        .validate()
        .map_err(|e| ScanError::Validation(format!("Validation error: {}", e)))?;

    let scan_service = ScanService::new(pool.get_ref().clone());
    let scan = scan_service.create_scan(&user.0, &request).await?;

    // Convert to response format
    let response = CreateScanResponse {
        id: scan.id,
        campground_id: scan.campground_id,
        campground_name: request.campground_name.clone(),
        check_in_date: scan.check_in_date,
        check_out_date: scan.check_out_date,
        nights: scan.nights,
        status: scan.status,
        notification_sent: scan.notification_sent,
        created_at: scan.created_at,
    };

    Ok(HttpResponse::Created().json(response))
}

/// Gets all scans for the authenticated user
pub async fn get_user_scans(
    pool: web::Data<sqlx::PgPool>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ScanError> {
    let scan_service = ScanService::new(pool.get_ref().clone());
    let scans = scan_service.get_user_scans(&user.0).await?;

    let response = ListScansResponse {
        total: scans.len() as i64,
        scans,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Gets a specific scan by ID for the authenticated user
pub async fn get_scan(
    pool: web::Data<sqlx::PgPool>,
    user: AuthenticatedUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ScanError> {
    let scan_id = path.into_inner();
    let scan_service = ScanService::new(pool.get_ref().clone());
    let scan = scan_service.get_user_scan(&user.0, &scan_id).await?;

    Ok(HttpResponse::Ok().json(scan))
}

/// Updates a scan's status
pub async fn update_scan(
    pool: web::Data<sqlx::PgPool>,
    user: AuthenticatedUser,
    path: web::Path<uuid::Uuid>,
    request: web::Json<UpdateScanRequest>,
) -> Result<HttpResponse, ScanError> {
    // Validate the request
    request
        .validate()
        .map_err(|e| ScanError::Validation(format!("Validation error: {}", e)))?;

    let scan_id = path.into_inner();
    let scan_service = ScanService::new(pool.get_ref().clone());
    let updated_scan = scan_service
        .update_scan_status(&user.0, &scan_id, &request.status)
        .await?;

    Ok(HttpResponse::Ok().json(updated_scan))
}

/// Deletes a scan
pub async fn delete_scan(
    pool: web::Data<sqlx::PgPool>,
    user: AuthenticatedUser,
    path: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ScanError> {
    let scan_id = path.into_inner();
    let scan_service = ScanService::new(pool.get_ref().clone());
    scan_service.delete_scan(&user.0, &scan_id).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Gets active scans for the authenticated user (for display on profile page)
pub async fn get_active_scans(
    pool: web::Data<sqlx::PgPool>,
    user: AuthenticatedUser,
) -> Result<HttpResponse, ScanError> {
    let scan_service = ScanService::new(pool.get_ref().clone());
    let all_scans = scan_service.get_user_scans(&user.0).await?;

    // Filter only active scans
    let active_scans: Vec<UserScanWithCampground> = all_scans
        .into_iter()
        .filter(|scan| scan.status == "active")
        .collect();

    let response = ListScansResponse {
        total: active_scans.len() as i64,
        scans: active_scans,
    };

    Ok(HttpResponse::Ok().json(response))
}
