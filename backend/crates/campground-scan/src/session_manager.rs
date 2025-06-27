use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::{Client, cookie::Jar};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::scan_types::ScanError;

/// Manages HTTP sessions for recreation.gov
/// Based on the Python implementation that maintains cookies and user agents
pub struct SessionManager {
    client: Client,
    session_state: Arc<RwLock<SessionState>>,
    config: SessionConfig,
}

#[derive(Debug, Clone)]
struct SessionState {
    /// When the session was last validated
    last_validated: Option<DateTime<Utc>>,

    /// Whether the current session is valid
    is_valid: bool,

    /// Session cookies and headers
    user_agent: String,

    /// Number of consecutive failures
    failure_count: u32,
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// How often to validate the session (default: 30 minutes)
    pub validation_interval: Duration,

    /// Maximum failures before recreating session (default: 3)
    pub max_failures: u32,

    /// Base URL for recreation.gov
    pub base_url: String,

    /// User agents to rotate through
    pub user_agents: Vec<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            validation_interval: Duration::from_secs(30 * 60), // 30 minutes
            max_failures: 3,
            base_url: "https://www.recreation.gov".to_string(),
            user_agents: vec![
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36".to_string(),
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36".to_string(),
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36".to_string(),
            ],
        }
    }
}

/// Response from recreation.gov homepage for session validation
#[derive(Debug, Deserialize)]
struct RecGovHomeResponse {
    // We don't need to parse the full response, just validate we can access it
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: Option<SessionConfig>) -> Result<Self, ScanError> {
        let config = config.unwrap_or_default();

        // Create a cookie jar for session management
        let jar = Arc::new(Jar::default());

        let client = Client::builder()
            .cookie_provider(jar)
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ScanError::ApiError(format!("Failed to create session client: {}", e)))?;

        let initial_state = SessionState {
            last_validated: None,
            is_valid: false,
            user_agent: config.user_agents[0].clone(),
            failure_count: 0,
        };

        Ok(Self {
            client,
            session_state: Arc::new(RwLock::new(initial_state)),
            config,
        })
    }

    /// Ensure we have a valid session, creating one if needed
    pub async fn ensure_valid_session(&self) -> Result<(), ScanError> {
        let needs_validation = {
            let state = self.session_state.read().await;

            // Check if we need to validate
            match state.last_validated {
                None => true, // Never validated
                Some(last) => {
                    let elapsed = Utc::now() - last;
                    let interval = chrono::Duration::from_std(self.config.validation_interval)
                        .map_err(|e| {
                            ScanError::ConfigError(format!("Invalid validation interval: {}", e))
                        })?;

                    elapsed > interval
                        || !state.is_valid
                        || state.failure_count >= self.config.max_failures
                }
            }
        };

        if needs_validation {
            self.create_new_session().await?;
        }

        Ok(())
    }

    /// Create a new session by visiting recreation.gov homepage
    async fn create_new_session(&self) -> Result<(), ScanError> {
        info!("Creating new recreation.gov session");

        // Select a user agent (rotate through them)
        let user_agent = {
            let state = self.session_state.read().await;
            let index = (state.failure_count as usize) % self.config.user_agents.len();
            self.config.user_agents[index].clone()
        };

        debug!("Using user agent: {}", user_agent);

        // Make a request to the homepage to establish session
        let response = self
            .client
            .get(&self.config.base_url)
            .header("User-Agent", &user_agent)
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
            )
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await
            .map_err(|e| ScanError::ApiError(format!("Failed to create session: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Update failure count
            {
                let mut state = self.session_state.write().await;
                state.failure_count += 1;
                state.is_valid = false;
            }

            return Err(ScanError::ApiError(format!(
                "Session creation failed with status {}: {}",
                status, error_text
            )));
        }

        // Session created successfully
        {
            let mut state = self.session_state.write().await;
            state.last_validated = Some(Utc::now());
            state.is_valid = true;
            state.user_agent = user_agent;
            state.failure_count = 0;
        }

        info!("Successfully created recreation.gov session");
        Ok(())
    }

    /// Validate current session by making a lightweight request
    pub async fn validate_session(&self) -> Result<bool, ScanError> {
        debug!("Validating recreation.gov session");

        let user_agent = {
            let state = self.session_state.read().await;
            state.user_agent.clone()
        };

        // Make a simple request to validate session
        let response = self
            .client
            .head(&format!("{}/api/permits", self.config.base_url))
            .header("User-Agent", user_agent)
            .send()
            .await
            .map_err(|e| ScanError::ApiError(format!("Session validation failed: {}", e)))?;

        let is_valid = response.status().is_success() || response.status() == 404; // 404 is ok, means API is accessible

        // Update session state
        {
            let mut state = self.session_state.write().await;
            state.last_validated = Some(Utc::now());
            state.is_valid = is_valid;

            if !is_valid {
                state.failure_count += 1;
                warn!(
                    "Session validation failed, failure count: {}",
                    state.failure_count
                );
            } else {
                state.failure_count = 0;
                debug!("Session validation successful");
            }
        }

        Ok(is_valid)
    }

    /// Get the HTTP client with current session
    pub fn get_client(&self) -> &Client {
        &self.client
    }

    /// Get current session statistics
    pub async fn get_session_stats(&self) -> SessionStats {
        let state = self.session_state.read().await;

        SessionStats {
            is_valid: state.is_valid,
            last_validated: state.last_validated,
            failure_count: state.failure_count,
            user_agent: state.user_agent.clone(),
        }
    }

    /// Force recreation of session (useful for recovery)
    pub async fn reset_session(&self) -> Result<(), ScanError> {
        info!("Forcing session reset");

        {
            let mut state = self.session_state.write().await;
            state.last_validated = None;
            state.is_valid = false;
            state.failure_count = 0;
        }

        self.create_new_session().await
    }
}

/// Statistics about the current session
#[derive(Debug, Serialize)]
pub struct SessionStats {
    pub is_valid: bool,
    pub last_validated: Option<DateTime<Utc>>,
    pub failure_count: u32,
    pub user_agent: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let manager = SessionManager::new(None).unwrap();
        let stats = manager.get_session_stats().await;

        assert!(!stats.is_valid);
        assert!(stats.last_validated.is_none());
        assert_eq!(stats.failure_count, 0);
    }

    #[tokio::test]
    async fn test_session_validation_needed() {
        let manager = SessionManager::new(None).unwrap();

        // Should need validation initially
        let result = manager.ensure_valid_session().await;
        // This will fail in tests without network, but that's ok for unit tests

        let stats = manager.get_session_stats().await;
        assert_eq!(stats.failure_count, 1); // Expected failure in test environment
    }
}
