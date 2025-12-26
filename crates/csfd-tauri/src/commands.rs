//! Tauri commands for ČSFD Scraper
//!
//! This module contains all Tauri commands that can be invoked from the frontend.

use tauri::State;

use crate::ScraperState;
use csfd_core::{Episode, PaginatedResult, SearchResult, SeriesDetail};

/// Search for series by name.
///
/// Returns the first page of search results.
///
/// # Arguments
/// * `query` - Search query string
///
/// # Returns
/// * `Ok(PaginatedResult<SearchResult>)` with matching series
/// * `Err(String)` with error message if search fails
#[tauri::command]
pub async fn search_series(
    state: State<'_, ScraperState>,
    query: String,
) -> Result<PaginatedResult<SearchResult>, String> {
    let scraper = state.scraper().lock().await;
    scraper.search(&query).await.map_err(|e| e.to_string())
}

/// Search for series by name with pagination.
///
/// # Arguments
/// * `query` - Search query string
/// * `page` - Page number (1-based)
///
/// # Returns
/// * `Ok(PaginatedResult<SearchResult>)` with matching series
/// * `Err(String)` with error message if search fails
#[tauri::command]
pub async fn search_series_page(
    state: State<'_, ScraperState>,
    query: String,
    page: u32,
) -> Result<PaginatedResult<SearchResult>, String> {
    let scraper = state.scraper().lock().await;
    scraper
        .search_page(&query, page)
        .await
        .map_err(|e| e.to_string())
}

/// Get detailed information about a series.
///
/// # Arguments
/// * `csfd_id` - ČSFD ID of the series
///
/// # Returns
/// * `Ok(SeriesDetail)` with series information and seasons
/// * `Err(String)` with error message if retrieval fails
#[tauri::command]
pub async fn get_series_detail(
    state: State<'_, ScraperState>,
    csfd_id: u32,
) -> Result<SeriesDetail, String> {
    let scraper = state.scraper().lock().await;
    scraper.get_series(csfd_id).await.map_err(|e| e.to_string())
}

/// Get all episodes for a series.
///
/// # Arguments
/// * `csfd_id` - ČSFD ID of the series
///
/// # Returns
/// * `Ok(Vec<Episode>)` with all episodes
/// * `Err(String)` with error message if retrieval fails
#[tauri::command]
pub async fn get_episodes(
    state: State<'_, ScraperState>,
    csfd_id: u32,
) -> Result<Vec<Episode>, String> {
    let scraper = state.scraper().lock().await;
    scraper
        .get_episodes(csfd_id)
        .await
        .map_err(|e| e.to_string())
}

/// Get episodes for a specific season.
///
/// # Arguments
/// * `series_id` - ČSFD ID of the series
/// * `season_id` - ČSFD ID of the season
///
/// # Returns
/// * `Ok(Vec<Episode>)` with episodes from the specified season
/// * `Err(String)` with error message if retrieval fails
#[tauri::command]
pub async fn get_season_episodes(
    state: State<'_, ScraperState>,
    series_id: u32,
    season_id: u32,
) -> Result<Vec<Episode>, String> {
    let scraper = state.scraper().lock().await;
    scraper
        .get_season_episodes(series_id, season_id)
        .await
        .map_err(|e| e.to_string())
}
