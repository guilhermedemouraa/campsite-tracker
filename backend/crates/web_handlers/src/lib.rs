//! # Web Handlers for the Campsite Tracker Web Application
//!
//! This crate provides the web handlers for the Campsite Tracker application.

/// Authentication handlers (signup, login)
mod auth_handlers;
pub use auth_handlers::*;

/// User profile handlers (get/update profile)
mod profile_handlers;
pub use profile_handlers::*;

/// Email and SMS verification handlers
mod verification_handlers;
pub use verification_handlers::*;

/// Admin and development handlers
mod admin_handlers;
pub use admin_handlers::*;

/// Handlers for campground scan API endpoints
mod scan_handlers;
pub use scan_handlers::*;
