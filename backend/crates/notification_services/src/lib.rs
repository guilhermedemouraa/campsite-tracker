//! # Notification Services
//!
//! This crate provides authentication services for the application.
//! //! It includes JWT token handling, middleware for request authentication, and service definitions.

/// Service definitions for user management and authentication operations.
pub mod service;
/// Types and structures used in authentication services.
pub mod types;

pub use service::{NotificationService, create_verification_store};
pub use types::{NotificationError, VerificationStore};
