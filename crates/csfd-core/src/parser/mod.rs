//! HTML parsers for ČSFD.cz pages
//!
//! This module contains parsers for extracting data from ČSFD HTML pages:
//! - `search`: Parse search results page
//! - `series`: Parse series detail page
//! - `episodes`: Parse episodes list page

pub mod episodes;
pub mod search;
pub mod series;

// Re-export main parsing functions
pub use episodes::{parse_episode_code, parse_episodes, parse_rating};
pub use search::{extract_csfd_id, parse_search_results};
pub use series::{parse_seasons, parse_series_detail};
