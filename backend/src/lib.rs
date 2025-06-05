//! # Campsite Tracker
//!
//! A web application for tracking campsite availability using the RIDB (Recreation Information Database) API.
//! This library provides authentication, database connectivity, and campsite search functionality.

/// API endpoints and handlers for campsite search functionality.
pub mod api;
pub use api::*;

/// Database connection and query utilities.
pub mod database;
pub use database::*;

/// Authentication and user management system including JWT tokens, user registration, and login.
pub mod auth;
pub use auth::*;
