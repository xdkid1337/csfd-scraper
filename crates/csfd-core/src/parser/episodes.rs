//! Episodes parser for ČSFD.cz
//!
//! Parses HTML from episodes list pages to extract episode information.

use scraper::{Html, Selector};

use crate::error::Result;
use crate::types::Episode;

use super::search::extract_csfd_id;

/// Parse episodes list from ČSFD episodes page HTML.
///
/// # Arguments
/// * `html` - Raw HTML content of the episodes page
///
/// # Returns
/// * `Ok(Vec<Episode>)` with parsed episodes
/// * `Err(CsfdError)` if parsing fails
pub fn parse_episodes(html: &str) -> Result<Vec<Episode>> {
    let document = Html::parse_document(html);
    let mut episodes = Vec::new();
    
    // Current ČSFD structure: episodes are in h3.film-title with a.film-title-name links
    // Similar to seasons structure
    if let Ok(selector) = Selector::parse("h3.film-title") {
        for h3 in document.select(&selector) {
            if let Some(episode) = parse_episode_from_h3(&h3) {
                episodes.push(episode);
            }
        }
    }
    
    if !episodes.is_empty() {
        return Ok(episodes);
    }
    
    // Fallback: try different selectors for episodes table/list
    let container_selectors = [
        ".film-episodes table tbody",
        ".episodes-list",
        "table.episodes tbody",
        ".box-content table tbody",
    ];
    
    for container_sel in &container_selectors {
        if let Ok(container_selector) = Selector::parse(container_sel) {
            for container in document.select(&container_selector) {
                if let Ok(row_selector) = Selector::parse("tr") {
                    let mut current_season: u8 = 1;
                    
                    for row in container.select(&row_selector) {
                        // Check if this is a season header row
                        if let Some(season_num) = extract_season_header(&row) {
                            current_season = season_num;
                            continue;
                        }
                        
                        // Try to parse as episode row
                        if let Some(episode) = parse_episode_row(&row, current_season) {
                            episodes.push(episode);
                        }
                    }
                    
                    if !episodes.is_empty() {
                        return Ok(episodes);
                    }
                }
            }
        }
    }
    
    // Alternative: look for episode links directly
    if let Ok(selector) = Selector::parse(".episode-item, .film-episodes a[href*='/film/']") {
        let mut current_season: u8 = 1;
        
        for el in document.select(&selector) {
            if let Some(episode) = parse_episode_element(&el, current_season) {
                // Update season from episode code if available
                current_season = episode.season_number;
                episodes.push(episode);
            }
        }
    }
    
    Ok(episodes)
}

/// Parse episode from h3.film-title element (current ČSFD structure).
fn parse_episode_from_h3(h3: &scraper::ElementRef) -> Option<Episode> {
    // Find the link inside h3
    let link_selector = Selector::parse("a.film-title-name").ok()?;
    let link = h3.select(&link_selector).next()?;
    
    // Get URL
    let url = link.value().attr("href")?.to_string();
    
    // Extract CSFD ID
    let csfd_id = extract_episode_id(&url)?;
    
    // Get name from link text
    let name = link.text().collect::<String>().trim().to_string();
    if name.is_empty() {
        return None;
    }
    
    // Get info from span.film-title-info - contains episode code like "(S01E01)"
    let info_selector = Selector::parse(".film-title-info").ok()?;
    let info_text = h3.select(&info_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default();
    
    // Extract episode code from info
    let (season_number, episode_number) = parse_episode_code(&info_text)
        .unwrap_or((1, 1));
    
    let episode_code = format!("S{:02}E{:02}", season_number, episode_number);
    
    // Rating is not typically shown in the episode list on ČSFD
    let rating = None;
    
    Some(Episode {
        csfd_id,
        name,
        episode_code,
        season_number,
        episode_number,
        rating,
        url,
    })
}

/// Extract episode ID from URL (the episode part of the path).
fn extract_episode_id(url: &str) -> Option<u32> {
    // URL format: /film/{series_id}-{slug}/{episode_id}-{episode_slug}/prehled/
    let parts: Vec<&str> = url.trim_matches('/').split('/').collect();
    
    // Find the film segment and look for the episode segment
    for (i, part) in parts.iter().enumerate() {
        if *part == "film" && i + 2 < parts.len() {
            // The episode ID is in parts[i+2]
            let episode_segment = parts[i + 2];
            if let Some(id_str) = episode_segment.split('-').next() {
                if let Ok(id) = id_str.parse::<u32>() {
                    if id > 0 {
                        return Some(id);
                    }
                }
            }
        }
    }
    
    None
}

/// Check if a row is a season header and extract season number.
fn extract_season_header(row: &scraper::ElementRef) -> Option<u8> {
    // Season headers often have colspan or special class
    if let Ok(selector) = Selector::parse("th[colspan], td.season-header, .season-title") {
        if row.select(&selector).next().is_some() {
            let text = row.text().collect::<String>();
            return extract_season_number_from_text(&text);
        }
    }
    
    // Check for "Série X" or "Season X" pattern in row text
    let text = row.text().collect::<String>().to_lowercase();
    if text.contains("série") || text.contains("season") || text.contains("řada") {
        return extract_season_number_from_text(&text);
    }
    
    None
}

/// Extract season number from text like "Série 1" or "Season 2".
fn extract_season_number_from_text(text: &str) -> Option<u8> {
    let re = regex_lite::Regex::new(r"(?i)(?:série|season|řada|s)\s*(\d+)").ok()?;
    if let Some(caps) = re.captures(text) {
        return caps.get(1)?.as_str().parse().ok();
    }
    None
}

/// Parse a single episode from a table row.
fn parse_episode_row(row: &scraper::ElementRef, default_season: u8) -> Option<Episode> {
    // Find the episode link
    let link_selector = Selector::parse("a[href*='/film/']").ok()?;
    let link = row.select(&link_selector).next()?;
    
    // Get URL
    let url = link.value().attr("href")?.to_string();
    
    // Extract CSFD ID
    let csfd_id = extract_csfd_id(&url)?;
    
    // Get episode name
    let name = link.text().collect::<String>().trim().to_string();
    if name.is_empty() {
        return None;
    }
    
    // Try to find episode code in the row
    let row_text = row.text().collect::<String>();
    let (season_number, episode_number) = parse_episode_code(&row_text)
        .or_else(|| parse_episode_code(&name))
        .unwrap_or((default_season, 0));
    
    // If we couldn't find episode number, try to extract from position or name
    let episode_number = if episode_number == 0 {
        extract_episode_number_from_name(&name).unwrap_or(1)
    } else {
        episode_number
    };
    
    // Format episode code
    let episode_code = format!("S{:02}E{:02}", season_number, episode_number);
    
    // Try to find rating
    let rating = extract_rating_from_row(row);
    
    Some(Episode {
        csfd_id,
        name: clean_episode_name(&name),
        episode_code,
        season_number,
        episode_number,
        rating,
        url,
    })
}

/// Parse a single episode from a generic element.
fn parse_episode_element(element: &scraper::ElementRef, default_season: u8) -> Option<Episode> {
    // Get URL - either from href or find a link inside
    let url = element.value().attr("href").map(|s| s.to_string()).or_else(|| {
        let link_selector = Selector::parse("a[href*='/film/']").ok()?;
        element.select(&link_selector).next()?.value().attr("href").map(|s| s.to_string())
    })?;
    
    // Extract CSFD ID
    let csfd_id = extract_csfd_id(&url)?;
    
    // Get episode name
    let name = element.text().collect::<String>().trim().to_string();
    if name.is_empty() {
        return None;
    }
    
    // Try to find episode code
    let (season_number, episode_number) = parse_episode_code(&name)
        .unwrap_or((default_season, extract_episode_number_from_name(&name).unwrap_or(1)));
    
    let episode_code = format!("S{:02}E{:02}", season_number, episode_number);
    
    // Try to find rating
    let rating = extract_rating_from_element(element);
    
    Some(Episode {
        csfd_id,
        name: clean_episode_name(&name),
        episode_code,
        season_number,
        episode_number,
        rating,
        url,
    })
}

/// Parse episode code from text in format SxxExx or similar.
///
/// # Arguments
/// * `text` - Text that may contain episode code
///
/// # Returns
/// * `Some((season, episode))` if a valid code is found
/// * `None` if no valid code is found
///
/// # Examples
/// ```
/// use csfd_core::parser::parse_episode_code;
///
/// assert_eq!(parse_episode_code("S01E05"), Some((1, 5)));
/// assert_eq!(parse_episode_code("Episode S02E10 - Title"), Some((2, 10)));
/// assert_eq!(parse_episode_code("no code here"), None);
/// ```
pub fn parse_episode_code(text: &str) -> Option<(u8, u8)> {
    // Pattern: S01E01, s01e01, S1E1, etc.
    let re = regex_lite::Regex::new(r"(?i)S(\d{1,2})E(\d{1,2})").ok()?;
    if let Some(caps) = re.captures(text) {
        let season: u8 = caps.get(1)?.as_str().parse().ok()?;
        let episode: u8 = caps.get(2)?.as_str().parse().ok()?;
        return Some((season, episode));
    }
    
    // Alternative pattern: 1x05, 01x05
    let re_alt = regex_lite::Regex::new(r"(\d{1,2})x(\d{1,2})").ok()?;
    if let Some(caps) = re_alt.captures(text) {
        let season: u8 = caps.get(1)?.as_str().parse().ok()?;
        let episode: u8 = caps.get(2)?.as_str().parse().ok()?;
        return Some((season, episode));
    }
    
    None
}

/// Extract episode number from name like "1. Episode Title" or "Episode 5".
fn extract_episode_number_from_name(name: &str) -> Option<u8> {
    // Pattern: "1. Title" or "01. Title"
    let re = regex_lite::Regex::new(r"^(\d{1,2})\.\s").ok()?;
    if let Some(caps) = re.captures(name) {
        return caps.get(1)?.as_str().parse().ok();
    }
    
    // Pattern: "Episode 5" or "Epizoda 5"
    let re_ep = regex_lite::Regex::new(r"(?i)(?:episode|epizoda|díl)\s*(\d{1,2})").ok()?;
    if let Some(caps) = re_ep.captures(name) {
        return caps.get(1)?.as_str().parse().ok();
    }
    
    None
}

/// Parse rating from text as percentage (0.0 - 100.0).
///
/// # Arguments
/// * `text` - Text that may contain rating
///
/// # Returns
/// * `Some(rating)` if a valid rating is found (0.0 - 100.0)
/// * `None` if no valid rating is found
///
/// # Examples
/// ```
/// use csfd_core::parser::parse_rating;
///
/// assert_eq!(parse_rating("85%"), Some(85.0));
/// assert_eq!(parse_rating("Rating: 72.5%"), Some(72.5));
/// assert_eq!(parse_rating("no rating"), None);
/// ```
pub fn parse_rating(text: &str) -> Option<f32> {
    // Pattern: 85%, 72.5%, etc.
    let re = regex_lite::Regex::new(r"(\d{1,3}(?:\.\d+)?)\s*%").ok()?;
    if let Some(caps) = re.captures(text) {
        let rating: f32 = caps.get(1)?.as_str().parse().ok()?;
        // Validate range
        if (0.0..=100.0).contains(&rating) {
            return Some(rating);
        }
    }
    
    None
}

/// Extract rating from a table row.
fn extract_rating_from_row(row: &scraper::ElementRef) -> Option<f32> {
    // Look for rating in specific cells
    let selectors = [
        ".rating",
        ".film-rating",
        "td:last-child",
        ".stars",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(el) = row.select(&selector).next() {
                let text = el.text().collect::<String>();
                if let Some(rating) = parse_rating(&text) {
                    return Some(rating);
                }
            }
        }
    }
    
    // Try the whole row text
    let row_text = row.text().collect::<String>();
    parse_rating(&row_text)
}

/// Extract rating from an element.
fn extract_rating_from_element(element: &scraper::ElementRef) -> Option<f32> {
    let selectors = [
        ".rating",
        ".film-rating",
        ".stars",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(el) = element.select(&selector).next() {
                let text = el.text().collect::<String>();
                if let Some(rating) = parse_rating(&text) {
                    return Some(rating);
                }
            }
        }
    }
    
    // Try the whole element text
    let text = element.text().collect::<String>();
    parse_rating(&text)
}

/// Clean episode name by removing episode code prefix.
fn clean_episode_name(name: &str) -> String {
    // Remove patterns like "S01E01 - " or "1. " from the beginning
    let re = regex_lite::Regex::new(r"^(?:S\d{1,2}E\d{1,2}\s*[-:]\s*|\d{1,2}\.\s*)").unwrap();
    re.replace(name, "").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_episode_code_standard() {
        assert_eq!(parse_episode_code("S01E05"), Some((1, 5)));
        assert_eq!(parse_episode_code("S02E10"), Some((2, 10)));
        assert_eq!(parse_episode_code("s1e1"), Some((1, 1)));
        assert_eq!(parse_episode_code("S12E99"), Some((12, 99)));
    }

    #[test]
    fn test_parse_episode_code_in_text() {
        assert_eq!(parse_episode_code("Episode S01E05 - Pilot"), Some((1, 5)));
        assert_eq!(parse_episode_code("Breaking Bad S05E16"), Some((5, 16)));
    }

    #[test]
    fn test_parse_episode_code_alternative() {
        assert_eq!(parse_episode_code("1x05"), Some((1, 5)));
        assert_eq!(parse_episode_code("02x10"), Some((2, 10)));
    }

    #[test]
    fn test_parse_episode_code_invalid() {
        assert_eq!(parse_episode_code("no code"), None);
        assert_eq!(parse_episode_code("Episode 5"), None);
        assert_eq!(parse_episode_code(""), None);
    }

    #[test]
    fn test_parse_rating_percentage() {
        assert_eq!(parse_rating("85%"), Some(85.0));
        assert_eq!(parse_rating("72.5%"), Some(72.5));
        assert_eq!(parse_rating("100%"), Some(100.0));
        assert_eq!(parse_rating("0%"), Some(0.0));
    }

    #[test]
    fn test_parse_rating_in_text() {
        assert_eq!(parse_rating("Rating: 85%"), Some(85.0));
        assert_eq!(parse_rating("Score 72.5 %"), Some(72.5));
    }

    #[test]
    fn test_parse_rating_invalid() {
        assert_eq!(parse_rating("no rating"), None);
        assert_eq!(parse_rating("150%"), None); // Out of range
        assert_eq!(parse_rating(""), None);
    }

    #[test]
    fn test_parse_rating_edge_cases() {
        assert_eq!(parse_rating("0.5%"), Some(0.5));
        assert_eq!(parse_rating("99.9%"), Some(99.9));
    }

    #[test]
    fn test_extract_episode_number_from_name() {
        assert_eq!(extract_episode_number_from_name("1. Pilot"), Some(1));
        assert_eq!(extract_episode_number_from_name("05. Episode Title"), Some(5));
        assert_eq!(extract_episode_number_from_name("Episode 3"), Some(3));
        assert_eq!(extract_episode_number_from_name("Epizoda 7"), Some(7));
        assert_eq!(extract_episode_number_from_name("Just a title"), None);
    }

    #[test]
    fn test_clean_episode_name() {
        assert_eq!(clean_episode_name("S01E01 - Pilot"), "Pilot");
        assert_eq!(clean_episode_name("1. First Episode"), "First Episode");
        assert_eq!(clean_episode_name("S02E05: The Title"), "The Title");
        assert_eq!(clean_episode_name("Just a title"), "Just a title");
    }

    #[test]
    fn test_parse_episodes_empty() {
        let result = parse_episodes("<html><body></body></html>").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_extract_season_number_from_text() {
        assert_eq!(extract_season_number_from_text("Season 1"), Some(1));
        assert_eq!(extract_season_number_from_text("Season 2"), Some(2));
        assert_eq!(extract_season_number_from_text("S3"), Some(3));
        assert_eq!(extract_season_number_from_text("no season"), None);
    }
}
