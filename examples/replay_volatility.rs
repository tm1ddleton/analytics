//! Replay Volatility Example
//!
//! This example demonstrates the ReplayEngine by:
//! 1. Creating sample AAPL data for 3 months
//! 2. Setting up a replay with progress callbacks
//! 3. Simulating real-time data arrival at high speed
//! 4. Collecting and displaying volatility calculations
//!
//! Run with: cargo run --example replay_volatility

use analytics::{
    analytics::containers::{StdDevVolatilityAnalytic, VolatilityAnalytic},
    AssetKey, DateRange, InMemoryDataProvider, ReplayEngine, TimeSeriesPoint,
};
use chrono::{NaiveDate, TimeZone, Utc};
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

    println!("=== High-Speed Data Replay: Volatility Example ===\n");

    // 1. Create sample data for AAPL (3 months, ~60 trading days)
    println!("Creating sample AAPL data for 3 months...");
    let mut provider = InMemoryDataProvider::new();
    let aapl = AssetKey::new_equity("AAPL")?;

    let mut test_data = Vec::new();
    let mut base_price = 150.0;

    // Generate realistic-looking price data with some volatility
    for month in 1..=3 {
        let days_in_month = match month {
            2 => 19, // February (approximate trading days)
            _ => 21,
        };

        for day in 1..=days_in_month {
            // Add some random-looking variation
            let variation = ((day * 7 + month * 13) % 10) as f64 - 5.0;
            base_price += variation * 0.5;

            test_data.push(TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, month, day, 0, 0, 0).unwrap(),
                base_price,
            ));
        }
    }

    provider.add_data(aapl.clone(), test_data.clone());
    println!("Created {} data points", test_data.len());

    // 2. Set up replay engine
    let provider_arc = Arc::new(provider);
    let mut replay = ReplayEngine::new(provider_arc);

    // Configure replay speed (100ms per trading day = ~6 seconds for 60 days)
    replay.set_delay(Duration::from_millis(100));

    // Add progress callback to show current date being replayed
    replay.set_progress_callback(|date| {
        print!("\r  Replaying: {} ", date.format("%Y-%m-%d"));
        std::io::Write::flush(&mut std::io::stdout()).ok();
    });

    // 3. Collect data for volatility calculation
    println!("\nStarting replay...\n");

    let prices = Arc::new(std::sync::Mutex::new(Vec::new()));
    let prices_clone = prices.clone();

    let date_range = DateRange::new(
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
    );

    // 4. Run replay
    let result = replay.run(
        vec![aapl.clone()],
        date_range,
        move |_asset, _timestamp, value| {
            prices_clone.lock().unwrap().push(value);
            Ok(())
        },
    )?;

    println!("\n\n{}", result);

    // 5. Calculate and display rolling volatility
    println!("\n=== Rolling Volatility Analysis ===\n");

    let collected_prices = prices.lock().unwrap();
    let window_sizes = vec![5usize, 10, 20];

    for window in window_sizes {
        let primitive = StdDevVolatilityAnalytic;
        let mut volatility = Vec::with_capacity(collected_prices.len());
        for (idx, _) in collected_prices.iter().enumerate() {
            let start = idx.saturating_sub(window.saturating_sub(1));
            volatility.push(primitive.compute(None, &collected_prices[start..=idx]));
        }

        // Get the last few volatility values
        let last_n = 5.min(volatility.len());
        let recent_vol: Vec<f64> = volatility
            .iter()
            .skip(volatility.len() - last_n)
            .copied()
            .collect();

        println!("{}-day volatility (last {} values):", window, last_n);
        for (i, vol) in recent_vol.iter().enumerate() {
            if vol.is_nan() {
                println!("  Day {}: N/A (insufficient data)", i + 1);
            } else {
                println!("  Day {}: {:.6}", i + 1, vol);
            }
        }
        println!();
    }

    // 6. Show final statistics
    let final_prices = &collected_prices[collected_prices.len().saturating_sub(10)..];
    println!("=== Final Price Summary ===");
    println!("Last 10 prices:");
    for (i, price) in final_prices.iter().enumerate() {
        println!("  {}: ${:.2}", i + 1, price);
    }

    println!("\n=== Replay Complete ===");
    println!(
        "Simulated {} trading days in {:.2} seconds",
        result.total_points,
        result.elapsed.as_secs_f64()
    );

    Ok(())
}
