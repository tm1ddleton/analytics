//! Example: Pull-Mode Analytics Query
//!
//! This example demonstrates how to use pull-mode to compute historical analytics
//! for a single asset over a 1-year period.
//!
//! Run with: `cargo run --example pull_mode_query`

use analytics::{
    AnalyticsDag, AssetKey, DateRange, InMemoryDataProvider, NodeParams, TimeSeriesPoint,
};
use chrono::{NaiveDate, TimeZone, Utc};
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pull-Mode Analytics Query Example ===\n");

    // Step 1: Create sample data for AAPL (1 year of daily prices)
    println!("Step 1: Creating sample data for AAPL...");
    let mut provider = InMemoryDataProvider::new();
    let aapl = AssetKey::new_equity("AAPL")?;

    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let mut prices = Vec::new();
    let mut current_price = 150.0;

    for day in 0..365 {
        let date = start_date + chrono::Duration::days(day);
        let timestamp = Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap());

        // Simulate price movement
        current_price += (day % 7) as f64 - 3.0;
        prices.push(TimeSeriesPoint::new(timestamp, current_price));
    }

    provider.add_data(aapl.clone(), prices);
    println!("  ✓ Created 365 days of price data\n");

    // Step 2: Build DAG with DataProvider → Returns → Volatility
    println!("Step 2: Building analytics DAG...");
    let mut dag = AnalyticsDag::new();

    let data_node = dag.add_node(
        "DataProvider".to_string(),
        NodeParams::None,
        vec![aapl.clone()],
    );

    let returns_node = dag.add_node("Returns".to_string(), NodeParams::None, vec![aapl.clone()]);

    let mut vol_params = HashMap::new();
    vol_params.insert("window_size".to_string(), "20".to_string());
    let vol_node = dag.add_node(
        "Volatility".to_string(),
        NodeParams::Map(vol_params),
        vec![aapl],
    );

    dag.add_edge(data_node, returns_node)?;
    dag.add_edge(returns_node, vol_node)?;
    println!("  ✓ DAG created: DataProvider → Returns → Volatility(20)\n");

    // Step 3: Execute pull-mode query
    println!("Step 3: Executing pull-mode query for full year...");
    let date_range = DateRange::new(
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
    );

    let start_time = Instant::now();
    let results = dag.execute_pull_mode(vol_node, date_range, &provider)?;
    let elapsed = start_time.elapsed();

    println!("  ✓ Query completed in {:?}", elapsed);
    println!("  ✓ Retrieved {} volatility data points\n", results.len());

    // Step 4: Display results
    println!("Step 4: Displaying results...\n");

    // Show first 10 points
    println!("First 10 data points:");
    println!("{:<12} {:<15}", "Date", "Volatility");
    println!("{:-<27}", "");
    for point in results.iter().take(10) {
        let date = point.timestamp.date_naive();
        if point.close_price.is_nan() {
            println!("{:<12} {:<15}", date, "NaN (burn-in)");
        } else {
            println!("{:<12} {:.6}", date, point.close_price);
        }
    }

    println!("\n...\n");

    // Show last 10 points
    println!("Last 10 data points:");
    println!("{:<12} {:<15}", "Date", "Volatility");
    println!("{:-<27}", "");
    for point in results.iter().rev().take(10).rev() {
        let date = point.timestamp.date_naive();
        println!("{:<12} {:.6}", date, point.close_price);
    }

    // Step 5: Statistics
    println!("\n=== Summary ===");
    let valid_points: Vec<_> = results.iter().filter(|p| !p.close_price.is_nan()).collect();

    if !valid_points.is_empty() {
        let min_vol = valid_points
            .iter()
            .map(|p| p.close_price)
            .fold(f64::INFINITY, f64::min);
        let max_vol = valid_points
            .iter()
            .map(|p| p.close_price)
            .fold(f64::NEG_INFINITY, f64::max);
        let avg_vol =
            valid_points.iter().map(|p| p.close_price).sum::<f64>() / valid_points.len() as f64;

        println!("Valid data points: {}", valid_points.len());
        println!("Min volatility:    {:.6}", min_vol);
        println!("Max volatility:    {:.6}", max_vol);
        println!("Avg volatility:    {:.6}", avg_vol);
    }

    println!("\n✅ Pull-mode query completed successfully!");

    Ok(())
}
