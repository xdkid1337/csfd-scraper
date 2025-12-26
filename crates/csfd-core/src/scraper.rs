//! Main ČSFD Scraper API
//!
//! This module provides the high-level API for scraping ČSFD.cz.
//! It combines the HTTP client with parsers to provide a simple interface
//! for searching series, getting details, and fetching episodes.

use crate::client::CsfdClient;
use crate::error::{CsfdError, Result};
use crate::parser::{parse_episodes, parse_search_results, parse_series_detail};
use crate::types::{Episode, PaginatedResult, SearchResult, SeriesDetail};

/// Main scraper API for ČSFD.cz
///
/// Provides methods for searching series, getting series details,
/// and fetching episode lists. All operations are asynchronous.
///
/// # Example
/// ```no_run
/// use csfd_core::CsfdScraper;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let scraper = CsfdScraper::new()?;
///     
///     // Search for series
///     let results = scraper.search("Breaking Bad").await?;
///     println!("Found {} results", results.items.len());
///     
///     Ok(())
/// }
/// ```
pub struct CsfdScraper {
    client: CsfdClient,
}

impl CsfdScraper {
    /// Create a new scraper with default configuration.
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be created.
    ///
    /// # Example
    /// ```
    /// use csfd_core::CsfdScraper;
    ///
    /// let scraper = CsfdScraper::new().expect("Failed to create scraper");
    /// ```
    pub fn new() -> Result<Self> {
        let client = CsfdClient::new()?;
        Ok(Self { client })
    }

    /// Create a new scraper with a custom client.
    ///
    /// This is useful for testing or when you need custom client configuration.
    ///
    /// # Arguments
    /// * `client` - Pre-configured CsfdClient instance
    pub fn with_client(client: CsfdClient) -> Self {
        Self { client }
    }


    /// Search for series by name.
    ///
    /// Returns the first page of search results. Use `search_page` for pagination.
    ///
    /// # Arguments
    /// * `query` - Search query string
    ///
    /// # Returns
    /// * `Ok(PaginatedResult<SearchResult>)` with matching series
    /// * `Err(CsfdError::InvalidUrl)` if query is empty or whitespace-only
    ///
    /// # Example
    /// ```no_run
    /// use csfd_core::CsfdScraper;
    ///
    /// # async fn example() -> Result<(), csfd_core::CsfdError> {
    /// let scraper = CsfdScraper::new()?;
    /// let results = scraper.search("Breaking Bad").await?;
    /// for item in results.items {
    ///     println!("{} ({})", item.name, item.csfd_id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(&self, query: &str) -> Result<PaginatedResult<SearchResult>> {
        self.search_page(query, 1).await
    }

    /// Search for series by name with pagination.
    ///
    /// # Arguments
    /// * `query` - Search query string
    /// * `page` - Page number (1-based)
    ///
    /// # Returns
    /// * `Ok(PaginatedResult<SearchResult>)` with matching series
    /// * `Err(CsfdError::InvalidUrl)` if query is empty or whitespace-only
    ///
    /// # Example
    /// ```no_run
    /// use csfd_core::CsfdScraper;
    ///
    /// # async fn example() -> Result<(), csfd_core::CsfdError> {
    /// let scraper = CsfdScraper::new()?;
    /// let page2 = scraper.search_page("Game of Thrones", 2).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_page(&self, query: &str, page: u32) -> Result<PaginatedResult<SearchResult>> {
        // Validate query is not empty or whitespace-only
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Err(CsfdError::InvalidUrl("Search query cannot be empty".to_string()));
        }

        // URL encode the query
        let encoded_query = urlencoding::encode(trimmed);
        
        // Build search URL with pagination
        let path = if page > 1 {
            format!("/hledat/?q={}&page={}", encoded_query, page)
        } else {
            format!("/hledat/?q={}", encoded_query)
        };

        // Fetch and parse
        let html = self.client.fetch(&path).await?;
        let mut result = parse_search_results(&html)?;
        
        // Ensure current_page is set correctly
        result.current_page = page;
        
        Ok(result)
    }


    /// Get detailed information about a series.
    ///
    /// # Arguments
    /// * `csfd_id` - ČSFD ID of the series
    ///
    /// # Returns
    /// * `Ok(SeriesDetail)` with series information and seasons
    /// * `Err(CsfdError::InvalidId)` if csfd_id is 0
    /// * `Err(CsfdError::NotFound)` if series doesn't exist
    ///
    /// # Example
    /// ```no_run
    /// use csfd_core::CsfdScraper;
    ///
    /// # async fn example() -> Result<(), csfd_core::CsfdError> {
    /// let scraper = CsfdScraper::new()?;
    /// let series = scraper.get_series(12345).await?;
    /// println!("{} has {} seasons", series.name, series.seasons.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_series(&self, csfd_id: u32) -> Result<SeriesDetail> {
        // Validate ID
        if csfd_id == 0 {
            return Err(CsfdError::InvalidId(csfd_id));
        }

        // Fetch series detail page
        let path = format!("/film/{}/prehled/", csfd_id);
        let html = self.client.fetch(&path).await?;
        
        // Parse and return
        parse_series_detail(&html, csfd_id)
    }

    /// Get all episodes for a series.
    ///
    /// # Arguments
    /// * `csfd_id` - ČSFD ID of the series
    ///
    /// # Returns
    /// * `Ok(Vec<Episode>)` with all episodes
    /// * `Err(CsfdError::InvalidId)` if csfd_id is 0
    ///
    /// # Example
    /// ```no_run
    /// use csfd_core::CsfdScraper;
    ///
    /// # async fn example() -> Result<(), csfd_core::CsfdError> {
    /// let scraper = CsfdScraper::new()?;
    /// let episodes = scraper.get_episodes(12345).await?;
    /// for ep in episodes {
    ///     println!("{}: {}", ep.episode_code, ep.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_episodes(&self, csfd_id: u32) -> Result<Vec<Episode>> {
        // Validate ID
        if csfd_id == 0 {
            return Err(CsfdError::InvalidId(csfd_id));
        }

        // Fetch episodes page
        let path = format!("/film/{}/epizody/", csfd_id);
        let html = self.client.fetch(&path).await?;
        
        // Parse and return
        parse_episodes(&html)
    }

    /// Get episodes for a specific season.
    ///
    /// # Arguments
    /// * `series_id` - ČSFD ID of the series
    /// * `season_id` - ČSFD ID of the season
    ///
    /// # Returns
    /// * `Ok(Vec<Episode>)` with episodes from the specified season
    /// * `Err(CsfdError::InvalidId)` if either ID is 0
    ///
    /// # Example
    /// ```no_run
    /// use csfd_core::CsfdScraper;
    ///
    /// # async fn example() -> Result<(), csfd_core::CsfdError> {
    /// let scraper = CsfdScraper::new()?;
    /// let episodes = scraper.get_season_episodes(12345, 67890).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_season_episodes(&self, series_id: u32, season_id: u32) -> Result<Vec<Episode>> {
        // Validate IDs
        if series_id == 0 {
            return Err(CsfdError::InvalidId(series_id));
        }
        if season_id == 0 {
            return Err(CsfdError::InvalidId(season_id));
        }

        // Fetch season episodes page
        let path = format!("/film/{}/{}/epizody/", series_id, season_id);
        let html = self.client.fetch(&path).await?;
        
        // Parse and return
        parse_episodes(&html)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scraper_creation() {
        let scraper = CsfdScraper::new();
        assert!(scraper.is_ok());
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let scraper = CsfdScraper::new().unwrap();
        let result = scraper.search("").await;
        assert!(result.is_err());
        
        match result {
            Err(CsfdError::InvalidUrl(msg)) => {
                assert!(msg.contains("empty"));
            }
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    #[tokio::test]
    async fn test_search_whitespace_query() {
        let scraper = CsfdScraper::new().unwrap();
        let result = scraper.search("   ").await;
        assert!(result.is_err());
        
        match result {
            Err(CsfdError::InvalidUrl(msg)) => {
                assert!(msg.contains("empty"));
            }
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    #[tokio::test]
    async fn test_get_series_invalid_id_zero() {
        let scraper = CsfdScraper::new().unwrap();
        let result = scraper.get_series(0).await;
        assert!(result.is_err());
        
        match result {
            Err(CsfdError::InvalidId(id)) => {
                assert_eq!(id, 0);
            }
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[tokio::test]
    async fn test_get_episodes_invalid_id_zero() {
        let scraper = CsfdScraper::new().unwrap();
        let result = scraper.get_episodes(0).await;
        assert!(result.is_err());
        
        match result {
            Err(CsfdError::InvalidId(id)) => {
                assert_eq!(id, 0);
            }
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[tokio::test]
    async fn test_get_season_episodes_invalid_series_id() {
        let scraper = CsfdScraper::new().unwrap();
        let result = scraper.get_season_episodes(0, 123).await;
        assert!(result.is_err());
        
        match result {
            Err(CsfdError::InvalidId(id)) => {
                assert_eq!(id, 0);
            }
            _ => panic!("Expected InvalidId error"),
        }
    }

    #[tokio::test]
    async fn test_get_season_episodes_invalid_season_id() {
        let scraper = CsfdScraper::new().unwrap();
        let result = scraper.get_season_episodes(123, 0).await;
        assert!(result.is_err());
        
        match result {
            Err(CsfdError::InvalidId(id)) => {
                assert_eq!(id, 0);
            }
            _ => panic!("Expected InvalidId error"),
        }
    }
}
