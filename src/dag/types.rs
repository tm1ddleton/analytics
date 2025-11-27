use crate::asset_key::AssetKey;
use crate::time_series::{DateRange, TimeSeriesPoint};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Node identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub usize);

/// Parameters for a node (generic, can be extended)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeParams {
    /// Simple key-value parameters
    Map(HashMap<String, String>),
    /// Empty parameters
    None,
}

/// Analytic types supported by DAG nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalyticType {
    DataProvider,
    Returns,
    Volatility,
    StdDev,
    ExponentialMovingAverage,
}

impl AnalyticType {
    pub fn from_str(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "returns" => AnalyticType::Returns,
            "volatility" => AnalyticType::Volatility,
            "std_dev" | "stddev" => AnalyticType::StdDev,
            "ema" | "exponentialmovingaverage" => AnalyticType::ExponentialMovingAverage,
            _ => AnalyticType::DataProvider,
        }
    }
}

impl std::fmt::Display for AnalyticType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            AnalyticType::DataProvider => "data_provider",
            AnalyticType::Returns => "returns",
            AnalyticType::Volatility => "volatility",
            AnalyticType::StdDev => "std_dev",
            AnalyticType::ExponentialMovingAverage => "ema",
        };
        write!(f, "{repr}")
    }
}

/// Windowing strategies describing lookback behavior.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WindowKind {
    Fixed {
        size: usize,
    },
    Exponential {
        lambda: OrderedFloat<f64>,
        lookback: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindowSpec {
    pub kind: WindowKind,
}

impl WindowSpec {
    pub fn fixed(size: usize) -> Self {
        WindowSpec {
            kind: WindowKind::Fixed { size },
        }
    }

    pub fn exponential(lambda: f64, lookback: usize) -> Self {
        WindowSpec {
            kind: WindowKind::Exponential {
                lambda: OrderedFloat(lambda),
                lookback,
            },
        }
    }
}

impl WindowSpec {
    pub fn burn_in(&self) -> usize {
        match self.kind {
            WindowKind::Fixed { size } => size,
            WindowKind::Exponential { lookback, .. } => lookback,
        }
    }
}

/// Node metadata key for deduplication and caching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeKey {
    pub analytic: AnalyticType,
    pub assets: Vec<AssetKey>,
    pub range: Option<DateRange>,
    pub window: Option<WindowSpec>,
    pub override_tag: Option<String>,
    pub params: HashMap<String, String>,
}

impl NodeKey {
    pub fn params_map(&self) -> HashMap<String, String> {
        let mut map = self.params.clone();
        map.insert("analytic_type".to_string(), self.analytic.to_string());
        if let Some(tag) = &self.override_tag {
            map.insert("override".to_string(), tag.clone());
        }
        if let Some(range) = &self.range {
            map.insert("start_date".to_string(), range.start.to_string());
            map.insert("end_date".to_string(), range.end.to_string());
        }
        if let Some(window) = &self.window {
            match window.kind {
                WindowKind::Fixed { size } => {
                    map.insert("window_size".to_string(), size.to_string());
                }
                WindowKind::Exponential { lambda, lookback } => {
                    map.insert("ema_lambda".to_string(), lambda.to_string());
                    map.insert("ema_lookback".to_string(), lookback.to_string());
                }
            }
        }
        map
    }
}

impl Hash for NodeKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.analytic.hash(state);
        self.assets.hash(state);
        if let Some(range) = &self.range {
            range.start.hash(state);
            range.end.hash(state);
        }
        self.window.hash(state);
        self.override_tag.hash(state);
        let mut pairs: Vec<_> = self.params.iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(b.0));
        for (k, v) in pairs {
            k.hash(state);
            v.hash(state);
        }
    }
}

/// Execution result for a node (can be collection of time series)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeOutput {
    /// Single time series result
    Single(Vec<TimeSeriesPoint>),
    /// Multiple time series results (collection)
    Collection(Vec<Vec<TimeSeriesPoint>>),
    /// Scalar value (e.g., correlation coefficient)
    Scalar(f64),
    /// No output (for sink nodes)
    None,
}

/// Node in the DAG representing an analytics computation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier for this node
    pub id: NodeId,
    /// Node type/name (e.g., "moving_average", "correlation")
    pub node_type: String,
    /// Parameters for this node (e.g., {"window": "20"} for 20-day moving average)
    pub params: NodeParams,
    /// Assets this node operates on
    pub assets: Vec<AssetKey>,
    /// Computation function identifier (for future use)
    pub computation_id: Option<String>,
}

impl Node {
    /// Creates a new node
    pub fn new(id: NodeId, node_type: String, params: NodeParams, assets: Vec<AssetKey>) -> Self {
        Node {
            id,
            node_type,
            params,
            assets,
            computation_id: None,
        }
    }
}

#[cfg(test)]
mod node_key_tests {
    use super::*;
    use chrono::NaiveDate;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;

    fn hash_key(key: &NodeKey) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn identical_node_keys_have_equal_hash() {
        let asset = AssetKey::new_equity("AAPL").unwrap();
        let range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );
        let key = NodeKey {
            analytic: AnalyticType::Returns,
            assets: vec![asset.clone()],
            range: Some(range),
            window: None,
            override_tag: None,
            params: HashMap::new(),
        };
        let key_clone = key.clone();
        assert_eq!(hash_key(&key), hash_key(&key_clone));
    }

    #[test]
    fn override_tag_mutates_hash() {
        let asset = AssetKey::new_equity("AAPL").unwrap();
        let base_key = NodeKey {
            analytic: AnalyticType::Returns,
            assets: vec![asset.clone()],
            range: None,
            window: None,
            override_tag: None,
            params: HashMap::new(),
        };
        let mut other = base_key.clone();
        other.override_tag = Some("arith".to_string());
        assert_ne!(hash_key(&base_key), hash_key(&other));
    }
}
