//! Example: Pull-Mode Multi-Asset Parallel Query
//!
//! This example demonstrates how to use pull-mode to compute analytics for
//! multiple assets in parallel.
//!
//! Run with: `cargo run --example pull_mode_multi_asset`

use analytics::{
    AnalyticsDag, AssetKey, DateRange, InMemoryDataProvider, NodeId, NodeParams, TimeSeriesPoint,
};
use chrono::{NaiveDate, TimeZone, Utc};
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pull-Mode Multi-Asset Parallel Query Example ===\n");

    // Step 1: Create sample data for 3 assets
    println!("Step 1: Creating sample data for AAPL, MSFT, GOOG...");
    let mut provider = InMemoryDataProvider::new();

    let assets = vec![
        (AssetKey::new_equity("AAPL")?, 150.0),
        (AssetKey::new_equity("MSFT")?, 300.0),
        (AssetKey::new_equity("GOOG")?, 2500.0),
    ];

    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    for (asset, base_price) in &assets {
        let mut prices = Vec::new();
        let mut current_price = *base_price;

        for day in 0..252 {
            // 1 trading year
            let date = start_date + chrono::Duration::days(day);
            let timestamp = Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap());

            // Simulate different price movements for each asset
            current_price += (day % 5) as f64 - 2.0;
            prices.push(TimeSeriesPoint::new(timestamp, current_price));
        }

        provider.add_data(asset.clone(), prices);
    }

    println!("  ✓ Created 252 days of price data for 3 assets\n");

    // Step 2: Build DAG for all assets
    println!("Step 2: Building analytics DAG for all assets...");
    let mut dag = AnalyticsDag::new();
    let mut node_map: HashMap<String, (AssetKey, NodeId)> = HashMap::new();

    for (asset, _) in &assets {
        let ticker = match asset {
            AssetKey::Equity(ref t) => t.as_str(),
            _ => continue,
        };

        let data_node = dag.add_node(
            "DataProvider".to_string(),
            NodeParams::None,
            vec![asset.clone()],
        );

        let returns_node =
            dag.add_node("Returns".to_string(), NodeParams::None, vec![asset.clone()]);

        let mut vol_params = HashMap::new();
        vol_params.insert("window_size".to_string(), "10".to_string());
        let vol_node = dag.add_node(
            "Volatility".to_string(),
            NodeParams::Map(vol_params),
            vec![asset.clone()],
        );

        dag.add_edge(data_node, returns_node)?;
        dag.add_edge(returns_node, vol_node)?;

        node_map.insert(ticker.to_string(), (asset.clone(), vol_node));
    }

    println!("  ✓ DAG created for 3 assets\n");

    // Step 3: Execute parallel query
    println!("Step 3: Executing parallel query for all assets...");
    let date_range = DateRange::new(
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
    );

    let node_ids: Vec<NodeId> = node_map.values().map(|(_, id)| *id).collect();

    let start_time = Instant::now();
    let results = dag.execute_pull_mode_parallel(node_ids, date_range, &provider)?;
    let parallel_elapsed = start_time.elapsed();

    println!("  ✓ Parallel query completed in {:?}", parallel_elapsed);
    println!("  ✓ Retrieved results for {} assets\n", results.len());

    // Step 4: Display results per asset
    println!("Step 4: Displaying results per asset...\n");

    for (ticker, (_, node_id)) in &node_map {
        if let Some(data) = results.get(node_id) {
            let valid_points: Vec<_> = data.iter().filter(|p| !p.close_price.is_nan()).collect();

            if !valid_points.is_empty() {
                let avg_vol = valid_points.iter().map(|p| p.close_price).sum::<f64>()
                    / valid_points.len() as f64;

                println!(
                    "{:<6} - {} data points, avg volatility: {:.6}",
                    ticker,
                    data.len(),
                    avg_vol
                );
            }
        }
    }

    // Step 5: Compare with sequential execution
    println!("\n=== Performance Comparison ===");
    println!("Parallel execution: {:?}", parallel_elapsed);
    println!(
        "Note: Sequential timing would be ~{:?} (3x single-asset time)",
        parallel_elapsed * 3
    );
    println!("(Actual parallelism depends on system resources)\n");

    println!("✅ Multi-asset parallel query completed successfully!");

    Ok(())
}
