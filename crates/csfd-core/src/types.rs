//! Data types for ČSFD Scraper
//!
//! This module contains all the core data structures used throughout the library.
//! All types implement Serialize and Deserialize for JSON compatibility with Tauri.

use serde::{Deserialize, Serialize};

/// Type of series/show on ČSFD
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeriesType {
    /// Regular TV series (seriál)
    Series,
    /// Single season (série)
    Season,
    /// Mini-series (minisérie)
    MiniSeries,
}

/// Search result item from ČSFD search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Display name of the series
    pub name: String,
    /// Original name (if different from Czech name)
    pub original_name: Option<String>,
    /// Year or year range (e.g., "2020" or "2020-2023")
    pub year: Option<String>,
    /// Type of the series
    pub series_type: SeriesType,
    /// Relative URL on ČSFD
    pub url: String,
    /// Unique ČSFD identifier
    pub csfd_id: u32,
}

/// Detailed information about a series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesDetail {
    /// Unique ČSFD identifier
    pub csfd_id: u32,
    /// Display name of the series
    pub name: String,
    /// Original name (if different from Czech name)
    pub original_name: Option<String>,
    /// Year range (e.g., "2020-2023" or "2020")
    pub year_range: Option<String>,
    /// List of genres
    pub genres: Vec<String>,
    /// List of countries of origin
    pub countries: Vec<String>,
    /// List of seasons
    pub seasons: Vec<Season>,
}

/// Season information within a series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    /// Unique ČSFD identifier for this season
    pub csfd_id: u32,
    /// Display name of the season
    pub name: String,
    /// Year of the season
    pub year: Option<String>,
    /// Number of episodes in this season
    pub episode_count: u32,
    /// Relative URL on ČSFD
    pub url: String,
}


/// Episode information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    /// Unique ČSFD identifier for this episode
    pub csfd_id: u32,
    /// Display name of the episode
    pub name: String,
    /// Episode code in format SxxExx (e.g., S01E01)
    pub episode_code: String,
    /// Season number (1-based)
    pub season_number: u8,
    /// Episode number within the season (1-based)
    pub episode_number: u8,
    /// Rating as percentage (0.0 - 100.0), None if not rated
    pub rating: Option<f32>,
    /// Relative URL on ČSFD
    pub url: String,
}

/// Paginated result wrapper for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    /// Items on the current page
    pub items: Vec<T>,
    /// Current page number (1-based)
    pub current_page: u32,
    /// Whether there are more pages available
    pub has_next_page: bool,
}

impl<T> PaginatedResult<T> {
    /// Create a new paginated result
    pub fn new(items: Vec<T>, current_page: u32, has_next_page: bool) -> Self {
        Self {
            items,
            current_page,
            has_next_page,
        }
    }

    /// Create an empty result for the first page
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            current_page: 1,
            has_next_page: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_type_serialization() {
        let series = SeriesType::Series;
        let json = serde_json::to_string(&series).unwrap();
        assert_eq!(json, "\"Series\"");

        let mini = SeriesType::MiniSeries;
        let json = serde_json::to_string(&mini).unwrap();
        assert_eq!(json, "\"MiniSeries\"");
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            name: "Test Series".to_string(),
            original_name: Some("Original Name".to_string()),
            year: Some("2020".to_string()),
            series_type: SeriesType::Series,
            url: "/film/123-test/".to_string(),
            csfd_id: 123,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SearchResult = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "Test Series");
        assert_eq!(deserialized.csfd_id, 123);
    }

    #[test]
    fn test_episode_rating_range() {
        let episode = Episode {
            csfd_id: 1,
            name: "Pilot".to_string(),
            episode_code: "S01E01".to_string(),
            season_number: 1,
            episode_number: 1,
            rating: Some(85.5),
            url: "/film/1-test/".to_string(),
        };

        assert!(episode.rating.unwrap() >= 0.0);
        assert!(episode.rating.unwrap() <= 100.0);
    }

    #[test]
    fn test_paginated_result_empty() {
        let result: PaginatedResult<SearchResult> = PaginatedResult::empty();
        assert!(result.items.is_empty());
        assert_eq!(result.current_page, 1);
        assert!(!result.has_next_page);
    }
}
