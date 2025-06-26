//! # Campground Scan
//!
//! This crate provides types and services for managing campground scan operations.
//! It handles the creation, management, and database operations for user scans
//! that monitor campground availability.

/// Types for campground scan operations
mod scan_types;
pub use scan_types::*;

/// Service for handling campground scan database operations
mod scan_service;
pub use scan_service::*;
