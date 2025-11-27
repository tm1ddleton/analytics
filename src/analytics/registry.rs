use crate::dag::{AnalyticType, DagError, Node, NodeKey, NodeOutput, NodeParams, WindowSpec};
use crate::time_series::{DataProvider, DateRange, TimeSeriesPoint};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

/// Executor invoked for a node to perform pull or push calculations.
pub trait AnalyticExecutor: Send + Sync {
    fn execute_pull(
        &self,
        node: &Node,
        parent_outputs: &[Vec<TimeSeriesPoint>],
        date_range: &DateRange,
        provider: &dyn DataProvider,
    ) -> Result<Vec<TimeSeriesPoint>, DagError>;

    fn execute_push(
        &self,
        node: &Node,
        parent_outputs: &[Vec<TimeSeriesPoint>],
        timestamp: DateTime<Utc>,
        value: f64,
    ) -> Result<NodeOutput, DagError>;
}

/// A trait describing how an analytic node resolves its dependencies.
pub trait AnalyticDefinition: Send + Sync {
    /// Analytic type that this definition satisfies.
    fn analytic_type(&self) -> AnalyticType;

    /// The DAG node type string for logging/identification.
    fn node_type(&self) -> &'static str;

    /// Dependencies that must exist before this node can execute.
    fn dependencies(&self, key: &NodeKey) -> Result<Vec<NodeKey>, DagError>;

    /// Executor that performs pull/push work for this node.
    fn executor(&self) -> &dyn AnalyticExecutor;
}

/// Registry of analytic definitions wired into the DAG.
pub struct AnalyticRegistry {
    definitions: HashMap<AnalyticType, Box<dyn AnalyticDefinition>>,
}

impl std::fmt::Debug for AnalyticRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyticRegistry")
            .field("definitions", &"<omitted>")
            .finish()
    }
}

impl AnalyticRegistry {
    /// Creates a new registry populated with the built-in analytics.
    pub fn new() -> Self {
        let mut definitions: HashMap<AnalyticType, Box<dyn AnalyticDefinition>> = HashMap::new();
        definitions.insert(
            AnalyticType::DataProvider,
            Box::new(DataProviderDefinition::new()),
        );
        definitions.insert(AnalyticType::Returns, Box::new(ReturnsDefinition::new()));
        definitions.insert(
            AnalyticType::Volatility,
            Box::new(VolatilityDefinition::new()),
        );
        AnalyticRegistry { definitions }
    }

    /// Returns the definition associated with an analytic type.
    pub fn definition(&self, analytic: AnalyticType) -> Option<&dyn AnalyticDefinition> {
        self.definitions.get(&analytic).map(|boxed| boxed.as_ref())
    }
}

impl Default for AnalyticRegistry {
    fn default() -> Self {
        Self::new()
    }
}

struct DataProviderDefinition {
    executor: Box<dyn AnalyticExecutor>,
}

impl DataProviderDefinition {
    fn new() -> Self {
        DataProviderDefinition {
            executor: Box::new(DataProviderExecutor),
        }
    }
}

impl AnalyticDefinition for DataProviderDefinition {
    fn analytic_type(&self) -> AnalyticType {
        AnalyticType::DataProvider
    }

    fn node_type(&self) -> &'static str {
        "data_provider"
    }

    fn dependencies(&self, _key: &NodeKey) -> Result<Vec<NodeKey>, DagError> {
        Ok(Vec::new())
    }

    fn executor(&self) -> &dyn AnalyticExecutor {
        self.executor.as_ref()
    }
}

struct ReturnsDefinition {
    executor: Box<dyn AnalyticExecutor>,
}

impl ReturnsDefinition {
    fn new() -> Self {
        ReturnsDefinition {
            executor: Box::new(ReturnsExecutor),
        }
    }
}

impl AnalyticDefinition for ReturnsDefinition {
    fn analytic_type(&self) -> AnalyticType {
        AnalyticType::Returns
    }

    fn node_type(&self) -> &'static str {
        "returns"
    }

    fn dependencies(&self, key: &NodeKey) -> Result<Vec<NodeKey>, DagError> {
        let range = require_range(key)?;
        let lookback = key
            .window
            .as_ref()
            .map(|window| window.burn_in())
            .unwrap_or(2);
        let provider_range = extend_range(&range, lookback);

        Ok(vec![NodeKey {
            analytic: AnalyticType::DataProvider,
            assets: key.assets.clone(),
            range: Some(provider_range),
            window: None,
            override_tag: key.override_tag.clone(),
            params: HashMap::new(),
        }])
    }

    fn executor(&self) -> &dyn AnalyticExecutor {
        self.executor.as_ref()
    }
}

struct VolatilityDefinition {
    executor: Box<dyn AnalyticExecutor>,
}

impl VolatilityDefinition {
    fn new() -> Self {
        VolatilityDefinition {
            executor: Box::new(VolatilityExecutor),
        }
    }
}

impl AnalyticDefinition for VolatilityDefinition {
    fn analytic_type(&self) -> AnalyticType {
        AnalyticType::Volatility
    }

    fn node_type(&self) -> &'static str {
        "volatility"
    }

    fn dependencies(&self, key: &NodeKey) -> Result<Vec<NodeKey>, DagError> {
        let range = require_range(key)?;
        let window_spec = key.window.clone().unwrap_or_else(|| WindowSpec::fixed(10));
        let returns_range = extend_range(&range, window_spec.burn_in());

        Ok(vec![NodeKey {
            analytic: AnalyticType::Returns,
            assets: key.assets.clone(),
            range: Some(returns_range),
            window: Some(WindowSpec::fixed(2)),
            override_tag: key.override_tag.clone(),
            params: HashMap::new(),
        }])
    }

    fn executor(&self) -> &dyn AnalyticExecutor {
        self.executor.as_ref()
    }
}

fn require_range(key: &NodeKey) -> Result<DateRange, DagError> {
    key.range
        .clone()
        .ok_or_else(|| DagError::InvalidOperation("Analytics node missing range".to_string()))
}

fn extend_range(range: &DateRange, burn_in_days: usize) -> DateRange {
    if burn_in_days == 0 {
        return range.clone();
    }

    let extra = Duration::days(burn_in_days as i64);
    let start = range.start.checked_sub_signed(extra).unwrap_or(range.start);

    DateRange::new(start, range.end)
}

struct DataProviderExecutor;

impl AnalyticExecutor for DataProviderExecutor {
    fn execute_pull(
        &self,
        node: &Node,
        _parent_outputs: &[Vec<TimeSeriesPoint>],
        date_range: &DateRange,
        provider: &dyn DataProvider,
    ) -> Result<Vec<TimeSeriesPoint>, DagError> {
        let asset = node.assets.first().ok_or_else(|| {
            DagError::ExecutionError("DataProvider node has no assets".to_string())
        })?;
        let data = provider.get_time_series(asset, date_range)?;
        Ok(data)
    }

    fn execute_push(
        &self,
        _node: &Node,
        _parent_outputs: &[Vec<TimeSeriesPoint>],
        timestamp: DateTime<Utc>,
        value: f64,
    ) -> Result<NodeOutput, DagError> {
        Ok(NodeOutput::Single(vec![TimeSeriesPoint::new(
            timestamp, value,
        )]))
    }
}

struct ReturnsExecutor;

impl AnalyticExecutor for ReturnsExecutor {
    fn execute_pull(
        &self,
        _node: &Node,
        parent_outputs: &[Vec<TimeSeriesPoint>],
        _date_range: &DateRange,
        _provider: &dyn DataProvider,
    ) -> Result<Vec<TimeSeriesPoint>, DagError> {
        if parent_outputs.is_empty() {
            return Err(DagError::ExecutionError(
                "Returns node requires parent data".to_string(),
            ));
        }

        let prices_data = &parent_outputs[0];
        if prices_data.is_empty() {
            return Ok(Vec::new());
        }

        let prices: Vec<f64> = prices_data.iter().map(|p| p.close_price).collect();
        let returns = super::calculate_returns(&prices);

        let result: Vec<TimeSeriesPoint> = prices_data
            .iter()
            .zip(returns.iter())
            .map(|(point, &ret)| TimeSeriesPoint::new(point.timestamp, ret))
            .collect();

        Ok(result)
    }

    fn execute_push(
        &self,
        _node: &Node,
        parent_outputs: &[Vec<TimeSeriesPoint>],
        _timestamp: DateTime<Utc>,
        _value: f64,
    ) -> Result<NodeOutput, DagError> {
        if parent_outputs.is_empty() || parent_outputs[0].len() < 2 {
            return Err(DagError::ExecutionError(
                "Returns update requires at least two price points".to_string(),
            ));
        }

        let value = super::calculate_returns_update(&parent_outputs[0]);
        Ok(NodeOutput::Scalar(value))
    }
}

struct VolatilityExecutor;

impl VolatilityExecutor {
    fn window_size_from_node(node: &Node) -> usize {
        if let NodeParams::Map(ref params) = node.params {
            params
                .get("window_size")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(10)
        } else {
            10
        }
    }
}

impl AnalyticExecutor for VolatilityExecutor {
    fn execute_pull(
        &self,
        node: &Node,
        parent_outputs: &[Vec<TimeSeriesPoint>],
        _date_range: &DateRange,
        _provider: &dyn DataProvider,
    ) -> Result<Vec<TimeSeriesPoint>, DagError> {
        if parent_outputs.is_empty() {
            return Err(DagError::ExecutionError(
                "Volatility node requires returns data".to_string(),
            ));
        }

        let returns_data = &parent_outputs[0];
        if returns_data.is_empty() {
            return Ok(Vec::new());
        }

        let window_size = Self::window_size_from_node(node);
        let returns = returns_data
            .iter()
            .map(|p| p.close_price)
            .collect::<Vec<_>>();
        let volatility = super::calculate_volatility(&returns, window_size);

        let result: Vec<TimeSeriesPoint> = returns_data
            .iter()
            .zip(volatility.iter())
            .map(|(point, &vol)| TimeSeriesPoint::new(point.timestamp, vol))
            .collect();

        Ok(result)
    }

    fn execute_push(
        &self,
        node: &Node,
        parent_outputs: &[Vec<TimeSeriesPoint>],
        _timestamp: DateTime<Utc>,
        _value: f64,
    ) -> Result<NodeOutput, DagError> {
        if parent_outputs.is_empty() || parent_outputs[0].is_empty() {
            return Err(DagError::ExecutionError(
                "Volatility update requires returns data".to_string(),
            ));
        }

        let window_size = Self::window_size_from_node(node);
        let value = super::calculate_volatility_update(&parent_outputs[0], window_size);
        Ok(NodeOutput::Scalar(value))
    }
}
