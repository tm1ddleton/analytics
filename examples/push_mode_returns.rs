//! Push-Mode Returns Calculation Example
//!
//! Demonstrates how to:
//! 1. Build a DAG with DataProvider → Returns
//! 2. Push prices incrementally
//! 3. Observe returns calculating automatically
//! 4. Use callbacks to see updates in real-time

use analytics::{AnalyticsDag, AssetKey, NodeOutput, NodeParams, PushModeEngine};
use chrono::Utc;
use std::sync::{Arc, Mutex};

fn main() {
    println!("🚀 Push-Mode Returns Calculation Example\n");

    // Create asset
    let asset = AssetKey::new_equity("AAPL").expect("Failed to create asset");

    // Build DAG: DataProvider → Returns
    let mut dag = AnalyticsDag::new();

    let data_node = dag.add_node(
        "data_provider".to_string(),
        NodeParams::None,
        vec![asset.clone()],
    );

    let returns_node = dag.add_node("returns".to_string(), NodeParams::None, vec![asset.clone()]);

    dag.add_edge(data_node, returns_node)
        .expect("Failed to add edge");

    println!("📊 DAG Structure: DataProvider → Returns\n");

    // Create push-mode engine
    let mut engine = PushModeEngine::new(dag);
    engine.is_initialized = true; // For demo purposes

    // Register callback to observe returns
    let returns_count = Arc::new(Mutex::new(0));
    let returns_count_clone = returns_count.clone();

    engine
        .register_callback(
            returns_node,
            Box::new(move |_, output, _| {
                if let NodeOutput::Single(points) = output {
                    let mut count = returns_count_clone.lock().unwrap();
                    *count += points.len();

                    if let Some(last_point) = points.last() {
                        if !last_point.close_price.is_nan() {
                            println!(
                                "   ↳ Return calculated: {:.4} ({:.2}%)",
                                last_point.close_price,
                                last_point.close_price * 100.0
                            );
                        } else {
                            println!("   ↳ First return: NaN (no previous price)");
                        }
                    }
                }
            }),
        )
        .expect("Failed to register callback");

    // Push prices incrementally
    println!("💰 Pushing prices incrementally:\n");

    let prices = vec![100.0, 105.0, 103.0, 108.0, 107.0];
    let mut ts = Utc::now();

    for (i, price) in prices.iter().enumerate() {
        println!("{}. Pushing AAPL price: ${:.2}", i + 1, price);

        engine
            .push_data(asset.clone(), ts, *price)
            .expect("Failed to push data");

        ts = ts + chrono::Duration::seconds(1);
    }

    println!("\n✅ Complete!");
    println!(
        "Total returns calculated: {}",
        *returns_count.lock().unwrap()
    );

    // Query final results
    println!("\n📈 Final Results:");

    let data_history = engine
        .get_history(data_node)
        .expect("Failed to get data history");
    println!("  Data points: {}", data_history.len());

    let returns_history = engine
        .get_history(returns_node)
        .expect("Failed to get returns history");
    println!("  Returns points: {}", returns_history.len());

    if let Some(latest) = engine.get_latest(returns_node).unwrap() {
        println!("  Latest return: {:.4}", latest.close_price);
    }
}
