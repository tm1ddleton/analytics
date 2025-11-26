//! Live Analytics Replay Example
//!
//! This example shows returns and volatility being calculated and printed
//! in real-time as data is replayed.
//!
//! Run with: cargo run --example replay_live_analytics

use analytics::{
    AnalyticsDag, AssetKey, DateRange, InMemoryDataProvider, NodeParams, PushModeEngine,
    ReplayEngine, TimeSeriesPoint,
};
use chrono::{NaiveDate, TimeZone, Utc};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("=== Live Analytics During Replay ===\n");

    // 1. Create sample data
    let mut provider = InMemoryDataProvider::new();
    let aapl = AssetKey::new_equity("AAPL")?;

    println!("Creating 30 days of AAPL data...");
    let mut test_data = Vec::new();
    let mut base_price = 150.0;

    for day in 1..=30 {
        // Add some variation to make it interesting
        let variation = ((day * 7) % 10) as f64 - 5.0;
        base_price += variation * 0.5;

        test_data.push(TimeSeriesPoint::new(
            Utc.with_ymd_and_hms(2024, 1, day, 0, 0, 0).unwrap(),
            base_price,
        ));
    }

    provider.add_data(aapl.clone(), test_data.clone());
    let provider_arc = Arc::new(provider);

    // 2. Set up DAG for analytics
    let mut dag = AnalyticsDag::new();

    // We'll just use the DAG structure, but calculate manually for simplicity
    // (Real integration would use AnalyticsNode with PushModeEngine)

    // 3. Set up data collection
    let prices = Arc::new(Mutex::new(Vec::new()));
    let prices_clone = prices.clone();

    // 4. Create replay engine with slower speed so we can see updates
    let mut replay = ReplayEngine::new(provider_arc);
    replay.set_delay(Duration::from_millis(200)); // 200ms per day = visible updates

    // Progress callback
    replay.set_progress_callback(|date| {
        println!("\n--- {} ---", date.format("%Y-%m-%d"));
    });

    // 5. Run replay with live analytics
    println!("\nStarting replay (200ms per day)...\n");

    let date_range = DateRange::new(
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 1, 30).unwrap(),
    );

    let result = replay.run(
        vec![aapl.clone()],
        date_range,
        move |_asset, timestamp, value| {
            let mut all_prices = prices_clone.lock().unwrap();
            all_prices.push(value);

            // Calculate and print returns (needs at least 2 prices)
            if all_prices.len() >= 2 {
                let prev_price = all_prices[all_prices.len() - 2];
                let return_val = (value / prev_price).ln();
                println!("  Price: ${:.2}  →  Return: {:.6}", value, return_val);
            } else {
                println!("  Price: ${:.2}  →  Return: N/A (first day)", value);
            }

            // Calculate and print 5-day volatility (needs at least 6 prices)
            if all_prices.len() >= 6 {
                // Calculate returns for last 5 days
                let recent_prices = &all_prices[all_prices.len() - 6..];
                let mut returns = Vec::new();
                for i in 1..recent_prices.len() {
                    let ret = (recent_prices[i] / recent_prices[i - 1]).ln();
                    returns.push(ret);
                }

                // Calculate volatility (population std dev)
                let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
                let variance: f64 =
                    returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
                let volatility = variance.sqrt();

                println!("  5-Day Volatility: {:.6}", volatility);
            }

            // Calculate and print 10-day volatility (needs at least 11 prices)
            if all_prices.len() >= 11 {
                let recent_prices = &all_prices[all_prices.len() - 11..];
                let mut returns = Vec::new();
                for i in 1..recent_prices.len() {
                    let ret = (recent_prices[i] / recent_prices[i - 1]).ln();
                    returns.push(ret);
                }

                let mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;
                let variance: f64 =
                    returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len() as f64;
                let volatility = variance.sqrt();

                println!("  10-Day Volatility: {:.6}", volatility);
            }

            Ok(())
        },
    )?;

    println!("\n\n{}", result);

    // 6. Show final summary
    let final_prices = prices.lock().unwrap();

    println!("\n=== Final Summary ===");
    println!("Total days replayed: {}", final_prices.len());
    println!("Starting price: ${:.2}", final_prices.first().unwrap());
    println!("Ending price: ${:.2}", final_prices.last().unwrap());

    let total_return = (final_prices.last().unwrap() / final_prices.first().unwrap()).ln();
    println!(
        "Total log return: {:.6} ({:.2}%)",
        total_return,
        (total_return.exp() - 1.0) * 100.0
    );

    Ok(())
}
