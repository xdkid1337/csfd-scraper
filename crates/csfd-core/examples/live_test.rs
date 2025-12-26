use csfd_core::CsfdScraper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scraper = CsfdScraper::new()?;
    
    println!("🔍 Hledám 'Teorie velkého třesku'...\n");
    
    let results = scraper.search("Teorie velkého třesku").await?;
    
    println!("Nalezeno {} výsledků:", results.items.len());
    for (i, item) in results.items.iter().enumerate() {
        println!("  {}. {} ({:?}) - ID: {}", i + 1, item.name, item.series_type, item.csfd_id);
    }
    
    // Najdeme hlavní seriál "Teorie velkého třesku" (ID 234260)
    let series = results.items.iter()
        .find(|r| r.name == "Teorie velkého třesku" && matches!(r.series_type, csfd_core::SeriesType::Series))
        .or_else(|| results.items.iter().find(|r| matches!(r.series_type, csfd_core::SeriesType::Series)))
        .or_else(|| results.items.first());
    
    if let Some(series) = series {
        println!("\n📺 Načítám detail seriálu: {} (ID: {})\n", series.name, series.csfd_id);
        
        let detail = scraper.get_series(series.csfd_id).await?;
        
        println!("Název: {}", detail.name);
        if let Some(orig) = &detail.original_name {
            println!("Originální název: {}", orig);
        }
        if let Some(years) = &detail.year_range {
            println!("Roky: {}", years);
        }
        println!("Žánry: {}", detail.genres.join(", "));
        println!("Země: {}", detail.countries.join(", "));
        
        println!("\n📋 Série ({}):", detail.seasons.len());
        for season in &detail.seasons {
            println!("  • {} - {} epizod (ID: {})", 
                season.name, 
                season.episode_count,
                season.csfd_id
            );
        }
        
        // Načteme epizody první série
        if let Some(first_season) = detail.seasons.first() {
            println!("\n🎬 Epizody série '{}' (ID: {}):\n", first_season.name, first_season.csfd_id);
            
            let episodes = scraper.get_season_episodes(series.csfd_id, first_season.csfd_id).await?;
            
            for ep in &episodes {
                let rating_str = ep.rating
                    .map(|r| format!("{:.0}%", r))
                    .unwrap_or_else(|| "—".to_string());
                println!("  {} {} [{}]", ep.episode_code, ep.name, rating_str);
            }
            
            println!("\nCelkem {} epizod v této sérii.", episodes.len());
        }
    }
    
    Ok(())
}
