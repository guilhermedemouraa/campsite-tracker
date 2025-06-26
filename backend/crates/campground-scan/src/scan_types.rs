use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Request structure for creating a new campground scan
#[derive(Debug, Deserialize, Validate)]
pub struct CreateScanRequest {
    /// ID of the campground to scan (RIDB facility ID)
    #[validate(length(min = 1, message = "Campground ID is required"))]
    pub campground_id: String,

    /// Name of the campground for display purposes
    #[validate(length(min = 1, message = "Campground name is required"))]
    pub campground_name: String,

    /// Check-in date for the camping reservation
    pub check_in_date: NaiveDate,

    /// Check-out date for the camping reservation
    pub check_out_date: NaiveDate,
}

/// Response structure for creating a scan
#[derive(Debug, Serialize)]
pub struct CreateScanResponse {
    /// Unique identifier for the created scan
    pub id: Uuid,
    /// ID of the campground
    pub campground_id: String,
    /// Name of the campground
    pub campground_name: String,
    /// Check-in date
    pub check_in_date: NaiveDate,
    /// Check-out date
    pub check_out_date: NaiveDate,
    /// Number of nights calculated from dates
    pub nights: i32,
    /// Current status of the scan
    pub status: String,
    /// Whether a notification has been sent
    pub notification_sent: bool,
    /// When the scan was created
    pub created_at: DateTime<Utc>,
}

/// Structure representing a user scan from the database
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserScan {
    /// Unique identifier for the scan
    pub id: Uuid,
    /// ID of the user who created the scan
    pub user_id: Uuid,
    /// ID of the campground
    pub campground_id: String,
    /// Check-in date
    pub check_in_date: NaiveDate,
    /// Check-out date
    pub check_out_date: NaiveDate,
    /// Number of nights (computed by database)
    pub nights: i32,
    /// Current status of the scan
    pub status: String,
    /// Whether a notification has been sent
    pub notification_sent: bool,
    /// When the scan was created
    pub created_at: DateTime<Utc>,
    /// When the scan was last updated
    pub updated_at: DateTime<Utc>,
    /// When the scan expires (optional)
    pub expires_at: Option<DateTime<Utc>>,
}

/// Enhanced user scan with campground information
#[derive(Debug, Serialize)]
pub struct UserScanWithCampground {
    /// Unique identifier for the scan
    pub id: Uuid,
    /// ID of the campground
    pub campground_id: String,
    /// Name of the campground
    pub campground_name: String,
    /// Check-in date
    pub check_in_date: NaiveDate,
    /// Check-out date
    pub check_out_date: NaiveDate,
    /// Number of nights
    pub nights: i32,
    /// Current status of the scan
    pub status: String,
    /// Whether a notification has been sent
    pub notification_sent: bool,
    /// When the scan was created
    pub created_at: DateTime<Utc>,
    /// When the scan was last updated
    pub updated_at: DateTime<Utc>,
    /// When the scan expires (optional)
    pub expires_at: Option<DateTime<Utc>>,
}

/// Request structure for updating a scan
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateScanRequest {
    /// New status for the scan
    #[validate(custom(function = "validate_scan_status"))]
    pub status: String,
}

/// Response structure for listing user scans
#[derive(Debug, Serialize)]
pub struct ListScansResponse {
    /// List of user scans with campground information
    pub scans: Vec<UserScanWithCampground>,
    /// Total count of scans
    pub total: i64,
}

/// Custom error type for scan operations
#[derive(thiserror::Error, Debug)]
pub enum ScanError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Scan not found
    #[error("Scan not found")]
    NotFound,

    /// User not authorized to access this scan
    #[error("Unauthorized access to scan")]
    Unauthorized,

    /// Invalid date range
    #[error("Invalid date range: check-out date must be after check-in date")]
    InvalidDateRange,

    /// Campground not found
    #[error("Campground not found")]
    CampgroundNotFound,
}

impl actix_web::ResponseError for ScanError {
    fn error_response(&self) -> actix_web::HttpResponse {
        use actix_web::HttpResponse;

        match self {
            ScanError::Validation(msg) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "validation_error",
                "message": msg
            })),
            ScanError::NotFound => HttpResponse::NotFound().json(serde_json::json!({
                "error": "scan_not_found",
                "message": "Scan not found"
            })),
            ScanError::Unauthorized => HttpResponse::Forbidden().json(serde_json::json!({
                "error": "unauthorized",
                "message": "You are not authorized to access this scan"
            })),
            ScanError::InvalidDateRange => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_date_range",
                "message": "Check-out date must be after check-in date"
            })),
            ScanError::CampgroundNotFound => HttpResponse::NotFound().json(serde_json::json!({
                "error": "campground_not_found",
                "message": "Campground not found"
            })),
            _ => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "An internal error occurred"
            })),
        }
    }
}

/// Custom validation function for scan status
fn validate_scan_status(status: &str) -> Result<(), validator::ValidationError> {
    match status {
        "active" | "paused" | "completed" | "cancelled" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_scan_status")),
    }
}
