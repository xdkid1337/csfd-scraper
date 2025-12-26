//! ČSFD Scraper Core Library
//!
//! This crate provides the core scraping functionality for ČSFD.cz
//! (Česko-Slovenská filmová databáze).
//!
//! # Features
//! - Search for TV series by name
//! - Get series details including seasons
//! - Get episode lists with ratings
//! - Rate-limited HTTP client to avoid server overload

pub mod client;
pub mod error;
pub mod parser;
pub mod scraper;
pub mod types;

// Re-export main types for convenience
pub use client::{ClientConfig, CsfdClient, RateLimiter};
pub use error::{CsfdError, Result};
pub use scraper::CsfdScraper;
pub use types::{Episode, PaginatedResult, SearchResult, Season, SeriesDetail, SeriesType};
