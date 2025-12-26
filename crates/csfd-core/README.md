# ČSFD Core

Core Rust library for scraping TV series data from [ČSFD.cz](https://www.csfd.cz) (Česko-Slovenská filmová databáze).

## Features

- 🔍 Search for TV series by name
- 📺 Get series details (name, year, genres, countries, seasons)
- 🎬 Fetch episode listings with ratings
- ⚡ Rate-limited HTTP client (respects server limits)
- 🔄 Automatic retry with exponential backoff

## Installation

```toml
[dependencies]
csfd-core = "0.1"
```

## Usage

```rust
use csfd_core::CsfdScraper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scraper = CsfdScraper::new()?;
    
    // Search for a series
    let results = scraper.search("Breaking Bad").await?;
    println!("Found {} results", results.items.len());
    
    // Get series details
    if let Some(series) = results.items.first() {
        let detail = scraper.get_series(series.csfd_id).await?;
        println!("{} ({:?})", detail.name, detail.year_range);
        
        // Get episodes
        let episodes = scraper.get_episodes(series.csfd_id).await?;
        for ep in episodes {
            println!("  {} - {}", ep.episode_code, ep.name);
        }
    }
    
    Ok(())
}
```

## License

MIT License

## Disclaimer

This is an **unofficial scraper** not affiliated with ČSFD.cz. You are responsible for complying with their Terms of Service. Use at your own risk.
