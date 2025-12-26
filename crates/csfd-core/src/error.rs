//! Error types for ČSFD Scraper
//!
//! This module defines all error types used throughout the library.
//! CsfdError implements Serialize for Tauri compatibility.

use serde::{Serialize, Serializer};
use thiserror::Error;

/// Error type for ČSFD Scraper operations
#[derive(Error, Debug)]
pub enum CsfdError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Failed to parse HTML content
    #[error("Failed to parse HTML: {0}")]
    ParseError(String),

    /// Required HTML element was not found
    #[error("Element not found: {0}")]
    ElementNotFound(String),

    /// Invalid URL format
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Rate limited by the server (HTTP 429)
    #[error("Rate limited - too many requests")]
    RateLimited,

    /// Requested resource was not found (HTTP 404)
    #[error("Series not found: {0}")]
    NotFound(String),

    /// Invalid CSFD ID provided
    #[error("Invalid CSFD ID: {0}")]
    InvalidId(u32),
}

/// Serialize CsfdError as a string for Tauri compatibility
impl Serialize for CsfdError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Result type alias for ČSFD Scraper operations
pub type Result<T> = std::result::Result<T, CsfdError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csfd_error_display_http_error() {
        // We can't easily create a reqwest::Error, so we test other variants
        let error = CsfdError::ParseError("invalid HTML".to_string());
        let display = error.to_string();
        assert!(!display.is_empty());
        assert!(display.contains("invalid HTML"));
    }

    #[test]
    fn test_csfd_error_display_parse_error() {
        let error = CsfdError::ParseError("missing element".to_string());
        assert_eq!(error.to_string(), "Failed to parse HTML: missing element");
    }

    #[test]
    fn test_csfd_error_display_element_not_found() {
        let error = CsfdError::ElementNotFound(".search-results".to_string());
        assert_eq!(error.to_string(), "Element not found: .search-results");
    }

    #[test]
    fn test_csfd_error_display_invalid_url() {
        let error = CsfdError::InvalidUrl("not-a-url".to_string());
        assert_eq!(error.to_string(), "Invalid URL: not-a-url");
    }

    #[test]
    fn test_csfd_error_display_rate_limited() {
        let error = CsfdError::RateLimited;
        assert_eq!(error.to_string(), "Rate limited - too many requests");
    }

    #[test]
    fn test_csfd_error_display_not_found() {
        let error = CsfdError::NotFound("Breaking Bad".to_string());
        assert_eq!(error.to_string(), "Series not found: Breaking Bad");
    }

    #[test]
    fn test_csfd_error_display_invalid_id() {
        let error = CsfdError::InvalidId(0);
        assert_eq!(error.to_string(), "Invalid CSFD ID: 0");
    }

    #[test]
    fn test_csfd_error_serialize() {
        let error = CsfdError::ParseError("test error".to_string());
        let json = serde_json::to_string(&error).unwrap();
        assert_eq!(json, "\"Failed to parse HTML: test error\"");
    }

    #[test]
    fn test_csfd_error_serialize_rate_limited() {
        let error = CsfdError::RateLimited;
        let json = serde_json::to_string(&error).unwrap();
        assert_eq!(json, "\"Rate limited - too many requests\"");
    }

    #[test]
    fn test_csfd_error_serialize_invalid_id() {
        let error = CsfdError::InvalidId(42);
        let json = serde_json::to_string(&error).unwrap();
        assert_eq!(json, "\"Invalid CSFD ID: 42\"");
    }
}
