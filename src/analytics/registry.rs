use crate::analytics::lag::{FixedLag, LagAnalytic};
use crate::analytics::primitives::{
    LogReturnPrimitive, ReturnPrimitive, StdDevVolatilityPrimitive, VolatilityPrimitive,
};
use crate::asset_key::AssetKey;
use crate::dag::{
    AnalyticType, DagError, Node, NodeId, NodeKey, NodeOutput, NodeParams, WindowSpec,
};
use crate::time_series::{DataProvider, DateRange, TimeSeriesPoint};
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, VecDeque};

fn parse_lag_from_map(params: &HashMap<String, String>) -> usize {
    params
        .get("lag")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1)
}

fn parse_window_from_map(params: &HashMap<String, String>) -> usize {
    params
        .get("window_size")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(10)
}

fn params_with_range(analytic_type: &str, range: &DateRange) -> HashMap<String, String> {
    let mut params = HashMap::new();
    params.insert("analytic_type".to_string(), analytic_type.to_string());
    params.insert("start_date".to_string(), range.start.to_string());
    params.insert("end_date".to_string(), range.end.to_string());
    params
}

/// Executor invoked for a node to perform pull or push calculations.
pub struct ParentOutput {
    pub node_id: NodeId,
    pub analytic: AnalyticType,
    pub output: Vec<TimeSeriesPoint>,
}

pub trait AnalyticExecutor: Send + Sync {
    fn execute_pull(
        &self,
        node: &Node,
        parent_outputs: &[ParentOutput],
        date_range: &DateRange,
        provider: &dyn DataProvider,
    ) -> Result<Vec<TimeSeriesPoint>, DagError>;

    fn execute_push(
        &self,
        node: &Node,
        parent_outputs: &[ParentOutput],
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
        definitions.insert(AnalyticType::Lag, Box::new(LagDefinition::new()));
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
            executor: Box::new(ReturnsExecutor::new(Box::new(LogReturnPrimitive))),
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
        let lag = parse_lag_from_map(&key.params);

        let data_params = params_with_range("data_provider", &range);
        let lag_params = {
            let mut params = params_with_range("lag", &range);
            params.insert("lag".to_string(), lag.to_string());
            params
        };

        Ok(vec![
            NodeKey {
                analytic: AnalyticType::DataProvider,
                assets: key.assets.clone(),
                range: Some(range.clone()),
                window: None,
                override_tag: key.override_tag.clone(),
                params: data_params,
            },
            NodeKey {
                analytic: AnalyticType::Lag,
                assets: key.assets.clone(),
                range: Some(range),
                window: Some(WindowSpec::fixed(lag + 1)),
                override_tag: key.override_tag.clone(),
                params: lag_params,
            },
        ])
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
            executor: Box::new(WindowedAnalyticExecutor::new(Box::new(
                StdDevVolatilityPrimitive,
            ))),
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
        let window_size = parse_window_from_map(&key.params);
        let returns_range = extend_range(&range, window_size.saturating_sub(1));

        let mut returns_params = params_with_range("returns", &returns_range);
        returns_params.insert("lag".to_string(), "1".to_string());

        Ok(vec![NodeKey {
            analytic: AnalyticType::Returns,
            assets: key.assets.clone(),
            range: Some(returns_range),
            window: None,
            override_tag: key.override_tag.clone(),
            params: returns_params,
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
        _parent_outputs: &[ParentOutput],
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
        _parent_outputs: &[ParentOutput],
        timestamp: DateTime<Utc>,
        value: f64,
    ) -> Result<NodeOutput, DagError> {
        Ok(NodeOutput::Single(vec![TimeSeriesPoint::new(
            timestamp, value,
        )]))
    }
}

struct LagDefinition {
    executor: Box<dyn AnalyticExecutor>,
}

impl LagDefinition {
    fn new() -> Self {
        LagDefinition {
            executor: Box::new(LagExecutor),
        }
    }
}

impl AnalyticDefinition for LagDefinition {
    fn analytic_type(&self) -> AnalyticType {
        AnalyticType::Lag
    }

    fn node_type(&self) -> &'static str {
        "lag"
    }

    fn dependencies(&self, key: &NodeKey) -> Result<Vec<NodeKey>, DagError> {
        let range = require_range(key)?;
        let lag = parse_lag_from_map(&key.params);
        let analytic = FixedLag::new(lag);
        let burn_in = analytic.required_points().saturating_sub(1);
        let provider_range = extend_range(&range, burn_in);

        let provider_params = params_with_range("data_provider", &provider_range);
        Ok(vec![NodeKey {
            analytic: AnalyticType::DataProvider,
            assets: key.assets.clone(),
            range: Some(provider_range),
            window: None,
            override_tag: key.override_tag.clone(),
            params: provider_params,
        }])
    }

    fn executor(&self) -> &dyn AnalyticExecutor {
        self.executor.as_ref()
    }
}

struct LagExecutor;

impl LagExecutor {
    fn lag_from_node(node: &Node) -> usize {
        if let NodeParams::Map(ref params) = node.params {
            parse_lag_from_map(params)
        } else {
            1
        }
    }

    fn build_series(&self, node: &Node, prices: &[TimeSeriesPoint]) -> Vec<TimeSeriesPoint> {
        let lag = Self::lag_from_node(node);
        let required = lag + 1;
        let analytic = FixedLag::new(lag);
        let mut window = VecDeque::new();
        let mut result = Vec::with_capacity(prices.len());

        for point in prices {
            window.push_front(point.close_price);
            if window.len() > required {
                window.pop_back();
            }

            let value = if window.len() == required {
                let values: Vec<f64> = window.iter().copied().collect();
                analytic.compute_lagged(&values).unwrap_or(f64::NAN)
            } else {
                f64::NAN
            };

            result.push(TimeSeriesPoint::new(point.timestamp, value));
        }

        result
    }
}

impl AnalyticExecutor for LagExecutor {
    fn execute_pull(
        &self,
        node: &Node,
        parent_outputs: &[ParentOutput],
        _date_range: &DateRange,
        _provider: &dyn DataProvider,
    ) -> Result<Vec<TimeSeriesPoint>, DagError> {
        let prices = parent_outputs
            .iter()
            .find(|parent| parent.analytic == AnalyticType::DataProvider)
            .map(|parent| parent.output.as_slice())
            .ok_or_else(|| {
                DagError::ExecutionError("Lag node requires input price data".to_string())
            })?;

        Ok(self.build_series(node, prices))
    }

    fn execute_push(
        &self,
        node: &Node,
        parent_outputs: &[ParentOutput],
        _timestamp: DateTime<Utc>,
        _value: f64,
    ) -> Result<NodeOutput, DagError> {
        let prices = parent_outputs
            .iter()
            .find(|parent| parent.analytic == AnalyticType::DataProvider)
            .map(|parent| parent.output.as_slice())
            .ok_or_else(|| {
                DagError::ExecutionError("Lag node requires input price data".to_string())
            })?;

        Ok(NodeOutput::Single(self.build_series(node, prices)))
    }
}

struct ReturnsExecutor {
    primitive: Box<dyn ReturnPrimitive>,
}

impl ReturnsExecutor {
    fn new(primitive: Box<dyn ReturnPrimitive>) -> Self {
        ReturnsExecutor { primitive }
    }

    fn lag_from_node(node: &Node) -> usize {
        if let NodeParams::Map(ref params) = node.params {
            parse_lag_from_map(params)
        } else {
            1
        }
    }

    fn build_lag_series(prices: &[TimeSeriesPoint], lag: usize) -> Vec<TimeSeriesPoint> {
        if prices.is_empty() {
            return Vec::new();
        }

        let analytic = FixedLag::new(lag);
        let required = analytic.required_points();
        let mut window = VecDeque::new();
        let mut result = Vec::with_capacity(prices.len());

        for point in prices {
            window.push_front(point.close_price);
            if window.len() > required {
                window.pop_back();
            }

            let value = if window.len() == required {
                let values: Vec<f64> = window.iter().copied().collect();
                analytic.compute_lagged(&values).unwrap_or(f64::NAN)
            } else {
                f64::NAN
            };
            result.push(TimeSeriesPoint::new(point.timestamp, value));
        }

        result
    }

    fn build_series(
        &self,
        asset: &AssetKey,
        prices: &[TimeSeriesPoint],
        lagged: &[TimeSeriesPoint],
    ) -> Vec<TimeSeriesPoint> {
        if prices.is_empty() || lagged.is_empty() {
            return Vec::new();
        }

        let length = std::cmp::min(prices.len(), lagged.len());
        let mut result = Vec::with_capacity(length);

        for (price, lag_point) in prices.iter().zip(lagged.iter()).take(length) {
            let value = if price.close_price.is_nan() || lag_point.close_price.is_nan() {
                f64::NAN
            } else {
                self.primitive
                    .compute(Some(asset), price.close_price, lag_point.close_price)
            };
            result.push(TimeSeriesPoint::new(price.timestamp, value));
        }

        result
    }

    fn build_update(
        &self,
        asset: &AssetKey,
        prices: &[TimeSeriesPoint],
        lagged: &[TimeSeriesPoint],
    ) -> Result<f64, DagError> {
        if prices.is_empty() || lagged.is_empty() {
            return Err(DagError::ExecutionError(
                "Returns update requires input price and lagged values".to_string(),
            ));
        }

        let price_point = prices.last().unwrap();
        let lag_point = lagged.last().unwrap();
        if price_point.close_price.is_nan() || lag_point.close_price.is_nan() {
            Ok(f64::NAN)
        } else {
            Ok(self
                .primitive
                .compute(Some(asset), price_point.close_price, lag_point.close_price))
        }
    }
}

impl AnalyticExecutor for ReturnsExecutor {
    fn execute_pull(
        &self,
        node: &Node,
        parent_outputs: &[ParentOutput],
        _date_range: &DateRange,
        _provider: &dyn DataProvider,
    ) -> Result<Vec<TimeSeriesPoint>, DagError> {
        let price_data = parent_outputs
            .iter()
            .find(|parent| parent.analytic == AnalyticType::DataProvider)
            .map(|parent| parent.output.as_slice())
            .ok_or_else(|| {
                DagError::ExecutionError("Returns node requires price input".to_string())
            })?;

        let mut _fallback_lag = Vec::new();
        let lag_data_slice = if let Some(lag_parent) = parent_outputs
            .iter()
            .find(|parent| parent.analytic == AnalyticType::Lag)
        {
            lag_parent.output.as_slice()
        } else {
            _fallback_lag = Self::build_lag_series(price_data, Self::lag_from_node(node));
            _fallback_lag.as_slice()
        };

        if price_data.is_empty() || lag_data_slice.is_empty() {
            return Ok(Vec::new());
        }

        let asset = node
            .assets
            .first()
            .ok_or_else(|| DagError::ExecutionError("Returns node missing asset".to_string()))?;

        Ok(self.build_series(asset, price_data, lag_data_slice))
    }

    fn execute_push(
        &self,
        node: &Node,
        parent_outputs: &[ParentOutput],
        _timestamp: DateTime<Utc>,
        _value: f64,
    ) -> Result<NodeOutput, DagError> {
        let price_data = parent_outputs
            .iter()
            .find(|parent| parent.analytic == AnalyticType::DataProvider)
            .map(|parent| parent.output.as_slice())
            .ok_or_else(|| {
                DagError::ExecutionError("Returns update requires price data".to_string())
            })?;

        let mut _fallback_lag = Vec::new();
        let lag_data_slice = if let Some(lag_parent) = parent_outputs
            .iter()
            .find(|parent| parent.analytic == AnalyticType::Lag)
        {
            lag_parent.output.as_slice()
        } else {
            _fallback_lag = Self::build_lag_series(price_data, Self::lag_from_node(node));
            _fallback_lag.as_slice()
        };

        let asset = node
            .assets
            .first()
            .ok_or_else(|| DagError::ExecutionError("Returns node missing asset".to_string()))?;

        let value = self.build_update(asset, price_data, lag_data_slice)?;
        Ok(NodeOutput::Scalar(value))
    }
}

struct WindowedAnalyticExecutor {
    primitive: Box<dyn VolatilityPrimitive>,
}

impl WindowedAnalyticExecutor {
    fn new(primitive: Box<dyn VolatilityPrimitive>) -> Self {
        WindowedAnalyticExecutor { primitive }
    }

    fn window_size(node: &Node) -> usize {
        if let NodeParams::Map(ref params) = node.params {
            parse_window_from_map(params)
        } else {
            10
        }
    }
}

impl AnalyticExecutor for WindowedAnalyticExecutor {
    fn execute_pull(
        &self,
        node: &Node,
        parent_outputs: &[ParentOutput],
        _date_range: &DateRange,
        _provider: &dyn DataProvider,
    ) -> Result<Vec<TimeSeriesPoint>, DagError> {
        let returns_data = parent_outputs
            .iter()
            .find(|parent| parent.analytic == AnalyticType::Returns)
            .map(|parent| parent.output.as_slice())
            .ok_or_else(|| {
                DagError::ExecutionError("Windowed analytic requires input data".to_string())
            })?;

        if returns_data.is_empty() {
            return Ok(Vec::new());
        }

        let window_size = Self::window_size(node);
        let closes = returns_data
            .iter()
            .map(|p| p.close_price)
            .collect::<Vec<_>>();
        let asset = node
            .assets
            .first()
            .ok_or_else(|| DagError::ExecutionError("Volatility node missing asset".to_string()))?;

        let mut result = Vec::with_capacity(returns_data.len());
        for (idx, point) in returns_data.iter().enumerate() {
            let start = idx.saturating_sub(window_size.saturating_sub(1));
            let value = self.primitive.compute(Some(asset), &closes[start..=idx]);
            result.push(TimeSeriesPoint::new(point.timestamp, value));
        }

        Ok(result)
    }

    fn execute_push(
        &self,
        node: &Node,
        parent_outputs: &[ParentOutput],
        _timestamp: DateTime<Utc>,
        _value: f64,
    ) -> Result<NodeOutput, DagError> {
        let returns_data = parent_outputs
            .iter()
            .find(|parent| parent.analytic == AnalyticType::Returns)
            .map(|parent| parent.output.as_slice())
            .ok_or_else(|| {
                DagError::ExecutionError("Windowed analytic update requires input data".to_string())
            })?;

        if returns_data.is_empty() {
            return Err(DagError::ExecutionError(
                "Windowed analytic update requires input data".to_string(),
            ));
        }

        let window_size = Self::window_size(node);
        let asset = node
            .assets
            .first()
            .ok_or_else(|| DagError::ExecutionError("Volatility node missing asset".to_string()))?;

        let closes: Vec<f64> = returns_data.iter().map(|point| point.close_price).collect();
        let start = closes.len().saturating_sub(window_size);
        let value = self.primitive.compute(Some(asset), &closes[start..]);
        Ok(NodeOutput::Scalar(value))
    }
}
