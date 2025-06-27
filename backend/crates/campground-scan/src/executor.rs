use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, sleep};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::rec_gov_client::RecGovClient;
use crate::scan_types::*;
use crate::session_manager::SessionManager;

/// Represents the availability data for a campsite on a specific date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteAvailability {
    pub site_id: String,
    pub site_name: String,
    pub available: bool,
    pub date: NaiveDate,
    pub price: Option<f64>,
}

/// Aggregated availability data for a campground on a date range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampgroundAvailability {
    pub campground_id: String,
    pub available_sites: Vec<SiteAvailability>,
    pub total_sites: usize,
    pub checked_at: DateTime<Utc>,
}

/// Polling job metadata stored in database
#[derive(Debug, Clone)]
pub struct PollingJob {
    pub campground_id: String,
    pub active_scan_count: i32,
    pub last_polled: Option<DateTime<Utc>>,
    pub next_poll_at: DateTime<Utc>,
    pub poll_frequency_minutes: i32,
    pub consecutive_errors: i32,
    pub is_being_polled: bool,
    pub priority: i32,
}

/// Main scan execution engine
pub struct ScanExecutor {
    pool: PgPool,
    rec_gov_client: Arc<RecGovClient>,
    session_manager: Arc<SessionManager>,
    notification_service: Arc<dyn NotificationService>,

    /// In-memory state to prevent duplicate polling
    active_polls: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,

    /// Rate limiting state
    last_api_call: Arc<Mutex<DateTime<Utc>>>,
    api_call_count: Arc<Mutex<u32>>,

    /// Configuration
    config: ScanExecutorConfig,
}

#[derive(Debug, Clone)]
pub struct ScanExecutorConfig {
    /// Minimum interval between API calls (default: 5 seconds)
    pub min_api_interval: Duration,

    /// Maximum API calls per hour (default: 1000)
    pub max_calls_per_hour: u32,

    /// How often to check for new polling jobs (default: 30 seconds)
    pub poll_check_interval: Duration,

    /// Default polling frequency for campgrounds (default: 15 minutes)
    pub default_poll_frequency: Duration,

    /// Maximum consecutive errors before pausing a job (default: 5)
    pub max_consecutive_errors: i32,

    /// How long to pause a job after max errors (default: 1 hour)
    pub error_backoff_duration: Duration,
}

impl Default for ScanExecutorConfig {
    fn default() -> Self {
        Self {
            min_api_interval: Duration::from_secs(5),
            max_calls_per_hour: 1000,
            poll_check_interval: Duration::from_secs(30),
            default_poll_frequency: Duration::from_secs(15 * 60), // 15 minutes
            max_consecutive_errors: 5,
            error_backoff_duration: Duration::from_secs(60 * 60), // 1 hour
        }
    }
}

/// Trait for notification services (SMS, Email)
#[async_trait::async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_availability_notification(
        &self,
        user_id: &Uuid,
        scan_id: &Uuid,
        availability: &CampgroundAvailability,
    ) -> Result<(), NotificationError>;
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Email error: {0}")]
    Email(String),
    #[error("SMS error: {0}")]
    Sms(String),
}

impl ScanExecutor {
    pub fn new(
        pool: PgPool,
        rec_gov_client: Arc<RecGovClient>,
        session_manager: Arc<SessionManager>,
        notification_service: Arc<dyn NotificationService>,
        config: Option<ScanExecutorConfig>,
    ) -> Self {
        Self {
            pool,
            rec_gov_client,
            session_manager,
            notification_service,
            active_polls: Arc::new(RwLock::new(HashMap::new())),
            last_api_call: Arc::new(Mutex::new(DateTime::<Utc>::MIN_UTC)),
            api_call_count: Arc::new(Mutex::new(0)),
            config: config.unwrap_or_default(),
        }
    }

    /// Start the scan execution engine
    pub async fn start(&self) -> Result<(), ScanError> {
        info!("Starting scan execution engine");

        // Log initial job count
        if let Ok(total_jobs) = self.get_total_active_jobs().await {
            info!("Monitoring {} active scan jobs", total_jobs);
        } else {
            warn!("Could not retrieve initial job count");
        }

        // Start the main polling loop
        let mut poll_interval = interval(self.config.poll_check_interval);

        loop {
            poll_interval.tick().await;

            if let Err(e) = self.process_polling_jobs().await {
                error!("Error processing polling jobs: {}", e);
            }

            // Reset API call count every hour
            self.reset_api_count_if_needed().await;
        }
    }

    /// Main polling logic - finds jobs that need to be polled and executes them
    async fn process_polling_jobs(&self) -> Result<(), ScanError> {
        debug!("Checking for polling jobs");

        // Get jobs that need polling
        let jobs = self.get_jobs_needing_poll().await?;

        if jobs.is_empty() {
            debug!("No polling jobs need processing");
            return Ok(());
        }

        info!("Found {} jobs needing polling", jobs.len());

        // Process jobs with priority (higher priority first)
        let mut sorted_jobs = jobs;
        sorted_jobs.sort_by(|a, b| b.priority.cmp(&a.priority));

        for job in sorted_jobs {
            // Check if we're already polling this campground
            {
                let active_polls = self.active_polls.read().await;
                if active_polls.contains_key(&job.campground_id) {
                    debug!("Skipping {}, already being polled", job.campground_id);
                    continue;
                }
            }

            // Check rate limits
            if !self.can_make_api_call().await {
                warn!("Rate limit reached, pausing polling");
                break;
            }

            // Mark as being polled
            {
                let mut active_polls = self.active_polls.write().await;
                active_polls.insert(job.campground_id.clone(), Utc::now());
            }

            // Mark job as being polled in database
            self.mark_job_in_progress(&job.campground_id, true).await?;

            // Execute the polling in a background task
            let executor = self.clone_for_task();
            let job_clone = job.clone();

            tokio::spawn(async move {
                let result = executor.poll_campground(&job_clone).await;

                // Remove from active polls
                {
                    let mut active_polls = executor.active_polls.write().await;
                    active_polls.remove(&job_clone.campground_id);
                }

                // Update job status
                if let Err(e) = executor
                    .mark_job_in_progress(&job_clone.campground_id, false)
                    .await
                {
                    error!("Failed to mark job as complete: {}", e);
                }

                match result {
                    Ok(_) => {
                        debug!("Successfully polled campground {}", job_clone.campground_id);
                        if let Err(e) = executor.update_job_success(&job_clone).await {
                            error!("Failed to update job success: {}", e);
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to poll campground {}: {}",
                            job_clone.campground_id, e
                        );
                        if let Err(e) = executor.update_job_error(&job_clone, &e.to_string()).await
                        {
                            error!("Failed to update job error: {}", e);
                        }
                    }
                }
            });

            // Small delay between job starts to prevent overwhelming the API
            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Poll a specific campground for availability
    async fn poll_campground(&self, job: &PollingJob) -> Result<(), ScanError> {
        info!(
            "Polling campground {} with {} active scans",
            job.campground_id, job.active_scan_count
        );

        // Get active scans for this campground
        let scans = self
            .get_active_scans_for_campground(&job.campground_id)
            .await?;

        if scans.is_empty() {
            warn!("No active scans found for campground {}", job.campground_id);
            return Ok(());
        }

        // Determine date range to check (union of all scan date ranges)
        let (earliest_date, latest_date) = self.calculate_date_range(&scans);

        // Get current availability from recreation.gov
        let new_availability = self
            .fetch_campground_availability(&job.campground_id, earliest_date, latest_date)
            .await?;

        // Get previous availability from cache
        let previous_availability = self
            .get_cached_availability(&job.campground_id, earliest_date, latest_date)
            .await?;

        // Update availability cache
        self.update_availability_cache(&new_availability).await?;

        // Find new availability (sites that became available)
        let new_sites = self.find_new_availability(&previous_availability, &new_availability);

        if !new_sites.is_empty() {
            info!(
                "Found {} newly available sites in {}",
                new_sites.len(),
                job.campground_id
            );

            // Send notifications to users whose scans match the new availability
            self.send_notifications_for_new_availability(&scans, &new_sites)
                .await?;
        } else {
            debug!("No new availability found for {}", job.campground_id);
        }

        Ok(())
    }

    /// Create a clone suitable for background tasks
    fn clone_for_task(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            rec_gov_client: self.rec_gov_client.clone(),
            session_manager: self.session_manager.clone(),
            notification_service: self.notification_service.clone(),
            active_polls: self.active_polls.clone(),
            last_api_call: self.last_api_call.clone(),
            api_call_count: self.api_call_count.clone(),
            config: self.config.clone(),
        }
    }

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
                active_scan_count: row.active_scan_count.unwrap_or(0),
                last_polled: row.last_polled,
                next_poll_at: row.next_poll_at.unwrap_or_else(|| Utc::now()),
                poll_frequency_minutes: row.poll_frequency_minutes.unwrap_or(15),
                consecutive_errors: row.consecutive_errors.unwrap_or(0),
                is_being_polled: row.is_being_polled.unwrap_or(false),
                priority: row.priority.unwrap_or(1),
            })
            .collect();

        Ok(jobs)
    }

    /// Get active scans for a specific campground
    async fn get_active_scans_for_campground(
        &self,
        campground_id: &str,
    ) -> Result<Vec<UserScan>, ScanError> {
        let rows = sqlx::query_as!(
            UserScan,
            r#"
            SELECT 
                id, user_id, campground_id, check_in_date, check_out_date,
                COALESCE(nights, 0) as "nights!: i32", 
                COALESCE(status, 'active') as "status!: String", 
                COALESCE(notification_sent, false) as "notification_sent!: bool", 
                COALESCE(created_at, NOW()) as "created_at!: DateTime<Utc>", 
                COALESCE(updated_at, NOW()) as "updated_at!: DateTime<Utc>", 
                expires_at
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

        // Make the API call using the internal Recreation.gov API
        let availability = self
            .rec_gov_client
            .get_internal_campground_availability(campground_id, start_date, end_date)
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
    async fn update_availability_cache(
        &self,
        availability: &CampgroundAvailability,
    ) -> Result<(), ScanError> {
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
            let sites_json =
                serde_json::to_value(&sites).map_err(|e| ScanError::DataFormat(e.to_string()))?;

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
                !prev_sites
                    .iter()
                    .any(|prev_site| prev_site.site_id == site.site_id && prev_site.available)
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
                .filter(|site| site.date >= scan.check_in_date && site.date < scan.check_out_date)
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
            let total_sites = relevant_sites.len();
            let available_sites: Vec<SiteAvailability> =
                relevant_sites.into_iter().cloned().collect();

            let availability = CampgroundAvailability {
                campground_id: scan.campground_id.clone(),
                available_sites,
                total_sites,
                checked_at: Utc::now(),
            };

            // Send notification
            match self
                .notification_service
                .send_availability_notification(&scan.user_id, &scan.id, &availability)
                .await
            {
                Ok(_) => {
                    info!(
                        "Sent notification for scan {} to user {}",
                        scan.id, scan.user_id
                    );

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
    async fn mark_job_in_progress(
        &self,
        campground_id: &str,
        in_progress: bool,
    ) -> Result<(), ScanError> {
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
    async fn update_job_error(
        &self,
        job: &PollingJob,
        error_message: &str,
    ) -> Result<(), ScanError> {
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

    /// Get total count of active jobs being monitored
    async fn get_total_active_jobs(&self) -> Result<i64, ScanError> {
        let row =
            sqlx::query!("SELECT COUNT(*) as count FROM polling_jobs WHERE active_scan_count > 0")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ScanError::Database(e))?;

        Ok(row.count.unwrap_or(0))
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
