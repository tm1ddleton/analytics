//! Push-Mode Volatility Calculation Example
//!
//! Demonstrates complete analytics chain:
//! DataProvider → Returns → Volatility (5-day rolling)

use analytics::{AnalyticsDag, AssetKey, NodeOutput, NodeParams, PushModeEngine};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn main() {
    println!("🚀 Push-Mode Volatility Calculation Example\n");

    // Create asset
    let asset = AssetKey::new_equity("AAPL").expect("Failed to create asset");

    // Build DAG: DataProvider → Returns → Volatility
    let mut dag = AnalyticsDag::new();

    let data_node = dag.add_node(
        "data_provider".to_string(),
        NodeParams::None,
        vec![asset.clone()],
    );

    let returns_node = dag.add_node("returns".to_string(), NodeParams::None, vec![asset.clone()]);

    let mut vol_params = HashMap::new();
    vol_params.insert("window_size".to_string(), "5".to_string());
    let vol_node = dag.add_node(
        "volatility".to_string(),
        NodeParams::Map(vol_params),
        vec![asset.clone()],
    );

    dag.add_edge(data_node, returns_node)
        .expect("Failed to add data→returns edge");
    dag.add_edge(returns_node, vol_node)
        .expect("Failed to add returns→volatility edge");

    println!("📊 DAG Structure: DataProvider → Returns → Volatility (5-day)\n");

    // Create push-mode engine
    let mut engine = PushModeEngine::new(dag);
    engine.is_initialized = true; // For demo purposes

    // Register callback for volatility updates
    let vol_updates = Arc::new(Mutex::new(Vec::new()));
    let vol_updates_clone = vol_updates.clone();

    engine
        .register_callback(
            vol_node,
            Box::new(move |output| {
                if let NodeOutput::Single(points) = output {
                    if let Some(last_point) = points.last() {
                        vol_updates_clone
                            .lock()
                            .unwrap()
                            .push(last_point.close_price);
                        println!("   ↳ Volatility updated: {:.6}", last_point.close_price);
                    }
                }
            }),
        )
        .expect("Failed to register callback");

    // Push multiple prices to see volatility evolve
    println!("💰 Pushing 10 prices to observe rolling volatility:\n");

    let prices = vec![
        100.0, 105.0, 103.0, 108.0, 107.0, 110.0, 109.0, 112.0, 111.0, 115.0,
    ];

    let mut ts = Utc::now();

    for (i, price) in prices.iter().enumerate() {
        println!("{}. AAPL: ${:.2}", i + 1, price);

        engine
            .push_data(asset.clone(), ts, *price)
            .expect("Failed to push data");

        ts = ts + chrono::Duration::seconds(1);
    }

    println!("\n✅ Complete!");

    // Query final results
    println!("\n📈 Final Analytics:");

    let data_history = engine
        .get_history(data_node)
        .expect("Failed to get data history");
    println!("  Price points: {}", data_history.len());

    let returns_history = engine
        .get_history(returns_node)
        .expect("Failed to get returns history");
    println!("  Returns calculated: {}", returns_history.len());

    let vol_history = engine
        .get_history(vol_node)
        .expect("Failed to get volatility history");
    println!("  Volatility points: {}", vol_history.len());

    if let Some(latest_vol) = engine.get_latest(vol_node).unwrap() {
        println!("\n  Latest 5-day volatility: {:.6}", latest_vol.close_price);
    }

    let vol_updates = vol_updates.lock().unwrap();
    println!("  Total volatility updates: {}", vol_updates.len());
}
