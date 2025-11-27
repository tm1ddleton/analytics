use analytics::asset_key::AssetKey;
use analytics::dag::{AnalyticType, AnalyticsDag, NodeKey, WindowSpec};
use analytics::time_series::DateRange;
use chrono::NaiveDate;
use std::collections::HashMap;

fn make_volatility_key(asset: AssetKey, range: DateRange, window_size: usize) -> NodeKey {
    let mut params = HashMap::new();
    params.insert("window_size".to_string(), window_size.to_string());

    NodeKey {
        analytic: AnalyticType::Volatility,
        assets: vec![asset],
        range: Some(range),
        window: Some(WindowSpec::fixed(window_size)),
        override_tag: None,
        params,
    }
}

#[test]
fn registry_builds_volatility_chain() {
    let asset = AssetKey::new_equity("AAPL").unwrap();
    let range = DateRange::new(
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
    );
    let key = make_volatility_key(asset.clone(), range.clone(), 5);

    let mut dag = AnalyticsDag::new();
    let target_node = dag.resolve_node(key.clone()).unwrap();

    assert_eq!(
        dag.node_count(),
        3,
        "Volatility should build returns + data provider"
    );
    assert_eq!(dag.edge_count(), 2, "Two edges should connect the chain");
    assert_eq!(dag.resolve_node(key.clone()).unwrap(), target_node);

    let execution = dag.execution_order().unwrap();
    let nodes: Vec<_> = execution
        .iter()
        .filter_map(|node_id| dag.get_node(*node_id))
        .map(|node| node.node_type.clone())
        .collect();

    assert_eq!(nodes.first().map(String::as_str), Some("data_provider"));
    assert!(nodes.contains(&"returns".to_string()));
    assert!(nodes.contains(&"volatility".to_string()));
}
