# ČSFD Tauri

Tauri 2.0 integration for [csfd-core](https://crates.io/crates/csfd-core) - scraping TV series data from [ČSFD.cz](https://www.csfd.cz).

## Installation

```toml
[dependencies]
csfd-tauri = "0.1"
```

## Usage

Register the plugin in your Tauri app:

```rust
use csfd_tauri::ScraperState;

fn main() {
    tauri::Builder::default()
        .manage(ScraperState::new().expect("Failed to create scraper"))
        .invoke_handler(tauri::generate_handler![
            csfd_tauri::commands::search,
            csfd_tauri::commands::get_series,
            csfd_tauri::commands::get_episodes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Then call from your frontend:

```typescript
import { invoke } from '@tauri-apps/api/core';

// Search for series
const results = await invoke('search', { query: 'Breaking Bad' });

// Get series details
const series = await invoke('get_series', { csfdId: 12345 });

// Get episodes
const episodes = await invoke('get_episodes', { csfdId: 12345 });
```

## License

MIT License

## Disclaimer

This is an **unofficial scraper** not affiliated with ČSFD.cz. You are responsible for complying with their Terms of Service. Use at your own risk.
