//! HTTP client with rate limiting for ČSFD.cz
//!
//! This module provides a rate-limited HTTP client that respects ČSFD.cz
//! server limits and implements retry logic with exponential backoff.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::error::{CsfdError, Result};

/// Base URL for ČSFD.cz
const CSFD_BASE_URL: &str = "https://www.csfd.cz";

/// Default User-Agent mimicking a modern browser
const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Default Accept-Language header for Czech content
const DEFAULT_ACCEPT_LANGUAGE: &str = "cs-CZ,cs;q=0.9,en;q=0.8";

/// Maximum number of retry attempts for transient errors
const MAX_RETRIES: u32 = 3;

/// Base delay for exponential backoff (in milliseconds)
const BASE_RETRY_DELAY_MS: u64 = 1000;

/// Rate limiter to control request frequency
///
/// Ensures that requests are spaced at least `min_interval` apart
/// to avoid overwhelming the ČSFD.cz server.
pub struct RateLimiter {
    /// Minimum interval between requests
    min_interval: Duration,
    /// Timestamp of the last request
    last_request: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter with the specified requests per second
    ///
    /// # Arguments
    /// * `requests_per_second` - Maximum number of requests allowed per second
    ///
    /// # Example
    /// ```
    /// use csfd_core::client::RateLimiter;
    ///
    /// let limiter = RateLimiter::new(2.0); // 2 requests per second
    /// ```
    pub fn new(requests_per_second: f64) -> Self {
        let min_interval = Duration::from_secs_f64(1.0 / requests_per_second);
        Self {
            min_interval,
            last_request: Arc::new(Mutex::new(Instant::now() - min_interval)),
        }
    }

    /// Acquire permission to make a request
    ///
    /// This method will wait if necessary to ensure the minimum interval
    /// between requests is respected.
    pub async fn acquire(&self) {
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();

        if elapsed < self.min_interval {
            let wait_time = self.min_interval - elapsed;
            sleep(wait_time).await;
        }

        *last = Instant::now();
    }

    /// Get the minimum interval between requests
    pub fn min_interval(&self) -> Duration {
        self.min_interval
    }
}


/// Configuration for the ČSFD HTTP client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Maximum requests per second (default: 2.0)
    pub requests_per_second: f64,
    /// Request timeout in seconds (default: 30)
    pub timeout_secs: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 2.0,
            timeout_secs: 30,
        }
    }
}

/// HTTP client for ČSFD.cz with rate limiting and retry logic
///
/// This client automatically:
/// - Limits request rate to avoid server overload
/// - Retries on transient errors (429, 5xx) with exponential backoff
/// - Sets appropriate headers for Czech content
pub struct CsfdClient {
    /// Underlying HTTP client
    client: reqwest::Client,
    /// Rate limiter for request throttling
    rate_limiter: RateLimiter,
}

impl CsfdClient {
    /// Create a new client with default configuration
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be created
    pub fn new() -> Result<Self> {
        Self::with_config(ClientConfig::default())
    }

    /// Create a new client with custom configuration
    ///
    /// # Arguments
    /// * `config` - Client configuration
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be created
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(DEFAULT_USER_AGENT)
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::ACCEPT_LANGUAGE,
                    DEFAULT_ACCEPT_LANGUAGE.parse().unwrap(),
                );
                headers
            })
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()?;

        let rate_limiter = RateLimiter::new(config.requests_per_second);

        Ok(Self {
            client,
            rate_limiter,
        })
    }

    /// Fetch HTML content from a ČSFD.cz path
    ///
    /// This method handles rate limiting and retries automatically.
    ///
    /// # Arguments
    /// * `path` - Relative path on ČSFD.cz (e.g., "/hledat/?q=test")
    ///
    /// # Returns
    /// The HTML content as a string
    ///
    /// # Errors
    /// - `CsfdError::HttpError` - Network or HTTP error after all retries
    /// - `CsfdError::RateLimited` - Server returned 429 after all retries
    /// - `CsfdError::NotFound` - Server returned 404
    pub async fn fetch(&self, path: &str) -> Result<String> {
        let url = format!("{}{}", CSFD_BASE_URL, path);
        self.fetch_with_retry(&url, 0).await
    }

    /// Internal method to fetch with retry logic
    fn fetch_with_retry<'a>(
        &'a self,
        url: &'a str,
        attempt: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send + 'a>> {
        Box::pin(async move {
            // Wait for rate limiter before making request
            self.rate_limiter.acquire().await;

            let response = self.client.get(url).send().await?;
            let status = response.status();

            // Handle different status codes
            if status.is_success() {
                return Ok(response.text().await?);
            }

            // Handle 404 - Not Found (no retry)
            if status == reqwest::StatusCode::NOT_FOUND {
                return Err(CsfdError::NotFound(url.to_string()));
            }

            // Handle 429 - Rate Limited
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                if attempt < MAX_RETRIES {
                    let delay = self.calculate_backoff_delay(attempt);
                    sleep(delay).await;
                    return self.fetch_with_retry(url, attempt + 1).await;
                }
                return Err(CsfdError::RateLimited);
            }

            // Handle 5xx - Server errors
            if status.is_server_error() {
                if attempt < MAX_RETRIES {
                    let delay = self.calculate_backoff_delay(attempt);
                    sleep(delay).await;
                    return self.fetch_with_retry(url, attempt + 1).await;
                }
                return Err(CsfdError::HttpError(
                    response.error_for_status().unwrap_err(),
                ));
            }

            // Other errors - convert to HttpError
            Err(CsfdError::HttpError(
                response.error_for_status().unwrap_err(),
            ))
        })
    }

    /// Calculate exponential backoff delay for retry
    fn calculate_backoff_delay(&self, attempt: u32) -> Duration {
        // Exponential backoff: 1s, 2s, 4s, ...
        let delay_ms = BASE_RETRY_DELAY_MS * 2u64.pow(attempt);
        Duration::from_millis(delay_ms)
    }

    /// Get a reference to the rate limiter (for testing)
    #[cfg(test)]
    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(2.0);
        assert_eq!(limiter.min_interval(), Duration::from_millis(500));
    }

    #[test]
    fn test_rate_limiter_different_rates() {
        let limiter = RateLimiter::new(1.0);
        assert_eq!(limiter.min_interval(), Duration::from_secs(1));

        let limiter = RateLimiter::new(4.0);
        assert_eq!(limiter.min_interval(), Duration::from_millis(250));
    }

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.requests_per_second, 2.0);
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_client_creation() {
        let client = CsfdClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_custom_config() {
        let config = ClientConfig {
            requests_per_second: 1.0,
            timeout_secs: 60,
        };
        let client = CsfdClient::with_config(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_backoff_delay_calculation() {
        let client = CsfdClient::new().unwrap();
        
        assert_eq!(client.calculate_backoff_delay(0), Duration::from_millis(1000));
        assert_eq!(client.calculate_backoff_delay(1), Duration::from_millis(2000));
        assert_eq!(client.calculate_backoff_delay(2), Duration::from_millis(4000));
    }

    #[tokio::test]
    async fn test_rate_limiter_acquire() {
        let limiter = RateLimiter::new(10.0); // 10 requests per second = 100ms interval
        
        let start = Instant::now();
        limiter.acquire().await;
        limiter.acquire().await;
        let elapsed = start.elapsed();
        
        // Second acquire should wait at least 100ms
        assert!(elapsed >= Duration::from_millis(100));
    }
}
