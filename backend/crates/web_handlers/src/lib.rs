//! # Web Handlers for the Campsite Tracker Web Application
//!
//! This crate provides the web handlers for the Campsite Tracker application.

/// Search for campgrounds given a query string on the Rec.gov API.
mod handlers;
pub use handlers::*;

/// Types for campground scan operations
mod scan_types;
pub use scan_types::*;

/// Service for handling campground scan database operations
mod scan_service;
pub use scan_service::*;

/// Handlers for campground scan API endpoints
mod scan_handlers;
pub use scan_handlers::*;
