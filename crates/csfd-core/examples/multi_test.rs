use csfd_core::{CsfdScraper, SeriesType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scraper = CsfdScraper::new()?;
    
    // (search query, expected name substring)
    let test_series = [
        ("Teorie velkého třesku", "Teorie velkého třesku"),
        ("Perníkový táta", "Perníkový táta"),  // Breaking Bad in Czech
        ("Star Trek Picard", "Picard"),
        ("Doctor Who", "Doctor Who"),
    ];
    
    for (query, expected) in test_series {
        println!("\n{}", "=".repeat(60));
        println!("🔍 Hledám: {}", query);
        println!("{}\n", "=".repeat(60));
        
        let results = scraper.search(query).await?;
        
        if results.items.is_empty() {
            println!("❌ Žádné výsledky!");
            continue;
        }
        
        // Find the best matching series
        let series = results.items.iter()
            // Prefer exact name match with Series type
            .find(|r| matches!(r.series_type, SeriesType::Series) && 
                      r.name.to_lowercase() == expected.to_lowercase())
            // Then try contains match with Series type
            .or_else(|| results.items.iter().find(|r| 
                matches!(r.series_type, SeriesType::Series) && 
                r.name.to_lowercase().contains(&expected.to_lowercase())))
            // Then any Series type
            .or_else(|| results.items.iter().find(|r| matches!(r.series_type, SeriesType::Series)))
            // Fallback to first result
            .or_else(|| results.items.first());
        
        if let Some(series) = series {
            println!("📺 Vybrán: {} (ID: {})", series.name, series.csfd_id);
            
            let detail = scraper.get_series(series.csfd_id).await?;
            
            println!("\n📋 Detail:");
            println!("   Název: {}", detail.name);
            if let Some(orig) = &detail.original_name {
                println!("   Originální název: {}", orig);
            }
            if let Some(years) = &detail.year_range {
                println!("   Roky: {}", years);
            }
            if !detail.genres.is_empty() {
                println!("   Žánry: {}", detail.genres.join(", "));
            }
            if !detail.countries.is_empty() {
                println!("   Země: {}", detail.countries.join(", "));
            } else {
                println!("   Země: ⚠️ NENALEZENO");
            }
            
            println!("\n📺 Série ({}):", detail.seasons.len());
            for (i, season) in detail.seasons.iter().take(5).enumerate() {
                println!("   {}. {} - {} epizod", i + 1, season.name, season.episode_count);
            }
            if detail.seasons.len() > 5 {
                println!("   ... a dalších {} sérií", detail.seasons.len() - 5);
            }
            
            // Get episodes from first season
            if let Some(first_season) = detail.seasons.first() {
                println!("\n🎬 Epizody série '{}' (prvních 5):", first_season.name);
                let episodes = scraper.get_season_episodes(series.csfd_id, first_season.csfd_id).await?;
                
                for ep in episodes.iter().take(5) {
                    println!("   {} {}", ep.episode_code, ep.name);
                }
                if episodes.len() > 5 {
                    println!("   ... a dalších {} epizod", episodes.len() - 5);
                }
                println!("   Celkem: {} epizod", episodes.len());
            }
        }
        
        // Rate limiting pause
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    println!("\n\n✅ Test dokončen!");
    Ok(())
}
