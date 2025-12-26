//! ČSFD Scraper Tauri Integration
//!
//! This crate provides Tauri commands for integrating the ČSFD scraper
//! into Tauri 2.0 applications.
//!
//! # Usage
//!
//! ```rust,ignore
//! use csfd_tauri::ScraperState;
//! use tauri::Manager;
//!
//! fn main() {
//!     tauri::Builder::default()
//!         .setup(|app| {
//!             app.manage(ScraperState::new()?);
//!             Ok(())
//!         })
//!         .invoke_handler(tauri::generate_handler![
//!             csfd_tauri::commands::search_series,
//!             csfd_tauri::commands::search_series_page,
//!             csfd_tauri::commands::get_series_detail,
//!             csfd_tauri::commands::get_episodes,
//!             csfd_tauri::commands::get_season_episodes,
//!         ])
//!         .run(tauri::generate_context!())
//!         .expect("error while running tauri application");
//! }
//! ```
//!
//! # Commands
//! - `search_series` - Search for series by name
//! - `search_series_page` - Search with pagination
//! - `get_series_detail` - Get series details
//! - `get_episodes` - Get episode list
//! - `get_season_episodes` - Get episodes for a specific season

pub mod commands;

use std::sync::Arc;
use tokio::sync::Mutex;

use csfd_core::CsfdScraper;

/// Thread-safe wrapper for CsfdScraper.
///
/// This state is managed by Tauri and provides safe concurrent access
/// to the scraper from multiple commands.
///
/// # Example
/// ```rust,ignore
/// use csfd_tauri::ScraperState;
/// use tauri::Manager;
///
/// tauri::Builder::default()
///     .setup(|app| {
///         app.manage(ScraperState::new()?);
///         Ok(())
///     })
/// ```
pub struct ScraperState {
    scraper: Arc<Mutex<CsfdScraper>>,
}

impl ScraperState {
    /// Create a new ScraperState with default configuration.
    ///
    /// # Errors
    /// Returns an error string if the scraper cannot be created.
    pub fn new() -> Result<Self, String> {
        let scraper = CsfdScraper::new().map_err(|e| e.to_string())?;
        Ok(Self {
            scraper: Arc::new(Mutex::new(scraper)),
        })
    }

    /// Get a reference to the inner scraper.
    pub fn scraper(&self) -> &Arc<Mutex<CsfdScraper>> {
        &self.scraper
    }
}
