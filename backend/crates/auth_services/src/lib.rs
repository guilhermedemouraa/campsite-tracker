//! # Auth Services
//!
//! This crate provides authentication services for the application.
//! //! It includes JWT token handling, middleware for request authentication, and service definitions.

/// JWT token handling and user authentication services.
pub mod jwt;
/// Middleware for request authentication and user session management.
pub mod middleware;
/// Service definitions for user management and authentication operations.
pub mod service;
/// Types and structures used in authentication services.
pub mod types;
