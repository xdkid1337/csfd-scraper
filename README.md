# ČSFD Scraper

A Rust library for scraping TV series data from [ČSFD.cz](https://www.csfd.cz) (Česko-Slovenská filmová databáze).

## Features

- 🔍 Search for TV series by name
- 📺 Get series details (name, year, genres, countries, seasons)
- 🎬 Fetch episode listings with ratings
- ⚡ Rate-limited HTTP client (respects server limits)
- 🔄 Automatic retry with exponential backoff
- 🖥️ Tauri 2.0 integration ready

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
csfd-core = { path = "crates/csfd-core" }
```

For Tauri integration:

```toml
[dependencies]
csfd-tauri = { path = "crates/csfd-tauri" }
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

## Project Structure

```
crates/
├── csfd-core/      # Core scraping library
└── csfd-tauri/     # Tauri 2.0 integration
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run example (live test against ČSFD.cz)
cargo run --example live_test -p csfd-core

# Format & lint
cargo fmt
cargo clippy
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Disclaimer

This project is an **unofficial, community-maintained scraper** for ČSFD.cz. It is **not affiliated with, endorsed, or sponsored by ČSFD** or any of its operators.

**⚠️ robots.txt Notice:** ČSFD's robots.txt disallows crawling of `/hledat` (search) endpoints. The search functionality in this library may violate their crawling policy. Film detail and episode pages are not restricted.

**You are solely responsible for:**
- Reviewing and complying with ČSFD.cz's Terms of Service and robots.txt
- Ensuring your use complies with all applicable laws in your jurisdiction
- Avoiding excessive traffic or accessing non-public data

The authors do not encourage or condone any misuse of this software. The code is provided "as is", without warranty of any kind. **Use at your own risk.**
