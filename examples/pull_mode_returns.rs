//! Pull-Mode Returns Calculation Example
//!
//! Demonstrates how to:
//! 1. Build a DAG using ReturnsQueryBuilder
//! 2. Load historical data
//! 3. Calculate returns over a complete date range
//! 4. Query results

use analytics::{AssetKey, DateRange, InMemoryDataProvider, ReturnsQueryBuilder, TimeSeriesPoint};
use chrono::NaiveDate;

fn main() {
    println!("📊 Pull-Mode Returns Calculation Example\n");

    // Create asset
    let asset = AssetKey::new_equity("AAPL").expect("Failed to create asset");

    // Define date range for analytics
    let date_range = DateRange::new(
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
    );

    println!("📅 Date Range: {} to {}", date_range.start, date_range.end);
    println!("🏢 Asset: AAPL\n");

    // Build DAG using query builder
    println!("🔧 Building DAG: DataProvider → Returns");
    let builder = ReturnsQueryBuilder::new(asset.clone(), date_range.clone());
    let (dag, data_node_id, returns_node_id) = builder.build_dag().expect("Failed to build DAG");

    println!("   ✓ DAG created with {} nodes", dag.node_count());
    println!("   ✓ Data node: {:?}", data_node_id);
    println!("   ✓ Returns node: {:?}\n", returns_node_id);

    // Create sample historical data
    println!("💾 Creating sample historical data:");
    let prices = vec![
        100.0, 102.0, 101.0, 105.0, 104.0, 107.0, 106.0, 109.0, 108.0, 110.0,
    ];

    let mut data_provider = InMemoryDataProvider::new();
    let mut date = date_range.start;
    let mut points = Vec::new();

    for (i, price) in prices.iter().enumerate() {
        let timestamp = date.and_hms_opt(16, 0, 0).unwrap().and_utc();
        let point = TimeSeriesPoint::new(timestamp, *price);
        points.push(point);

        println!("   Day {}: ${:.2}", i + 1, price);
        date = date + chrono::Duration::days(1);
    }

    data_provider.add_data(asset.clone(), points);

    println!("\n🔄 Computing returns over entire date range...");
    println!("   (In full pull-mode, this would execute the DAG with DataProvider)");
    println!("   (For now, demonstrating with manual calculation)\n");

    // Manual calculation for demonstration
    println!("📈 Calculated Returns:");
    println!("   Day 1: NaN (no previous price)");

    for i in 1..prices.len() {
        let return_val = (prices[i] / prices[i - 1]).ln();
        let percent = return_val * 100.0;
        println!("   Day {}: {:.4} ({:+.2}%)", i + 1, return_val, percent);
    }

    println!("\n✅ Pull-mode demonstration complete!");
    println!("\n💡 Note: Full pull-mode execution (Item 10) will:");
    println!("   • Execute DAG with real DataProvider integration");
    println!("   • Compute all analytics in batch");
    println!("   • Return complete time series for date range");
}
