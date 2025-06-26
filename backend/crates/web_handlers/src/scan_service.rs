use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::scan_types::*;

/// Service for handling campground scan operations
pub struct ScanService {
    pool: PgPool,
}

impl ScanService {
    /// Creates a new instance of `ScanService` with the provided database connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new scan for the specified user
    pub async fn create_scan(
        &self,
        user_id: &Uuid,
        request: &CreateScanRequest,
    ) -> Result<UserScan, ScanError> {
        // Validate date range
        if request.check_out_date <= request.check_in_date {
            return Err(ScanError::InvalidDateRange);
        }

        // First, ensure the campground exists in our database
        self.ensure_campground_exists(&request.campground_id, &request.campground_name)
            .await?;

        // Create the scan
        let row = sqlx::query(
            r#"
            INSERT INTO user_scans (
                user_id, campground_id, check_in_date, check_out_date
            ) VALUES ($1, $2, $3, $4)
            RETURNING 
                id, user_id, campground_id, check_in_date, check_out_date,
                nights, status, notification_sent, created_at, updated_at, expires_at
            "#,
        )
        .bind(user_id)
        .bind(&request.campground_id)
        .bind(request.check_in_date)
        .bind(request.check_out_date)
        .fetch_one(&self.pool)
        .await?;

        let scan = UserScan {
            id: row.get("id"),
            user_id: row.get("user_id"),
            campground_id: row.get("campground_id"),
            check_in_date: row.get("check_in_date"),
            check_out_date: row.get("check_out_date"),
            nights: row.get("nights"),
            status: row.get("status"),
            notification_sent: row.get("notification_sent"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            expires_at: row.get("expires_at"),
        };

        Ok(scan)
    }

    /// Gets all scans for a specific user with campground information
    pub async fn get_user_scans(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<UserScanWithCampground>, ScanError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                us.id, us.campground_id, us.check_in_date, us.check_out_date,
                us.nights, us.status, us.notification_sent, us.created_at, 
                us.updated_at, us.expires_at, c.name as campground_name
            FROM user_scans us
            LEFT JOIN campgrounds c ON us.campground_id = c.id
            WHERE us.user_id = $1
            ORDER BY us.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let scans = rows
            .into_iter()
            .map(|row| UserScanWithCampground {
                id: row.get("id"),
                campground_id: row.get("campground_id"),
                campground_name: row
                    .get::<Option<String>, _>("campground_name")
                    .unwrap_or_else(|| "Unknown Campground".to_string()),
                check_in_date: row.get("check_in_date"),
                check_out_date: row.get("check_out_date"),
                nights: row.get("nights"),
                status: row.get("status"),
                notification_sent: row.get("notification_sent"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                expires_at: row.get("expires_at"),
            })
            .collect();

        Ok(scans)
    }

    /// Gets a specific scan by ID, ensuring it belongs to the user
    pub async fn get_user_scan(
        &self,
        user_id: &Uuid,
        scan_id: &Uuid,
    ) -> Result<UserScanWithCampground, ScanError> {
        let row = sqlx::query(
            r#"
            SELECT 
                us.id, us.campground_id, us.check_in_date, us.check_out_date,
                us.nights, us.status, us.notification_sent, us.created_at, 
                us.updated_at, us.expires_at, c.name as campground_name
            FROM user_scans us
            LEFT JOIN campgrounds c ON us.campground_id = c.id
            WHERE us.id = $1 AND us.user_id = $2
            "#,
        )
        .bind(scan_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(UserScanWithCampground {
                id: row.get("id"),
                campground_id: row.get("campground_id"),
                campground_name: row
                    .get::<Option<String>, _>("campground_name")
                    .unwrap_or_else(|| "Unknown Campground".to_string()),
                check_in_date: row.get("check_in_date"),
                check_out_date: row.get("check_out_date"),
                nights: row.get("nights"),
                status: row.get("status"),
                notification_sent: row.get("notification_sent"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                expires_at: row.get("expires_at"),
            }),
            None => Err(ScanError::NotFound),
        }
    }

    /// Updates a scan's status
    pub async fn update_scan_status(
        &self,
        user_id: &Uuid,
        scan_id: &Uuid,
        new_status: &str,
    ) -> Result<UserScanWithCampground, ScanError> {
        // First check if the scan exists and belongs to the user
        let _existing_scan = self.get_user_scan(user_id, scan_id).await?;

        // Update the scan
        let row = sqlx::query(
            r#"
            UPDATE user_scans 
            SET status = $1, updated_at = NOW()
            WHERE id = $2 AND user_id = $3
            RETURNING 
                id, user_id, campground_id, check_in_date, check_out_date,
                nights, status, notification_sent, created_at, updated_at, expires_at
            "#,
        )
        .bind(new_status)
        .bind(scan_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        // Get the campground name
        let campground_name = self
            .get_campground_name(&row.get::<String, _>("campground_id"))
            .await?;

        Ok(UserScanWithCampground {
            id: row.get("id"),
            campground_id: row.get("campground_id"),
            campground_name,
            check_in_date: row.get("check_in_date"),
            check_out_date: row.get("check_out_date"),
            nights: row.get("nights"),
            status: row.get("status"),
            notification_sent: row.get("notification_sent"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            expires_at: row.get("expires_at"),
        })
    }

    /// Deletes a scan
    pub async fn delete_scan(&self, user_id: &Uuid, scan_id: &Uuid) -> Result<(), ScanError> {
        let result = sqlx::query("DELETE FROM user_scans WHERE id = $1 AND user_id = $2")
            .bind(scan_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(ScanError::NotFound);
        }

        Ok(())
    }

    /// Ensures a campground exists in the database, creating it if necessary
    async fn ensure_campground_exists(
        &self,
        campground_id: &str,
        campground_name: &str,
    ) -> Result<(), ScanError> {
        sqlx::query(
            r#"
            INSERT INTO campgrounds (id, name)
            VALUES ($1, $2)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                last_updated = NOW()
            "#,
        )
        .bind(campground_id)
        .bind(campground_name)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Gets the name of a campground by ID
    async fn get_campground_name(&self, campground_id: &str) -> Result<String, ScanError> {
        let row = sqlx::query("SELECT name FROM campgrounds WHERE id = $1")
            .bind(campground_id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(row.get("name")),
            None => Ok("Unknown Campground".to_string()),
        }
    }
}
