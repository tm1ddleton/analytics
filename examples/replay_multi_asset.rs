//! Multi-Asset Replay Example
//!
//! This example demonstrates replaying multiple assets chronologically:
//! 1. Creates sample data for AAPL, MSFT, and GOOG
//! 2. Replays all three assets in chronological order
//! 3. Tracks which asset is being processed
//! 4. Calculates and displays analytics for each asset
//!
//! Run with: cargo run --example replay_multi_asset

use analytics::{
    analytics::calculators::{StdDevVolatilityAnalytic, VolatilityAnalytic},
    AssetKey, DateRange, InMemoryDataProvider, ReplayEngine, TimeSeriesPoint,
};
use chrono::{NaiveDate, TimeZone, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

    println!("=== Multi-Asset Chronological Replay Example ===\n");

    // 1. Create sample data for multiple assets
    let tickers = vec!["AAPL", "MSFT", "GOOG"];
    let base_prices = vec![150.0, 300.0, 2800.0];

    println!(
        "Creating sample data for {} assets (3 months each)...",
        tickers.len()
    );

    let mut provider = InMemoryDataProvider::new();
    let mut assets = Vec::new();

    for (idx, &ticker) in tickers.iter().enumerate() {
        let asset = AssetKey::new_equity(ticker)?;
        assets.push(asset.clone());

        let mut test_data = Vec::new();
        let mut base_price = base_prices[idx];

        // Generate data with different patterns for each asset
        for month in 1u32..=3 {
            let days_in_month = match month {
                2 => 19, // February
                _ => 21,
            };

            for day in 1u32..=days_in_month {
                // Each asset has slightly different price movement
                let variation = ((day * (idx as u32 + 3) + month * 7) % 15) as f64 - 7.0;
                base_price += variation * (0.3 + idx as f64 * 0.1);

                test_data.push(TimeSeriesPoint::new(
                    Utc.with_ymd_and_hms(2024, month, day, 0, 0, 0).unwrap(),
                    base_price,
                ));
            }
        }

        provider.add_data(asset.clone(), test_data.clone());
        println!("  {}: {} data points", ticker, test_data.len());
    }

    // 2. Set up replay engine
    let provider_arc = Arc::new(provider);
    let mut replay = ReplayEngine::new(provider_arc);

    // Faster replay for multi-asset
    replay.set_delay(Duration::from_millis(50));

    // Track which asset is being replayed
    let current_date = Arc::new(std::sync::Mutex::new(String::new()));
    let current_date_clone = current_date.clone();

    replay.set_progress_callback(move |date| {
        let date_str = date.format("%Y-%m-%d").to_string();
        let mut current = current_date_clone.lock().unwrap();

        // Only print when date changes (to show all 3 assets per day)
        if *current != date_str {
            if !current.is_empty() {
                println!(); // New line for new date
            }
            print!("  {}: ", date_str);
            *current = date_str;
        }
        std::io::Write::flush(&mut std::io::stdout()).ok();
    });

    // 3. Collect data per asset
    println!("\nStarting chronological replay...\n");

    let asset_prices: Arc<std::sync::Mutex<HashMap<AssetKey, Vec<f64>>>> =
        Arc::new(std::sync::Mutex::new(HashMap::new()));
    let asset_prices_clone = asset_prices.clone();

    let date_range = DateRange::new(
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
    );

    // 4. Run replay with asset tracking
    let result = replay.run(
        assets.clone(),
        date_range,
        move |asset, _timestamp, value| {
            // Show which asset is being processed
            let ticker = match &asset {
                AssetKey::Equity(ticker) => ticker.as_str(),
                _ => "Unknown",
            };
            print!("{} ", ticker);
            std::io::Write::flush(&mut std::io::stdout()).ok();

            // Store price for analytics
            asset_prices_clone
                .lock()
                .unwrap()
                .entry(asset)
                .or_insert_with(Vec::new)
                .push(value);

            Ok(())
        },
    )?;

    println!("\n\n{}", result);

    // 5. Calculate and display analytics for each asset
    println!("\n=== Per-Asset Analytics Summary ===\n");

    let prices_map = asset_prices.lock().unwrap();

    println!("┌─────────┬─────────────┬─────────────┬──────────────┬──────────────┐");
    println!("│ Asset   │ Data Points │ Final Price │ 10-Day Vol   │ Total Return │");
    println!("├─────────┼─────────────┼─────────────┼──────────────┼──────────────┤");

    for asset in &assets {
        if let Some(prices) = prices_map.get(asset) {
            let ticker = match asset {
                AssetKey::Equity(ticker) => ticker.clone(),
                _ => "Unknown".to_string(),
            };

            let data_points = prices.len();
            let final_price = prices.last().copied().unwrap_or(0.0);

            // Calculate 10-day volatility
            let primitive = StdDevVolatilityAnalytic;
            let mut volatility = Vec::with_capacity(prices.len());
            for (idx, _) in prices.iter().enumerate() {
                let start = idx.saturating_sub(10 - 1);
                volatility.push(primitive.compute(None, &prices[start..=idx]));
            }
            let latest_vol = volatility
                .last()
                .and_then(|v| if v.is_nan() { None } else { Some(*v) })
                .unwrap_or(0.0);

            // Calculate total return
            let first_price = prices.first().copied().unwrap_or(1.0);
            let total_return = ((final_price - first_price) / first_price) * 100.0;

            println!(
                "│ {:7} │ {:>11} │ ${:>10.2} │ {:>12.6} │ {:>11.2}% │",
                ticker, data_points, final_price, latest_vol, total_return
            );
        }
    }

    println!("└─────────┴─────────────┴─────────────┴──────────────┴──────────────┘");

    // 6. Show chronological interleaving statistics
    println!("\n=== Chronological Interleaving ===");
    println!("Total data points: {}", result.total_points);
    println!(
        "Expected per asset: ~{} trading days",
        result.total_points / assets.len()
    );
    println!(
        "All assets replayed in chronological order across {} trading days",
        result.total_points / assets.len()
    );

    println!("\n=== Replay Complete ===");
    println!(
        "Simulated {} trading days in {:.2} seconds",
        result.total_points,
        result.elapsed.as_secs_f64()
    );

    Ok(())
}
