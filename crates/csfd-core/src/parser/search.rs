//! Search results parser for ČSFD.cz
//!
//! Parses HTML from search results pages to extract series information.

use scraper::{Html, Selector};

use crate::error::{CsfdError, Result};
use crate::types::{PaginatedResult, SearchResult, SeriesType};

/// Extract CSFD ID from a URL path.
///
/// Parses URLs in formats:
/// - `/film/{id}-{slug}/` -> Some(id)
/// - `/film/{id}-{slug}/prehled/` -> Some(id)
/// - `/film/{id}-{slug}/{season_id}-{season_slug}/` -> Some(id)
///
/// # Arguments
/// * `url` - URL path string to parse
///
/// # Returns
/// * `Some(id)` if a valid numeric ID is found
/// * `None` if the URL doesn't contain a valid ID
///
/// # Examples
/// ```
/// use csfd_core::parser::extract_csfd_id;
///
/// assert_eq!(extract_csfd_id("/film/12345-breaking-bad/"), Some(12345));
/// assert_eq!(extract_csfd_id("/film/999-test/prehled/"), Some(999));
/// assert_eq!(extract_csfd_id("invalid-url"), None);
/// ```
pub fn extract_csfd_id(url: &str) -> Option<u32> {
    // Look for pattern: /film/{id}-{slug}/ or similar
    // The ID is always the first numeric part after /film/
    
    // Find the /film/ prefix
    let film_idx = url.find("/film/")?;
    let after_film = &url[film_idx + 6..]; // Skip "/film/"
    
    // Find the first segment (up to next / or end)
    let segment = after_film.split('/').next()?;
    
    // Extract the numeric prefix before the first dash
    let id_str = segment.split('-').next()?;
    
    // Parse as u32, must be positive (non-zero)
    let id: u32 = id_str.parse().ok()?;
    
    if id > 0 {
        Some(id)
    } else {
        None
    }
}

/// Parse search results from ČSFD search page HTML.
///
/// # Arguments
/// * `html` - Raw HTML content of the search results page
///
/// # Returns
/// * `Ok(PaginatedResult<SearchResult>)` with parsed results
/// * `Err(CsfdError)` if parsing fails
pub fn parse_search_results(html: &str) -> Result<PaginatedResult<SearchResult>> {
    let document = Html::parse_document(html);
    
    // ČSFD search results are in article elements with class "article-poster-50"
    // or in older format with "ui-film-list"
    let results_selector = Selector::parse("article.article-poster-50, .ui-film-list .film-item")
        .map_err(|e| CsfdError::ParseError(format!("Invalid selector: {:?}", e)))?;
    
    let mut items = Vec::new();
    
    for element in document.select(&results_selector) {
        if let Some(result) = parse_search_item(&element) {
            items.push(result);
        }
    }
    
    // Check for pagination - look for "next page" link
    let has_next_page = detect_pagination(&document);
    
    // Extract current page from pagination if available
    let current_page = extract_current_page(&document).unwrap_or(1);
    
    Ok(PaginatedResult::new(items, current_page, has_next_page))
}

/// Parse a single search result item from an HTML element.
fn parse_search_item(element: &scraper::ElementRef) -> Option<SearchResult> {
    // Try to find the link element with the title
    // Current ČSFD uses "a.film-title-name" inside article elements
    let link_selector = Selector::parse("a.film-title-name, a.name, h3 a, .article-header a").ok()?;
    let link = element.select(&link_selector).next()?;
    
    // Get the URL from href attribute
    let url = link.value().attr("href")?.to_string();
    
    // Extract CSFD ID from URL
    let csfd_id = extract_csfd_id(&url)?;
    
    // Get the name from link text
    let name = link.text().collect::<String>().trim().to_string();
    if name.is_empty() {
        return None;
    }
    
    // Try to get original name (usually in parentheses or separate element)
    let original_name = extract_original_name(element);
    
    // Try to get year
    let year = extract_year(element);
    
    // Determine series type
    let series_type = extract_series_type(element);
    
    Some(SearchResult {
        name,
        original_name,
        year,
        series_type,
        url,
        csfd_id,
    })
}

/// Extract original name from search result element.
fn extract_original_name(element: &scraper::ElementRef) -> Option<String> {
    // Original name is often in a span with class "film-title-info" or similar
    let selectors = [
        ".film-title-info .info",
        ".origin-name",
        ".original-name",
        ".info span:first-child",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(el) = element.select(&selector).next() {
                let text = el.text().collect::<String>().trim().to_string();
                if !text.is_empty() && text != "-" {
                    return Some(text);
                }
            }
        }
    }
    
    None
}

/// Extract year from search result element.
fn extract_year(element: &scraper::ElementRef) -> Option<String> {
    // Year is often in a span with class containing "year" or in parentheses
    let selectors = [
        ".film-title-info .info",
        ".year",
        ".info-year",
        "span.year",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(el) = element.select(&selector).next() {
                let text = el.text().collect::<String>();
                // Look for year pattern (4 digits)
                if let Some(year) = extract_year_from_text(&text) {
                    return Some(year);
                }
            }
        }
    }
    
    // Also check the full element text for year pattern
    let full_text = element.text().collect::<String>();
    extract_year_from_text(&full_text)
}

/// Extract year pattern from text (e.g., "2020" or "2020-2023").
fn extract_year_from_text(text: &str) -> Option<String> {
    // Look for patterns like (2020) or (2020-2023)
    let re_range = regex_lite::Regex::new(r"\((\d{4}(?:-\d{4})?)\)").ok()?;
    if let Some(caps) = re_range.captures(text) {
        return Some(caps.get(1)?.as_str().to_string());
    }
    
    // Look for standalone year
    let re_year = regex_lite::Regex::new(r"\b((?:19|20)\d{2})\b").ok()?;
    if let Some(caps) = re_year.captures(text) {
        return Some(caps.get(1)?.as_str().to_string());
    }
    
    None
}

/// Determine series type from element.
fn extract_series_type(element: &scraper::ElementRef) -> SeriesType {
    let text = element.text().collect::<String>().to_lowercase();
    
    if text.contains("minisérie") || text.contains("miniserie") {
        SeriesType::MiniSeries
    } else if text.contains("série") && !text.contains("seriál") {
        SeriesType::Season
    } else {
        SeriesType::Series
    }
}

/// Detect if there are more pages of results.
fn detect_pagination(document: &Html) -> bool {
    // Look for "next page" link in pagination
    let next_selectors = [
        ".pagination .next:not(.disabled)",
        ".paging a.next",
        "a[rel='next']",
        ".pagination-next:not(.disabled)",
    ];
    
    for selector_str in &next_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if document.select(&selector).next().is_some() {
                return true;
            }
        }
    }
    
    false
}

/// Extract current page number from pagination.
fn extract_current_page(document: &Html) -> Option<u32> {
    let selectors = [
        ".pagination .active",
        ".paging .current",
        ".pagination-current",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(el) = document.select(&selector).next() {
                let text = el.text().collect::<String>();
                if let Ok(page) = text.trim().parse::<u32>() {
                    return Some(page);
                }
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_csfd_id_basic() {
        assert_eq!(extract_csfd_id("/film/12345-breaking-bad/"), Some(12345));
        assert_eq!(extract_csfd_id("/film/999-test/"), Some(999));
        assert_eq!(extract_csfd_id("/film/1-a/"), Some(1));
    }

    #[test]
    fn test_extract_csfd_id_with_subpath() {
        assert_eq!(extract_csfd_id("/film/12345-breaking-bad/prehled/"), Some(12345));
        assert_eq!(extract_csfd_id("/film/12345-breaking-bad/456-season-1/"), Some(12345));
    }

    #[test]
    fn test_extract_csfd_id_invalid() {
        assert_eq!(extract_csfd_id("invalid-url"), None);
        assert_eq!(extract_csfd_id("/film/"), None);
        assert_eq!(extract_csfd_id("/film/abc-test/"), None);
        assert_eq!(extract_csfd_id("/film/0-test/"), None);
        assert_eq!(extract_csfd_id(""), None);
    }

    #[test]
    fn test_extract_csfd_id_full_url() {
        assert_eq!(
            extract_csfd_id("https://www.csfd.cz/film/12345-breaking-bad/"),
            Some(12345)
        );
    }

    #[test]
    fn test_extract_year_from_text() {
        assert_eq!(extract_year_from_text("(2020)"), Some("2020".to_string()));
        assert_eq!(extract_year_from_text("(2020-2023)"), Some("2020-2023".to_string()));
        assert_eq!(extract_year_from_text("Some text 2020 more"), Some("2020".to_string()));
        assert_eq!(extract_year_from_text("no year here"), None);
    }

    #[test]
    fn test_parse_empty_html() {
        let result = parse_search_results("<html><body></body></html>").unwrap();
        assert!(result.items.is_empty());
        assert_eq!(result.current_page, 1);
        assert!(!result.has_next_page);
    }
}
