//! # Campground Scan
//!
//! This crate provides a complete campground monitoring system that polls recreation.gov
//! for availability and sends notifications to users when campsites become available.
//!
//! ## Features
//!
//! - **Scan Management**: Create, update, and delete user scans
//! - **Execution Engine**: Background polling system with deduplication
//! - **Rate Limiting**: Respects recreation.gov API limits
//! - **Session Management**: Handles authentication with recreation.gov
//! - **Notifications**: Email and SMS alerts for new availability
//! - **Persistence**: Resumes monitoring after server restarts

/// Types for campground scan operations
mod scan_types;
pub use scan_types::*;

/// Service for handling campground scan database operations
mod scan_service;
pub use scan_service::*;

/// Main scan execution engine with background polling
mod executor;
pub use executor::*;

/// Recreation.gov API client
mod rec_gov_client;
pub use rec_gov_client::*;

/// Session management for recreation.gov authentication
mod session_manager;
pub use session_manager::*;

/// Notification service for sending alerts
mod notification_service;
pub use notification_service::*;

/// Email service implementations
mod email_service;
pub use email_service::*;

/// SMS service implementations
mod sms_service;
pub use sms_service::*;
