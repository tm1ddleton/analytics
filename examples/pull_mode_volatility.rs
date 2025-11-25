//! Pull-Mode Volatility Calculation Example
//! 
//! Demonstrates the concept of pull-mode analytics:
//! - Query for volatility over a complete date range
//! - Get full time series result
//! - Compare with push-mode (which updates incrementally)

use analytics::{
    VolatilityQueryBuilder, AssetKey, DateRange, 
    calculate_returns, calculate_volatility,
};
use chrono::NaiveDate;

fn main() {
    println!("📊 Pull-Mode Volatility Calculation Example\n");
    
    // Create asset
    let asset = AssetKey::new_equity("AAPL").expect("Failed to create asset");
    
    // Define parameters
    let window_size = 5;
    let date_range = DateRange::new(
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
    );
    
    println!("📅 Date Range: {} to {}", date_range.start, date_range.end);
    println!("🏢 Asset: AAPL");
    println!("📏 Window Size: {} days\n", window_size);
    
    // Build DAG using query builder
    println!("🔧 Building DAG: DataProvider → Returns → Volatility");
    let builder = VolatilityQueryBuilder::new(asset, window_size, date_range);
    let (dag, data_node_id, returns_node_id, vol_node_id) = builder.build_dag()
        .expect("Failed to build DAG");
    
    println!("   ✓ DAG created with {} nodes", dag.node_count());
    println!("   ✓ Data node: {:?}", data_node_id);
    println!("   ✓ Returns node: {:?}", returns_node_id);
    println!("   ✓ Volatility node: {:?}\n", vol_node_id);
    
    // Sample historical prices
    println!("💾 Historical Prices:");
    let prices = vec![
        100.0, 102.0, 101.0, 105.0, 104.0,
        107.0, 106.0, 109.0, 108.0, 110.0,
        109.0, 112.0, 111.0, 114.0, 113.0,
    ];
    
    for (i, price) in prices.iter().enumerate() {
        println!("   Day {:2}: ${:.2}", i + 1, price);
    }
    
    println!("\n🔄 Computing analytics over entire date range...\n");
    
    // Calculate returns (full time series)
    println!("📈 Step 1: Calculate Returns");
    let returns = calculate_returns(&prices);
    println!("   ✓ {} returns calculated", returns.len());
    
    // Calculate volatility (full time series)
    println!("\n📊 Step 2: Calculate {}-day Rolling Volatility", window_size);
    let volatility = calculate_volatility(&returns, window_size);
    println!("   ✓ {} volatility points calculated", volatility.len());
    
    // Display results
    println!("\n📉 Volatility Time Series:");
    for (i, vol) in volatility.iter().enumerate() {
        if vol.is_nan() {
            println!("   Day {:2}: NaN", i + 1);
        } else {
            println!("   Day {:2}: {:.6}", i + 1, vol);
        }
    }
    
    // Final statistics
    let valid_vols: Vec<f64> = volatility.iter()
        .filter(|v| !v.is_nan())
        .copied()
        .collect();
    
    if !valid_vols.is_empty() {
        let avg_vol = valid_vols.iter().sum::<f64>() / valid_vols.len() as f64;
        let max_vol = valid_vols.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_vol = valid_vols.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        
        println!("\n📊 Statistics:");
        println!("   Average Volatility: {:.6}", avg_vol);
        println!("   Max Volatility: {:.6}", max_vol);
        println!("   Min Volatility: {:.6}", min_vol);
    }
    
    println!("\n✅ Pull-mode demonstration complete!");
    
    println!("\n🔄 Push vs Pull Comparison:");
    println!("   Push-Mode (Item 6 ✓): Updates incrementally as each price arrives");
    println!("   Pull-Mode (Item 10): Computes entire time series for date range");
    
    println!("\n💡 Full Pull-Mode Engine (Item 10) will add:");
    println!("   • Batch execution through DAG with DataProvider");
    println!("   • Efficient computation of full time series");
    println!("   • Query API: query_analytics(asset, analytic, range)");
    println!("   • Caching and optimization for repeated queries");
}

