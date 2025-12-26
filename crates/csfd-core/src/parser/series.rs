//! Series detail parser for ČSFD.cz
//!
//! Parses HTML from series detail pages to extract series information and seasons.

use scraper::{Html, Selector};

use crate::error::{CsfdError, Result};
use crate::types::{Season, SeriesDetail};

use super::search::extract_csfd_id;

/// Parse series detail from ČSFD series page HTML.
///
/// # Arguments
/// * `html` - Raw HTML content of the series detail page
/// * `csfd_id` - The CSFD ID of the series (used in the result)
///
/// # Returns
/// * `Ok(SeriesDetail)` with parsed series information
/// * `Err(CsfdError)` if parsing fails
pub fn parse_series_detail(html: &str, csfd_id: u32) -> Result<SeriesDetail> {
    let document = Html::parse_document(html);
    
    // Extract series name
    let name = extract_series_name(&document)
        .ok_or_else(|| CsfdError::ElementNotFound("series name".to_string()))?;
    
    // Extract original name (optional)
    let original_name = extract_original_name(&document);
    
    // Extract year range (optional)
    let year_range = extract_year_range(&document);
    
    // Extract genres
    let genres = extract_genres(&document);
    
    // Extract countries
    let countries = extract_countries(&document);
    
    // Extract seasons
    let seasons = parse_seasons(&document);
    
    Ok(SeriesDetail {
        csfd_id,
        name,
        original_name,
        year_range,
        genres,
        countries,
        seasons,
    })
}

/// Extract series name from the page.
fn extract_series_name(document: &Html) -> Option<String> {
    let selectors = [
        "h1.film-header-name",
        ".film-header h1",
        "h1[itemprop='name']",
        ".movie-title h1",
        "h1",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(el) = document.select(&selector).next() {
                let text = el.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }
    
    None
}

/// Extract original name from the page.
fn extract_original_name(document: &Html) -> Option<String> {
    // First try: look for original name in film-names list (first item with USA flag)
    if let Ok(selector) = Selector::parse("ul.film-names li:first-child") {
        if let Some(li) = document.select(&selector).next() {
            // Get text after the flag image, clean up whitespace and "(více)"
            let text = li.text().collect::<String>();
            let cleaned = text
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.contains("(více)"))
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string();
            if !cleaned.is_empty() {
                return Some(cleaned);
            }
        }
    }
    
    // Fallback selectors
    let selectors = [
        ".film-header-name .film-header-origin-name",
        ".origin-name",
        "[itemprop='alternateName']",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(el) = document.select(&selector).next() {
                let text = el.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }
    
    None
}

/// Extract year range from the page.
fn extract_year_range(document: &Html) -> Option<String> {
    let selectors = [
        ".film-header-origin .origin span",
        ".origin .year",
        "[itemprop='datePublished']",
        ".film-info .origin",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for el in document.select(&selector) {
                let text = el.text().collect::<String>();
                if let Some(year) = extract_year_pattern(&text) {
                    return Some(year);
                }
            }
        }
    }
    
    None
}

/// Extract year pattern from text.
fn extract_year_pattern(text: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"(\d{4}(?:\s*[-–]\s*\d{4})?|\d{4}\s*[-–]\s*)").ok()?;
    if let Some(caps) = re.captures(text) {
        let year = caps.get(1)?.as_str().trim().to_string();
        // Clean up whitespace around dash
        let cleaned = year.replace(" - ", "-").replace(" – ", "-");
        return Some(cleaned);
    }
    None
}

/// Extract genres from the page.
fn extract_genres(document: &Html) -> Vec<String> {
    let mut genres = Vec::new();
    
    let selectors = [
        ".film-header-origin .genre a",
        ".genres a",
        "[itemprop='genre']",
        ".film-info .genre a",
    ];
    
    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for el in document.select(&selector) {
                let text = el.text().collect::<String>().trim().to_string();
                if !text.is_empty() && !genres.contains(&text) {
                    genres.push(text);
                }
            }
            if !genres.is_empty() {
                break;
            }
        }
    }
    
    genres
}

/// Extract countries from the page.
fn extract_countries(document: &Html) -> Vec<String> {
    let mut countries = Vec::new();
    
    // First try to find country links
    let link_selectors = [
        ".film-header-origin .origin a",
        ".origin .country a",
        "[itemprop='countryOfOrigin']",
        ".film-info .origin a",
    ];
    
    for selector_str in &link_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for el in document.select(&selector) {
                let text = el.text().collect::<String>().trim().to_string();
                if !text.is_empty() && !countries.contains(&text) {
                    countries.push(text);
                }
            }
            if !countries.is_empty() {
                return countries;
            }
        }
    }
    
    // Fallback: extract from div.origin text directly
    // Format: "USA, 2007-2019, 279 epizod" or just "USA,"
    if let Ok(selector) = Selector::parse("div.origin") {
        if let Some(el) = document.select(&selector).next() {
            let text = el.text().collect::<String>();
            // Country is usually the first part before comma
            if let Some(country_part) = text.split(',').next() {
                let country = country_part.trim().to_string();
                if !country.is_empty() && !country.chars().all(|c| c.is_numeric()) {
                    countries.push(country);
                }
            }
        }
    }
    
    countries
}

/// Parse seasons list from series detail page.
///
/// # Arguments
/// * `document` - Parsed HTML document
///
/// # Returns
/// Vector of Season objects found on the page
pub fn parse_seasons(document: &Html) -> Vec<Season> {
    let mut seasons = Vec::new();
    
    // Current ČSFD structure: seasons are in h3.film-title with a.film-title-name links
    // URL format: /film/{series_id}-{slug}/{season_id}-{season_slug}/prehled/
    // Text format: "Série 1" with info span "(2007) - 17 epizod"
    
    // First try the current ČSFD structure
    if let Ok(selector) = Selector::parse("h3.film-title") {
        for h3 in document.select(&selector) {
            if let Some(season) = parse_season_from_h3(&h3) {
                if !seasons.iter().any(|s: &Season| s.csfd_id == season.csfd_id) {
                    seasons.push(season);
                }
            }
        }
    }
    
    if !seasons.is_empty() {
        return seasons;
    }
    
    // Fallback: try different selectors for seasons list
    let container_selectors = [
        ".film-episodes-list",
        ".seasons-list",
        ".series-seasons",
        ".box-content ul",
    ];
    
    let item_selectors = [
        "li a",
        ".season-item a",
        "a.season-link",
    ];
    
    for container_sel in &container_selectors {
        if let Ok(container_selector) = Selector::parse(container_sel) {
            for container in document.select(&container_selector) {
                for item_sel in &item_selectors {
                    if let Ok(item_selector) = Selector::parse(item_sel) {
                        for item in container.select(&item_selector) {
                            if let Some(season) = parse_season_item(&item) {
                                if !seasons.iter().any(|s: &Season| s.csfd_id == season.csfd_id) {
                                    seasons.push(season);
                                }
                            }
                        }
                        if !seasons.is_empty() {
                            return seasons;
                        }
                    }
                }
            }
        }
    }
    
    // Alternative: look for season links directly
    if let Ok(selector) = Selector::parse("a[href*='/film/'][href*='serie']") {
        for el in document.select(&selector) {
            if let Some(season) = parse_season_item(&el) {
                if !seasons.iter().any(|s: &Season| s.csfd_id == season.csfd_id) {
                    seasons.push(season);
                }
            }
        }
    }
    
    seasons
}

/// Parse season from h3.film-title element (current ČSFD structure).
fn parse_season_from_h3(h3: &scraper::ElementRef) -> Option<Season> {
    // Find the link inside h3
    let link_selector = Selector::parse("a.film-title-name").ok()?;
    let link = h3.select(&link_selector).next()?;
    
    // Get URL
    let url = link.value().attr("href")?.to_string();
    
    // Extract season ID from URL
    let csfd_id = extract_season_id(&url)?;
    
    // Get name from link text
    let name = link.text().collect::<String>().trim().to_string();
    if name.is_empty() {
        return None;
    }
    
    // Get info from span.film-title-info
    let info_selector = Selector::parse(".film-title-info").ok()?;
    let info_text = h3.select(&info_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default();
    
    // Extract year from info (e.g., "(2007)")
    let year = extract_year_from_season_name(&info_text);
    
    // Extract episode count from info (e.g., "17 epizod")
    let episode_count = extract_episode_count_from_info(&info_text).unwrap_or(0);
    
    Some(Season {
        csfd_id,
        name,
        year,
        episode_count,
        url,
    })
}

/// Extract episode count from info text like "(2007) - 17 epizod".
fn extract_episode_count_from_info(text: &str) -> Option<u32> {
    let re = regex_lite::Regex::new(r"(\d+)\s*epizod").ok()?;
    if let Some(caps) = re.captures(text) {
        return caps.get(1)?.as_str().parse().ok();
    }
    None
}

/// Parse a single season item from an element.
fn parse_season_item(element: &scraper::ElementRef) -> Option<Season> {
    // Get URL
    let url = element.value().attr("href")?.to_string();
    
    // Extract CSFD ID - for seasons, we need to look at the season part of the URL
    let csfd_id = extract_season_id(&url).or_else(|| extract_csfd_id(&url))?;
    
    // Get name from text
    let name = element.text().collect::<String>().trim().to_string();
    if name.is_empty() {
        return None;
    }
    
    // Try to extract year from name or nearby elements
    let year = extract_year_from_season_name(&name);
    
    // Try to extract episode count (often in parentheses like "(10 epizod)")
    let episode_count = extract_episode_count(&name).unwrap_or(0);
    
    Some(Season {
        csfd_id,
        name: clean_season_name(&name),
        year,
        episode_count,
        url,
    })
}

/// Extract season ID from URL (the second ID in the path).
fn extract_season_id(url: &str) -> Option<u32> {
    // URL format: /film/{series_id}-{slug}/{season_id}-{season_slug}/
    let parts: Vec<&str> = url.trim_matches('/').split('/').collect();
    
    // Find the film segment and look for the next segment
    for (i, part) in parts.iter().enumerate() {
        if *part == "film" && i + 2 < parts.len() {
            // The season ID is in parts[i+2]
            let season_segment = parts[i + 2];
            if let Some(id_str) = season_segment.split('-').next() {
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

/// Extract year from season name.
fn extract_year_from_season_name(name: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"\((\d{4})\)").ok()?;
    if let Some(caps) = re.captures(name) {
        return Some(caps.get(1)?.as_str().to_string());
    }
    None
}

/// Extract episode count from season name.
fn extract_episode_count(name: &str) -> Option<u32> {
    // Look for patterns like "(10 epizod)" or "(8)"
    let re = regex_lite::Regex::new(r"\((\d+)(?:\s*epizod[ay]?)?\)").ok()?;
    if let Some(caps) = re.captures(name) {
        return caps.get(1)?.as_str().parse().ok();
    }
    None
}

/// Clean season name by removing year and episode count.
fn clean_season_name(name: &str) -> String {
    let re = regex_lite::Regex::new(r"\s*\([^)]*\)\s*").unwrap();
    re.replace_all(name, " ").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_year_pattern() {
        assert_eq!(extract_year_pattern("2020"), Some("2020".to_string()));
        assert_eq!(extract_year_pattern("2020-2023"), Some("2020-2023".to_string()));
        assert_eq!(extract_year_pattern("2020 - 2023"), Some("2020-2023".to_string()));
        assert_eq!(extract_year_pattern("no year"), None);
    }

    #[test]
    fn test_extract_season_id() {
        assert_eq!(
            extract_season_id("/film/12345-breaking-bad/456-season-1/"),
            Some(456)
        );
        assert_eq!(
            extract_season_id("/film/12345-test/789-serie-2/prehled/"),
            Some(789)
        );
        assert_eq!(extract_season_id("/film/12345-test/"), None);
    }

    #[test]
    fn test_extract_year_from_season_name() {
        assert_eq!(
            extract_year_from_season_name("Série 1 (2020)"),
            Some("2020".to_string())
        );
        assert_eq!(extract_year_from_season_name("Série 1"), None);
    }

    #[test]
    fn test_extract_episode_count() {
        assert_eq!(extract_episode_count("Série 1 (10 epizod)"), Some(10));
        assert_eq!(extract_episode_count("Série 1 (8)"), Some(8));
        assert_eq!(extract_episode_count("Série 1"), None);
    }

    #[test]
    fn test_clean_season_name() {
        assert_eq!(clean_season_name("Série 1 (2020)"), "Série 1");
        assert_eq!(clean_season_name("Série 1 (10 epizod)"), "Série 1");
        assert_eq!(clean_season_name("Série 1"), "Série 1");
    }

    #[test]
    fn test_parse_series_detail_minimal() {
        let html = r#"
            <html>
            <body>
                <h1 class="film-header-name">Breaking Bad</h1>
            </body>
            </html>
        "#;
        
        let result = parse_series_detail(html, 12345).unwrap();
        assert_eq!(result.name, "Breaking Bad");
        assert_eq!(result.csfd_id, 12345);
        assert!(result.seasons.is_empty());
    }
}
