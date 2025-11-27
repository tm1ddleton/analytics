use analytics::dag::{AnalyticsDag, NodeParams};
use analytics::push_mode::PushModeEngine;
use analytics::sqlite_provider::SqliteDataProvider;
use analytics::time_series::{DateRange, InMemoryDataProvider, TimeSeriesPoint};
use analytics::{AssetKey, Equity, NodeOutput};
use chrono::{Duration, TimeZone, Utc};
use std::sync::{Arc, Mutex};

fn sample_prices() -> Vec<TimeSeriesPoint> {
    let base = Utc.with_ymd_and_hms(2024, 1, 2, 16, 0, 0).unwrap();
    (0..5)
        .map(|i| TimeSeriesPoint::new(base + Duration::days(i), 150.0 + i as f64))
        .collect()
}

#[test]
fn run_demo_pull_mode_returns() {
    let mut provider = SqliteDataProvider::new_in_memory().unwrap();
    let asset_key = AssetKey::new_equity("AAPL").unwrap();
    let equity = Equity::new("AAPL", "Apple", "NASDAQ", "USD", "Technology").unwrap();
    provider.store_asset_equity(&equity).unwrap();

    let points = sample_prices();
    provider
        .insert_time_series_batch(&asset_key, &points)
        .unwrap();

    let mut dag = AnalyticsDag::new();
    let data_node = dag.add_node(
        "data_provider".to_string(),
        NodeParams::None,
        vec![asset_key.clone()],
    );
    let returns_node = dag.add_node(
        "returns".to_string(),
        NodeParams::None,
        vec![asset_key.clone()],
    );
    dag.add_edge(data_node, returns_node).unwrap();

    let start_date = points.first().unwrap().timestamp.date_naive();
    let end_date = points.last().unwrap().timestamp.date_naive();
    let range = DateRange::new(start_date, end_date);

    let result = dag
        .execute_pull_mode(returns_node, range, &provider)
        .unwrap();

    assert!(
        !result.is_empty(),
        "Pull-mode returns should produce values"
    );
    assert!(
        result.iter().any(|point| point.close_price.is_finite()),
        "At least one return should be finite"
    );
}

#[test]
fn run_demo_push_mode_returns_updates() {
    let asset_key = AssetKey::new_equity("AAPL").unwrap();
    let mut dag = AnalyticsDag::new();
    let data_node = dag.add_node(
        "data_provider".to_string(),
        NodeParams::None,
        vec![asset_key.clone()],
    );
    let returns_node = dag.add_node(
        "returns".to_string(),
        NodeParams::None,
        vec![asset_key.clone()],
    );
    dag.add_edge(data_node, returns_node).unwrap();

    let mut engine = PushModeEngine::new(dag);
    let provider = InMemoryDataProvider::new();
    engine.initialize(&provider, Utc::now(), 30).unwrap();

    let captured = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = Arc::clone(&captured);

    engine
        .register_callback(
            returns_node,
            Box::new(move |_id, output, _timestamp| {
                if let NodeOutput::Scalar(value) = output {
                    captured_clone.lock().unwrap().push(*value);
                }
            }),
        )
        .unwrap();

    let start = Utc.with_ymd_and_hms(2024, 1, 2, 16, 0, 0).unwrap();
    for i in 0..5 {
        engine
            .push_data(
                asset_key.clone(),
                start + Duration::days(i),
                150.0 + i as f64,
            )
            .unwrap();
    }

    let outputs = captured.lock().unwrap();
    assert!(
        outputs.len() >= 1,
        "Push-mode callback should receive one or more scalars"
    );
    assert!(outputs.iter().all(|v| v.is_finite()));
}
