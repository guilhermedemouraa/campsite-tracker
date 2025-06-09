//! # RecGov
//!
//! This crate provides a client for the Rec.gov API, which is used to search for campgrounds and other facilities.

/// Search for campgrounds given a query string on the Rec.gov API.
mod facility_search;
pub use facility_search::*;
