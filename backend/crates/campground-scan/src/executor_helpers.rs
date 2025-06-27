// Continuation of executor.rs - Helper methods for ScanExecutor

use std::collections::HashMap;
use chrono::{NaiveDate, Utc};
use tokio::time::sleep;
use tracing::{debug, info, error};
use uuid::Uuid;

use crate::{ScanExecutor, PollingJob, UserScan, ScanError, CampgroundAvailability, SiteAvailability};

impl ScanExecutor {
    /// Get polling jobs that need to be executed
    async fn get_jobs_needing_poll(&self) -> Result<Vec<PollingJob>, ScanError> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                campground_id, active_scan_count, last_polled, next_poll_at,
                poll_frequency_minutes, consecutive_errors, is_being_polled, priority
            FROM polling_jobs
            WHERE active_scan_count > 0
              AND next_poll_at <= NOW()
              AND NOT is_being_polled
              AND consecutive_errors < $1
            ORDER BY priority DESC, next_poll_at ASC
            LIMIT 50
            "#,
            self.config.max_consecutive_errors
        )
        .fetch_all(&self.pool)
        .await?;

        let jobs = rows
            .into_iter()
            .map(|row| PollingJob {
                campground_id: row.campground_id,
                active_scan_count: row.active_scan_count,
                last_polled: row.last_polled,
                next_poll_at: row.next_poll_at,
                poll_frequency_minutes: row.poll_frequency_minutes,
                consecutive_errors: row.consecutive_errors,
                is_being_polled: row.is_being_polled,
                priority: row.priority,
            })
            .collect();

        Ok(jobs)
    }

    /// Get active scans for a specific campground
    async fn get_active_scans_for_campground(&self, campground_id: &str) -> Result<Vec<UserScan>, ScanError> {
        let rows = sqlx::query_as!(
            UserScan,
            r#"
            SELECT 
                id, user_id, campground_id, check_in_date, check_out_date,
                nights, status, notification_sent, created_at, updated_at, expires_at
            FROM user_scans
            WHERE campground_id = $1 
              AND status = 'active'
              AND (expires_at IS NULL OR expires_at > NOW())
              AND check_out_date >= CURRENT_DATE
            "#,
            campground_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Calculate the overall date range needed for all scans
    fn calculate_date_range(&self, scans: &[UserScan]) -> (NaiveDate, NaiveDate) {
        let earliest = scans
            .iter()
            .map(|s| s.check_in_date)
            .min()
            .unwrap_or_else(|| chrono::Utc::now().date_naive());

        let latest = scans
            .iter()
            .map(|s| s.check_out_date)
            .max()
            .unwrap_or_else(|| chrono::Utc::now().date_naive());

        (earliest, latest)
    }

    /// Fetch availability from recreation.gov API
    async fn fetch_campground_availability(
        &self,
        campground_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<CampgroundAvailability, ScanError> {
        // Ensure we have a valid session
        self.session_manager.ensure_valid_session().await?;

        // Rate limiting
        self.enforce_rate_limit().await;

        // Make the API call
        let availability = self.rec_gov_client
            .get_campground_availability(campground_id, start_date, end_date)
            .await?;

        // Update API call tracking
        self.record_api_call().await;

        Ok(availability)
    }

    /// Get cached availability from database
    async fn get_cached_availability(
        &self,
        campground_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<NaiveDate, Vec<SiteAvailability>>, ScanError> {
        let rows = sqlx::query!(
            r#"
            SELECT date, availability_data
            FROM campground_availability
            WHERE campground_id = $1
              AND date >= $2
              AND date <= $3
              AND check_status = 'success'
            "#,
            campground_id,
            start_date,
            end_date
        )
        .fetch_all(&self.pool)
        .await?;

        let mut cached = HashMap::new();

        for row in rows {
            if let Some(data) = row.availability_data {
                let sites: Vec<SiteAvailability> = serde_json::from_value(data)
                    .map_err(|e| ScanError::DataFormat(e.to_string()))?;
                cached.insert(row.date, sites);
            }
        }

        Ok(cached)
    }

    /// Update availability cache in database
    async fn update_availability_cache(&self, availability: &CampgroundAvailability) -> Result<(), ScanError> {
        // Group sites by date
        let mut sites_by_date: HashMap<NaiveDate, Vec<SiteAvailability>> = HashMap::new();
        
        for site in &availability.available_sites {
            sites_by_date
                .entry(site.date)
                .or_insert_with(Vec::new)
                .push(site.clone());
        }

        // Insert/update each date
        for (date, sites) in sites_by_date {
            let available_count = sites.iter().filter(|s| s.available).count() as i32;
            let total_count = sites.len() as i32;
            let sites_json = serde_json::to_value(&sites)
                .map_err(|e| ScanError::DataFormat(e.to_string()))?;

            sqlx::query!(
                r#"
                INSERT INTO campground_availability 
                (campground_id, date, available_sites, total_sites, availability_data, last_checked, check_status)
                VALUES ($1, $2, $3, $4, $5, $6, 'success')
                ON CONFLICT (campground_id, date)
                DO UPDATE SET
                    available_sites = EXCLUDED.available_sites,
                    total_sites = EXCLUDED.total_sites,
                    availability_data = EXCLUDED.availability_data,
                    last_checked = EXCLUDED.last_checked,
                    check_status = EXCLUDED.check_status,
                    error_message = NULL
                "#,
                availability.campground_id,
                date,
                available_count,
                total_count,
                sites_json,
                availability.checked_at
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Find newly available sites by comparing with previous availability
    fn find_new_availability(
        &self,
        previous: &HashMap<NaiveDate, Vec<SiteAvailability>>,
        current: &CampgroundAvailability,
    ) -> Vec<SiteAvailability> {
        let mut new_sites = Vec::new();

        for site in &current.available_sites {
            if !site.available {
                continue;
            }

            let is_new = if let Some(prev_sites) = previous.get(&site.date) {
                // Check if this site was previously unavailable or not in cache
                !prev_sites.iter().any(|prev_site| {
                    prev_site.site_id == site.site_id && prev_site.available
                })
            } else {
                // No previous data for this date, so it's new
                true
            };

            if is_new {
                new_sites.push(site.clone());
            }
        }

        new_sites
    }

    /// Send notifications to users for new availability
    async fn send_notifications_for_new_availability(
        &self,
        scans: &[UserScan],
        new_sites: &[SiteAvailability],
    ) -> Result<(), ScanError> {
        for scan in scans {
            // Check if any new sites overlap with this scan's date range
            let relevant_sites: Vec<&SiteAvailability> = new_sites
                .iter()
                .filter(|site| {
                    site.date >= scan.check_in_date && site.date < scan.check_out_date
                })
                .collect();

            if relevant_sites.is_empty() {
                continue;
            }

            // Check if we've already sent a notification for this scan
            if scan.notification_sent {
                debug!("Notification already sent for scan {}", scan.id);
                continue;
            }

            // Create availability data for notification
            let availability = CampgroundAvailability {
                campground_id: scan.campground_id.clone(),
                available_sites: relevant_sites.into_iter().cloned().collect(),
                total_sites: relevant_sites.len(),
                checked_at: Utc::now(),
            };

            // Send notification
            match self.notification_service
                .send_availability_notification(&scan.user_id, &scan.id, &availability)
                .await
            {
                Ok(_) => {
                    info!("Sent notification for scan {} to user {}", scan.id, scan.user_id);
                    
                    // Mark notification as sent
                    if let Err(e) = self.mark_notification_sent(&scan.id).await {
                        error!("Failed to mark notification as sent: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to send notification for scan {}: {}", scan.id, e);
                }
            }
        }

        Ok(())
    }

    /// Mark a scan as having notification sent
    async fn mark_notification_sent(&self, scan_id: &Uuid) -> Result<(), ScanError> {
        sqlx::query!(
            "UPDATE user_scans SET notification_sent = true, updated_at = NOW() WHERE id = $1",
            scan_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark a polling job as in progress or complete
    async fn mark_job_in_progress(&self, campground_id: &str, in_progress: bool) -> Result<(), ScanError> {
        sqlx::query!(
            "UPDATE polling_jobs SET is_being_polled = $1, updated_at = NOW() WHERE campground_id = $2",
            in_progress,
            campground_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update job after successful poll
    async fn update_job_success(&self, job: &PollingJob) -> Result<(), ScanError> {
        let next_poll = Utc::now() + chrono::Duration::minutes(job.poll_frequency_minutes as i64);

        sqlx::query!(
            r#"
            UPDATE polling_jobs 
            SET last_polled = NOW(),
                next_poll_at = $1,
                consecutive_errors = 0,
                is_being_polled = false,
                updated_at = NOW()
            WHERE campground_id = $2
            "#,
            next_poll,
            job.campground_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update job after error
    async fn update_job_error(&self, job: &PollingJob, error_message: &str) -> Result<(), ScanError> {
        let new_error_count = job.consecutive_errors + 1;
        let next_poll = if new_error_count >= self.config.max_consecutive_errors {
            // Backoff on max errors
            Utc::now() + chrono::Duration::from_std(self.config.error_backoff_duration).unwrap()
        } else {
            // Normal retry interval
            Utc::now() + chrono::Duration::minutes(job.poll_frequency_minutes as i64)
        };

        sqlx::query!(
            r#"
            UPDATE polling_jobs 
            SET consecutive_errors = $1,
                next_poll_at = $2,
                is_being_polled = false,
                updated_at = NOW()
            WHERE campground_id = $3
            "#,
            new_error_count,
            next_poll,
            job.campground_id
        )
        .execute(&self.pool)
        .await?;

        // Also update availability cache with error status
        sqlx::query!(
            r#"
            INSERT INTO campground_availability 
            (campground_id, date, check_status, error_message, last_checked)
            VALUES ($1, CURRENT_DATE, 'error', $2, NOW())
            ON CONFLICT (campground_id, date)
            DO UPDATE SET
                check_status = 'error',
                error_message = EXCLUDED.error_message,
                last_checked = EXCLUDED.last_checked
            "#,
            job.campground_id,
            error_message
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if we can make an API call (rate limiting)
    async fn can_make_api_call(&self) -> bool {
        let call_count = *self.api_call_count.lock().await;
        call_count < self.config.max_calls_per_hour
    }

    /// Enforce rate limiting between API calls
    async fn enforce_rate_limit(&self) {
        let last_call = *self.last_api_call.lock().await;
        let time_since_last = Utc::now() - last_call;
        let min_interval = chrono::Duration::from_std(self.config.min_api_interval).unwrap();

        if time_since_last < min_interval {
            let sleep_duration = min_interval - time_since_last;
            if let Ok(sleep_std) = sleep_duration.to_std() {
                sleep(sleep_std).await;
            }
        }
    }

    /// Record an API call for rate limiting
    async fn record_api_call(&self) {
        *self.last_api_call.lock().await = Utc::now();
        *self.api_call_count.lock().await += 1;
    }

    /// Reset API call count every hour
    async fn reset_api_count_if_needed(&self) {
        let last_call = *self.last_api_call.lock().await;
        let hour_ago = Utc::now() - chrono::Duration::hours(1);

        if last_call < hour_ago {
            *self.api_call_count.lock().await = 0;
        }
    }
}
