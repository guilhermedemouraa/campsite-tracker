use std::sync::Arc;

use sqlx::PgPool;
use tokio::task::JoinHandle;
use tracing::{error, info};

use campground_scan::{
    EmailService, MockEmailService, MockSmsService, NotificationServiceImpl, RecGovClient,
    ScanExecutor, ScanExecutorConfig, SessionConfig, SessionManager, SmsService,
};

/// Manager for the scan execution system
/// Integrates with the web server to provide background scanning
pub struct ScanManager {
    pool: PgPool,
    executor_handle: Option<JoinHandle<()>>,
    executor: Option<Arc<ScanExecutor>>,
}

impl ScanManager {
    /// Create a new scan manager
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            executor_handle: None,
            executor: None,
        }
    }

    /// Start the scan execution engine
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting scan execution system");

        // Get configuration from environment
        let rec_gov_api_key = std::env::var("RECREATION_GOV_API_KEY").ok();
        let executor_config = ScanExecutorConfig::default();
        let session_config = SessionConfig::default();

        // Create recreation.gov client
        let rec_gov_client = Arc::new(RecGovClient::new(rec_gov_api_key)?);

        // Create session manager
        let session_manager = Arc::new(SessionManager::new(Some(session_config))?);

        // Create notification service
        let email_service: Arc<dyn EmailService> = Arc::new(MockEmailService);
        let sms_service: Arc<dyn SmsService> = Arc::new(MockSmsService);

        let notification_service = Arc::new(NotificationServiceImpl::new(
            self.pool.clone(),
            Some(email_service),
            Some(sms_service),
        ));

        // Create scan executor
        let executor = Arc::new(ScanExecutor::new(
            self.pool.clone(),
            rec_gov_client,
            session_manager,
            notification_service,
            Some(executor_config),
        ));

        // Store executor reference
        self.executor = Some(executor.clone());

        // Start the executor in a background task
        let executor_clone = executor.clone();
        let handle = tokio::spawn(async move {
            if let Err(e) = executor_clone.start().await {
                error!("Scan executor failed: {}", e);
            }
        });

        self.executor_handle = Some(handle);

        info!("Scan execution system started successfully");
        Ok(())
    }

    /// Stop the scan execution engine
    pub async fn stop(&mut self) {
        info!("Stopping scan execution system");

        if let Some(handle) = self.executor_handle.take() {
            handle.abort();
            let _ = handle.await;
        }

        self.executor = None;

        info!("Scan execution system stopped");
    }

    /// Get statistics about the scan execution system
    pub async fn get_stats(&self) -> Option<ScanExecutorStats> {
        if let Some(ref executor) = self.executor {
            // TODO: Implement stats collection from executor
            Some(ScanExecutorStats {
                active_polls: 0,
                total_scans: 0,
                last_poll: None,
                api_calls_remaining: 1000,
            })
        } else {
            None
        }
    }

    /// Force a scan of a specific campground (for testing/admin)
    pub async fn force_scan(
        &self,
        campground_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref executor) = self.executor {
            // TODO: Implement forced scan functionality
            info!("Force scanning campground: {}", campground_id);
            Ok(())
        } else {
            Err("Scan executor not running".into())
        }
    }
}

/// Statistics about the scan executor
#[derive(Debug, serde::Serialize)]
pub struct ScanExecutorStats {
    pub active_polls: u32,
    pub total_scans: u64,
    pub last_poll: Option<chrono::DateTime<chrono::Utc>>,
    pub api_calls_remaining: u32,
}

impl Drop for ScanManager {
    fn drop(&mut self) {
        if let Some(handle) = self.executor_handle.take() {
            handle.abort();
        }
    }
}
