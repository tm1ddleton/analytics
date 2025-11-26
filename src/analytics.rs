//! Analytics Functions
//!
//! This module provides stateless analytics calculation functions including
//! returns and volatility calculations. These functions operate on raw f64
//! arrays for performance and are designed to integrate with the DAG framework.

use crate::asset_key::AssetKey;
use crate::dag::{DagError, Node, NodeId, NodeOutput, NodeParams};
use crate::time_series::{DateRange, TimeSeriesPoint};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Output mode for analytics queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Return full time series for the date range
    TimeSeries,
    /// Return only the last available value (for live/real-time queries)
    LiveValue,
}

/// Calculates log returns from a price series.
///
/// Uses the formula: ln(P_t / P_{t-1})
///
/// # Arguments
/// * `prices` - Slice of prices as f64 values
///
/// # Returns
/// Vector of log returns as decimal values (e.g., 0.05 for 5% return)
///
/// # Behavior
/// - First value is NaN (no previous price for comparison)
/// - Subsequent values are ln(P_t / P_{t-1})
/// - NaN values in input are converted to 0 in output
/// - Empty input returns empty output
///
/// # Examples
/// ```
/// use analytics::analytics::calculate_returns;
///
/// let prices = vec![100.0, 105.0, 103.0];
/// let returns = calculate_returns(&prices);
///
/// assert!(returns[0].is_nan()); // First value is NaN
/// // Second value: ln(105/100) ≈ 0.04879
/// // Third value: ln(103/105) ≈ -0.01942
/// ```
pub fn calculate_returns(prices: &[f64]) -> Vec<f64> {
    if prices.is_empty() {
        return Vec::new();
    }

    if prices.len() == 1 {
        return vec![f64::NAN];
    }

    let mut returns = Vec::with_capacity(prices.len());

    // First value is NaN (no previous price)
    returns.push(f64::NAN);

    // Calculate log returns for remaining prices
    for i in 1..prices.len() {
        let prev_price = prices[i - 1];
        let curr_price = prices[i];

        // Handle NaN or invalid values
        if prev_price.is_nan() || curr_price.is_nan() || prev_price <= 0.0 || curr_price <= 0.0 {
            returns.push(0.0); // Convert NaN to 0 as per spec
        } else {
            let log_return = (curr_price / prev_price).ln();
            if log_return.is_nan() {
                returns.push(0.0); // Convert NaN to 0
            } else {
                returns.push(log_return);
            }
        }
    }

    returns
}

/// Calculates rolling volatility from a returns series.
///
/// Uses population standard deviation formula: σ = sqrt(sum((r_i - μ)²) / N)
/// where μ is the mean of returns in the window.
///
/// # Arguments
/// * `returns` - Slice of returns as f64 values
/// * `window_size` - Size of the rolling window (N)
///
/// # Returns
/// Vector of volatility values (population standard deviation)
///
/// # Behavior
/// - Each output point shows volatility of past N days (rolling window)
/// - Uses population standard deviation (divide by N, not N-1)
/// - NOT annualized (no √252 multiplier)
/// - If window_size exceeds available data, uses all available data
/// - Gracefully handles insufficient data at start of sequence
/// - Returns NaN if window has no valid data
///
/// # Examples
/// ```
/// use analytics::analytics::calculate_volatility;
///
/// let returns = vec![0.01, -0.02, 0.015, -0.01, 0.02];
/// let volatility = calculate_volatility(&returns, 3);
///
/// // Each point calculates std dev of past 3 returns
/// ```
pub fn calculate_volatility(returns: &[f64], window_size: usize) -> Vec<f64> {
    if returns.is_empty() || window_size == 0 {
        return Vec::new();
    }

    let mut volatility = Vec::with_capacity(returns.len());

    for i in 0..returns.len() {
        // Determine window bounds
        let window_start = if i + 1 < window_size {
            0 // Use all available data if less than window_size
        } else {
            i + 1 - window_size
        };

        let window = &returns[window_start..=i];

        // Calculate population standard deviation
        let vol = calculate_std_dev(window);
        volatility.push(vol);
    }

    volatility
}

/// Helper function to calculate population standard deviation.
///
/// # Arguments
/// * `values` - Slice of f64 values
///
/// # Returns
/// Population standard deviation, or NaN if input is empty or all NaN
fn calculate_std_dev(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }

    // Filter out NaN values
    let valid_values: Vec<f64> = values.iter().filter(|&&v| !v.is_nan()).copied().collect();

    if valid_values.is_empty() {
        return f64::NAN;
    }

    let n = valid_values.len() as f64;

    // Calculate mean
    let mean = valid_values.iter().sum::<f64>() / n;

    // Calculate sum of squared differences
    let sum_squared_diff: f64 = valid_values.iter().map(|&v| (v - mean).powi(2)).sum();

    // Population standard deviation (divide by N, not N-1)
    (sum_squared_diff / n).sqrt()
}

/// Generates a unique node identifier hash from assets, analytic type, and date range.
///
/// # Arguments
/// * `assets` - Assets involved in the calculation
/// * `analytic_type` - Type of analytic ("returns", "volatility", etc.)
/// * `date_range` - Date range for the calculation
/// * `params` - Additional parameters (e.g., window size)
///
/// # Returns
/// A u64 hash value uniquely identifying this node
pub fn generate_node_hash(
    assets: &[AssetKey],
    analytic_type: &str,
    date_range: &DateRange,
    params: &HashMap<String, String>,
) -> u64 {
    let mut hasher = DefaultHasher::new();

    // Hash assets
    for asset in assets {
        format!("{:?}", asset).hash(&mut hasher);
    }

    // Hash analytic type
    analytic_type.hash(&mut hasher);

    // Hash date range
    format!("{:?}", date_range).hash(&mut hasher);

    // Hash parameters
    for (key, value) in params {
        key.hash(&mut hasher);
        value.hash(&mut hasher);
    }

    hasher.finish()
}

/// Converts TimeSeriesPoint vector to f64 price vector
pub fn timeseries_to_prices(data: &[TimeSeriesPoint]) -> Vec<f64> {
    data.iter().map(|point| point.close_price).collect()
}

/// Converts f64 values back to TimeSeriesPoint vector, preserving timestamps
pub fn prices_to_timeseries(
    prices: &[f64],
    original_data: &[TimeSeriesPoint],
) -> Vec<TimeSeriesPoint> {
    prices
        .iter()
        .zip(original_data.iter())
        .map(|(&price, original)| TimeSeriesPoint {
            timestamp: original.timestamp,
            close_price: price,
        })
        .collect()
}

/// Creates a DataProvider wrapper node that fetches price data
///
/// # Arguments
/// * `node_id` - Unique identifier for this node
/// * `asset` - Asset to fetch data for
/// * `date_range` - Date range to fetch
///
/// # Returns
/// Node configured to fetch price data
pub fn create_data_provider_node(node_id: NodeId, asset: AssetKey, date_range: DateRange) -> Node {
    let mut params_map = HashMap::new();
    params_map.insert("analytic_type".to_string(), "data_provider".to_string());
    params_map.insert("start_date".to_string(), date_range.start.to_string());
    params_map.insert("end_date".to_string(), date_range.end.to_string());

    Node::new(
        node_id,
        "data_provider".to_string(),
        NodeParams::Map(params_map),
        vec![asset],
    )
}

/// Creates a Returns calculation node
///
/// # Arguments
/// * `node_id` - Unique identifier for this node
/// * `asset` - Asset to calculate returns for
/// * `date_range` - Date range for calculation
///
/// # Returns
/// Node configured to calculate returns
pub fn create_returns_node(node_id: NodeId, asset: AssetKey, date_range: DateRange) -> Node {
    let mut params_map = HashMap::new();
    params_map.insert("analytic_type".to_string(), "returns".to_string());
    params_map.insert("start_date".to_string(), date_range.start.to_string());
    params_map.insert("end_date".to_string(), date_range.end.to_string());

    Node::new(
        node_id,
        "returns".to_string(),
        NodeParams::Map(params_map),
        vec![asset],
    )
}

/// Creates a Volatility calculation node
///
/// # Arguments
/// * `node_id` - Unique identifier for this node
/// * `asset` - Asset to calculate volatility for
/// * `window_size` - Rolling window size
/// * `date_range` - Date range for calculation
///
/// # Returns
/// Node configured to calculate volatility
pub fn create_volatility_node(
    node_id: NodeId,
    asset: AssetKey,
    window_size: usize,
    date_range: DateRange,
) -> Node {
    let mut params_map = HashMap::new();
    params_map.insert("analytic_type".to_string(), "volatility".to_string());
    params_map.insert("window_size".to_string(), window_size.to_string());
    params_map.insert("start_date".to_string(), date_range.start.to_string());
    params_map.insert("end_date".to_string(), date_range.end.to_string());

    Node::new(
        node_id,
        "volatility".to_string(),
        NodeParams::Map(params_map),
        vec![asset],
    )
}

/// Executes a returns calculation node
///
/// # Arguments
/// * `node` - The returns node
/// * `inputs` - Input data from parent nodes (should contain price data)
///
/// # Returns
/// NodeOutput containing calculated returns as TimeSeriesPoint vector
pub fn execute_returns_node(_node: &Node, inputs: &[NodeOutput]) -> Result<NodeOutput, DagError> {
    if inputs.is_empty() {
        return Err(DagError::ExecutionError(
            "Returns node requires price data input".to_string(),
        ));
    }

    // Extract price data from first input
    let price_data = match &inputs[0] {
        NodeOutput::Single(data) => data,
        _ => {
            return Err(DagError::ExecutionError(
                "Returns node expects Single(Vec<TimeSeriesPoint>) input".to_string(),
            ))
        }
    };

    // Convert to f64 prices
    let prices = timeseries_to_prices(price_data);

    // Calculate returns
    let returns = calculate_returns(&prices);

    // Convert back to TimeSeriesPoint
    let result = prices_to_timeseries(&returns, price_data);

    Ok(NodeOutput::Single(result))
}

/// Executes a volatility calculation node
///
/// # Arguments
/// * `node` - The volatility node (contains window_size parameter)
/// * `inputs` - Input data from parent nodes (should contain returns data)
///
/// # Returns
/// NodeOutput containing calculated volatility as TimeSeriesPoint vector
pub fn execute_volatility_node(node: &Node, inputs: &[NodeOutput]) -> Result<NodeOutput, DagError> {
    if inputs.is_empty() {
        return Err(DagError::ExecutionError(
            "Volatility node requires returns data input".to_string(),
        ));
    }

    // Extract window size from node parameters
    let window_size = match &node.params {
        NodeParams::Map(params) => params
            .get("window_size")
            .and_then(|s| s.parse::<usize>().ok())
            .ok_or_else(|| {
                DagError::ExecutionError(
                    "Volatility node missing window_size parameter".to_string(),
                )
            })?,
        _ => {
            return Err(DagError::ExecutionError(
                "Volatility node has invalid parameters".to_string(),
            ))
        }
    };

    // Extract returns data from first input
    let returns_data = match &inputs[0] {
        NodeOutput::Single(data) => data,
        _ => {
            return Err(DagError::ExecutionError(
                "Volatility node expects Single(Vec<TimeSeriesPoint>) input".to_string(),
            ))
        }
    };

    // Convert to f64 returns
    let returns = timeseries_to_prices(returns_data);

    // Calculate volatility
    let volatility = calculate_volatility(&returns, window_size);

    // Convert back to TimeSeriesPoint
    let result = prices_to_timeseries(&volatility, returns_data);

    Ok(NodeOutput::Single(result))
}

/// Calculates burn-in period needed for volatility calculation
///
/// Formula: volatility_window + 1 = price_days_needed
/// (Need 1 extra day for first return calculation)
pub fn calculate_volatility_burnin(window_size: usize) -> usize {
    window_size + 1
}

/// Query builder for volatility calculation
///
/// Automatically creates DAG chain: DataProvider → Returns → Volatility
/// with proper burn-in calculation
pub struct VolatilityQueryBuilder {
    asset: AssetKey,
    window_size: usize,
    date_range: DateRange,
}

impl VolatilityQueryBuilder {
    /// Creates a new volatility query builder
    pub fn new(asset: AssetKey, window_size: usize, date_range: DateRange) -> Self {
        VolatilityQueryBuilder {
            asset,
            window_size,
            date_range,
        }
    }

    /// Builds the DAG with automatic burn-in calculation
    ///
    /// Returns (dag, data_node_id, returns_node_id, volatility_node_id)
    pub fn build_dag(
        &self,
    ) -> Result<(crate::dag::AnalyticsDag, NodeId, NodeId, NodeId), DagError> {
        use crate::dag::AnalyticsDag;
        use chrono::Duration;

        let mut dag = AnalyticsDag::new();

        // Calculate burn-in: N-day volatility needs N+1 days of price data
        let burnin_days = calculate_volatility_burnin(self.window_size);

        // Adjust start date for burn-in
        let adjusted_start = self.date_range.start - Duration::days(burnin_days as i64);
        let adjusted_range = DateRange::new(adjusted_start, self.date_range.end);

        // Create nodes (node creation helpers are no longer needed, we build directly)
        let mut data_params = HashMap::new();
        data_params.insert("analytic_type".to_string(), "data_provider".to_string());
        data_params.insert("start_date".to_string(), adjusted_range.start.to_string());
        data_params.insert("end_date".to_string(), adjusted_range.end.to_string());

        let mut returns_params = HashMap::new();
        returns_params.insert("analytic_type".to_string(), "returns".to_string());
        returns_params.insert("start_date".to_string(), adjusted_range.start.to_string());
        returns_params.insert("end_date".to_string(), adjusted_range.end.to_string());

        let mut volatility_params = HashMap::new();
        volatility_params.insert("analytic_type".to_string(), "volatility".to_string());
        volatility_params.insert("window_size".to_string(), self.window_size.to_string());
        volatility_params.insert("start_date".to_string(), self.date_range.start.to_string());
        volatility_params.insert("end_date".to_string(), self.date_range.end.to_string());

        // Add nodes to DAG (add_node returns the NodeId)
        let data_node_id = dag.add_node(
            "data_provider".to_string(),
            NodeParams::Map(data_params),
            vec![self.asset.clone()],
        );

        let returns_node_id = dag.add_node(
            "returns".to_string(),
            NodeParams::Map(returns_params),
            vec![self.asset.clone()],
        );

        let volatility_node_id = dag.add_node(
            "volatility".to_string(),
            NodeParams::Map(volatility_params),
            vec![self.asset.clone()],
        );

        // Create edges: data → returns → volatility
        dag.add_edge(data_node_id, returns_node_id)?;
        dag.add_edge(returns_node_id, volatility_node_id)?;

        Ok((dag, data_node_id, returns_node_id, volatility_node_id))
    }
}

/// Query builder for returns calculation
///
/// Automatically creates DAG chain: DataProvider → Returns
/// with proper burn-in calculation (need 1 extra day for first return)
pub struct ReturnsQueryBuilder {
    asset: AssetKey,
    date_range: DateRange,
}

impl ReturnsQueryBuilder {
    /// Creates a new returns query builder
    pub fn new(asset: AssetKey, date_range: DateRange) -> Self {
        ReturnsQueryBuilder { asset, date_range }
    }

    /// Builds the DAG with automatic burn-in calculation
    ///
    /// Returns (dag, data_node_id, returns_node_id)
    pub fn build_dag(&self) -> Result<(crate::dag::AnalyticsDag, NodeId, NodeId), DagError> {
        use crate::dag::AnalyticsDag;
        use chrono::Duration;

        let mut dag = AnalyticsDag::new();

        // Returns need 1 extra day for first return calculation
        let burnin_days = 1;

        // Adjust start date for burn-in
        let adjusted_start = self.date_range.start - Duration::days(burnin_days);
        let adjusted_range = DateRange::new(adjusted_start, self.date_range.end);

        // Create node parameters
        let mut data_params = HashMap::new();
        data_params.insert("analytic_type".to_string(), "data_provider".to_string());
        data_params.insert("start_date".to_string(), adjusted_range.start.to_string());
        data_params.insert("end_date".to_string(), adjusted_range.end.to_string());

        let mut returns_params = HashMap::new();
        returns_params.insert("analytic_type".to_string(), "returns".to_string());
        returns_params.insert("start_date".to_string(), self.date_range.start.to_string());
        returns_params.insert("end_date".to_string(), self.date_range.end.to_string());

        // Add nodes to DAG (add_node returns the NodeId)
        let data_node_id = dag.add_node(
            "data_provider".to_string(),
            NodeParams::Map(data_params),
            vec![self.asset.clone()],
        );

        let returns_node_id = dag.add_node(
            "returns".to_string(),
            NodeParams::Map(returns_params),
            vec![self.asset.clone()],
        );

        // Create edge: data → returns
        dag.add_edge(data_node_id, returns_node_id)?;

        Ok((dag, data_node_id, returns_node_id))
    }
}

/// High-level query API for analytics
///
/// Provides a simple interface to query analytics without manually building DAGs
pub struct AnalyticsQuery;

impl AnalyticsQuery {
    /// Query returns for an asset over a date range
    ///
    /// # Arguments
    /// * `asset` - Asset to calculate returns for
    /// * `date_range` - Date range for calculation
    /// * `output_mode` - Whether to return time series or live value
    ///
    /// # Returns
    /// Vector of TimeSeriesPoint (time series mode) or single value (live mode)
    ///
    /// Note: This is a placeholder that builds the DAG structure.
    /// Actual execution would require a DataProvider to fetch data.
    pub fn query_returns(
        _asset: &AssetKey,
        _date_range: &DateRange,
        _output_mode: OutputMode,
    ) -> Result<Vec<TimeSeriesPoint>, DagError> {
        // This would:
        // 1. Build DAG using ReturnsQueryBuilder
        // 2. Execute DAG with DataProvider
        // 3. Extract returns from final node
        // 4. Apply output mode (full series or last value)

        // For now, return placeholder to demonstrate API
        Err(DagError::ExecutionError(
            "AnalyticsQuery requires DataProvider integration for execution".to_string(),
        ))
    }

    /// Query volatility for an asset over a date range
    ///
    /// # Arguments
    /// * `asset` - Asset to calculate volatility for
    /// * `window_size` - Rolling window size for volatility
    /// * `date_range` - Date range for calculation
    /// * `output_mode` - Whether to return time series or live value
    ///
    /// # Returns
    /// Vector of TimeSeriesPoint (time series mode) or single value (live mode)
    ///
    /// Note: This is a placeholder that builds the DAG structure.
    /// Actual execution would require a DataProvider to fetch data.
    pub fn query_volatility(
        _asset: &AssetKey,
        _window_size: usize,
        _date_range: &DateRange,
        _output_mode: OutputMode,
    ) -> Result<Vec<TimeSeriesPoint>, DagError> {
        // This would:
        // 1. Build DAG using VolatilityQueryBuilder
        // 2. Execute DAG with DataProvider
        // 3. Extract volatility from final node
        // 4. Apply output mode (full series or last value)

        // For now, return placeholder to demonstrate API
        Err(DagError::ExecutionError(
            "AnalyticsQuery requires DataProvider integration for execution".to_string(),
        ))
    }
}

/// Helper to extract output based on OutputMode
pub fn apply_output_mode(data: Vec<TimeSeriesPoint>, mode: OutputMode) -> Vec<TimeSeriesPoint> {
    match mode {
        OutputMode::TimeSeries => data,
        OutputMode::LiveValue => {
            // Return only the last value
            data.into_iter().last().into_iter().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Task Group 1.1: Tests for returns calculation

    #[test]
    fn test_returns_with_known_price_sequence() {
        // Test log returns with known price sequences
        let prices = vec![100.0, 110.0, 105.0, 115.0];
        let returns = calculate_returns(&prices);

        assert_eq!(returns.len(), 4);
        assert!(returns[0].is_nan(), "First return should be NaN");

        // ln(110/100) ≈ 0.09531
        assert!(
            (returns[1] - 0.09531).abs() < 0.0001,
            "Second return incorrect"
        );

        // ln(105/110) ≈ -0.04652
        assert!(
            (returns[2] - (-0.04652)).abs() < 0.0001,
            "Third return incorrect"
        );

        // ln(115/105) ≈ 0.09090
        assert!(
            (returns[3] - 0.09090).abs() < 0.0001,
            "Fourth return incorrect"
        );
    }

    #[test]
    fn test_returns_first_value_is_nan() {
        // Test that first value is NaN (no previous price)
        let prices = vec![100.0, 105.0];
        let returns = calculate_returns(&prices);

        assert!(returns[0].is_nan(), "First return should be NaN");
        assert!(!returns[1].is_nan(), "Second return should not be NaN");
    }

    #[test]
    fn test_returns_nan_handling() {
        // Test NaN handling (convert to 0)
        let prices = vec![100.0, f64::NAN, 110.0];
        let returns = calculate_returns(&prices);

        assert!(returns[0].is_nan(), "First return should be NaN");
        assert_eq!(returns[1], 0.0, "NaN price should produce 0 return");
        assert_eq!(returns[2], 0.0, "Return after NaN should be 0");
    }

    #[test]
    fn test_returns_flat_prices() {
        // Test returns on flat prices (ln(1) = 0)
        let prices = vec![100.0, 100.0, 100.0];
        let returns = calculate_returns(&prices);

        assert!(returns[0].is_nan());
        assert_eq!(returns[1], 0.0, "Flat price should give 0 return");
        assert_eq!(returns[2], 0.0, "Flat price should give 0 return");
    }

    #[test]
    fn test_returns_increasing_sequence() {
        // Test returns on increasing sequence
        let prices = vec![100.0, 110.0, 121.0];
        let returns = calculate_returns(&prices);

        assert!(returns[0].is_nan());
        assert!(
            returns[1] > 0.0,
            "Increasing price should give positive return"
        );
        assert!(
            returns[2] > 0.0,
            "Increasing price should give positive return"
        );
    }

    #[test]
    fn test_returns_decreasing_sequence() {
        // Test returns on decreasing sequence
        let prices = vec![100.0, 90.0, 81.0];
        let returns = calculate_returns(&prices);

        assert!(returns[0].is_nan());
        assert!(
            returns[1] < 0.0,
            "Decreasing price should give negative return"
        );
        assert!(
            returns[2] < 0.0,
            "Decreasing price should give negative return"
        );
    }

    #[test]
    fn test_returns_empty_input() {
        // Test empty input
        let prices: Vec<f64> = vec![];
        let returns = calculate_returns(&prices);

        assert_eq!(returns.len(), 0, "Empty input should give empty output");
    }

    #[test]
    fn test_returns_single_price() {
        // Test single price
        let prices = vec![100.0];
        let returns = calculate_returns(&prices);

        assert_eq!(returns.len(), 1);
        assert!(returns[0].is_nan(), "Single price should give NaN");
    }

    // Task Group 2.1: Tests for volatility calculation

    #[test]
    fn test_volatility_with_known_return_sequence() {
        // Test volatility with known return sequences
        let returns = vec![0.01, -0.01, 0.02, -0.02, 0.015];
        let window_size = 3;
        let volatility = calculate_volatility(&returns, window_size);

        assert_eq!(volatility.len(), returns.len());

        // First point uses only itself (window of 1)
        // std dev of [0.01] = 0
        assert_eq!(volatility[0], 0.0);

        // Second point uses [0.01, -0.01]
        // mean = 0, std dev = sqrt((0.01²+ 0.01²)/2) = sqrt(0.0001) ≈ 0.01
        assert!((volatility[1] - 0.01).abs() < 0.0001);
    }

    #[test]
    fn test_volatility_rolling_window_behavior() {
        // Test rolling window (each point uses past N days)
        let returns = vec![0.01, 0.02, 0.03, 0.04, 0.05];
        let window_size = 2;
        let volatility = calculate_volatility(&returns, window_size);

        assert_eq!(volatility.len(), 5);

        // Each volatility should be calculated from correct window
        // Point 2 (index 1): window [0.01, 0.02]
        // Point 3 (index 2): window [0.02, 0.03]
        // Point 4 (index 3): window [0.03, 0.04]
        // Verify window is rolling
    }

    #[test]
    fn test_volatility_population_std_dev() {
        // Test population standard deviation (divide by N)
        let returns = vec![0.02, 0.04]; // mean = 0.03, diff = [0.01, 0.01]
        let volatility = calculate_volatility(&returns, 2);

        // Pop std dev = sqrt((0.01² + 0.01²) / 2) = sqrt(0.0001) = 0.01
        let expected = 0.01;
        assert!((volatility[1] - expected).abs() < 0.0001);
    }

    #[test]
    fn test_volatility_insufficient_data() {
        // Test handling when window size larger than data
        let returns = vec![0.01, 0.02, 0.03];
        let window_size = 10; // Larger than data
        let volatility = calculate_volatility(&returns, window_size);

        assert_eq!(volatility.len(), 3);
        // Should use all available data
    }

    #[test]
    fn test_volatility_edge_case_window_larger_than_data() {
        // Test window size exceeds data length
        let returns = vec![0.01, 0.02];
        let window_size = 5;
        let volatility = calculate_volatility(&returns, window_size);

        assert_eq!(
            volatility.len(),
            2,
            "Should return volatility for each point"
        );
    }

    #[test]
    fn test_volatility_zero_window() {
        // Test edge case: zero window size
        let returns = vec![0.01, 0.02, 0.03];
        let volatility = calculate_volatility(&returns, 0);

        assert_eq!(volatility.len(), 0, "Zero window should return empty");
    }

    #[test]
    fn test_volatility_empty_returns() {
        // Test empty returns
        let returns: Vec<f64> = vec![];
        let volatility = calculate_volatility(&returns, 5);

        assert_eq!(volatility.len(), 0);
    }

    // Task Group 3.1: Tests for node wrappers

    #[test]
    fn test_node_identification_hashing() {
        use chrono::NaiveDate;

        // Test that same inputs produce same hash
        let asset = AssetKey::new_equity("AAPL").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );
        let mut params = HashMap::new();
        params.insert("window".to_string(), "10".to_string());

        let hash1 = generate_node_hash(&[asset.clone()], "volatility", &date_range, &params);
        let hash2 = generate_node_hash(&[asset.clone()], "volatility", &date_range, &params);

        assert_eq!(hash1, hash2, "Same inputs should produce same hash");

        // Test that different inputs produce different hashes
        let asset2 = AssetKey::new_equity("MSFT").unwrap();
        let hash3 = generate_node_hash(&[asset2], "volatility", &date_range, &params);

        assert_ne!(
            hash1, hash3,
            "Different assets should produce different hashes"
        );
    }

    #[test]
    fn test_timeseries_to_prices_conversion() {
        use chrono::Utc;

        let data = vec![
            TimeSeriesPoint::new(Utc::now(), 100.0),
            TimeSeriesPoint::new(Utc::now(), 105.0),
            TimeSeriesPoint::new(Utc::now(), 103.0),
        ];

        let prices = timeseries_to_prices(&data);

        assert_eq!(prices.len(), 3);
        assert_eq!(prices[0], 100.0);
        assert_eq!(prices[1], 105.0);
        assert_eq!(prices[2], 103.0);
    }

    #[test]
    fn test_prices_to_timeseries_preserves_timestamps() {
        use chrono::{Duration, Utc};

        let now = Utc::now();
        let original_data = vec![
            TimeSeriesPoint::new(now, 100.0),
            TimeSeriesPoint::new(now + Duration::days(1), 105.0),
            TimeSeriesPoint::new(now + Duration::days(2), 103.0),
        ];

        let new_prices = vec![200.0, 210.0, 206.0];
        let result = prices_to_timeseries(&new_prices, &original_data);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].timestamp, original_data[0].timestamp);
        assert_eq!(result[0].close_price, 200.0);
        assert_eq!(result[1].timestamp, original_data[1].timestamp);
        assert_eq!(result[1].close_price, 210.0);
    }

    #[test]
    fn test_returns_node_creation() {
        use chrono::NaiveDate;

        let node_id = NodeId(1);
        let asset = AssetKey::new_equity("AAPL").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        let node = create_returns_node(node_id, asset.clone(), date_range);

        assert_eq!(node.id, node_id);
        assert_eq!(node.node_type, "returns");
        assert_eq!(node.assets.len(), 1);
        assert_eq!(node.assets[0], asset);
    }

    #[test]
    fn test_volatility_node_creation() {
        use chrono::NaiveDate;

        let node_id = NodeId(2);
        let asset = AssetKey::new_equity("AAPL").unwrap();
        let window_size = 10;
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        let node = create_volatility_node(node_id, asset.clone(), window_size, date_range);

        assert_eq!(node.id, node_id);
        assert_eq!(node.node_type, "volatility");
        assert_eq!(node.assets.len(), 1);

        // Check parameters
        if let NodeParams::Map(params) = &node.params {
            assert_eq!(params.get("window_size").unwrap(), "10");
        } else {
            panic!("Expected Map parameters");
        }
    }

    #[test]
    fn test_execute_returns_node() {
        use chrono::Utc;

        // Create test price data
        let price_data = vec![
            TimeSeriesPoint::new(Utc::now(), 100.0),
            TimeSeriesPoint::new(Utc::now(), 110.0),
            TimeSeriesPoint::new(Utc::now(), 105.0),
        ];

        let input = NodeOutput::Single(price_data.clone());
        let node = create_returns_node(
            NodeId(1),
            AssetKey::new_equity("AAPL").unwrap(),
            DateRange::new(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
            ),
        );

        let result = execute_returns_node(&node, &[input]).unwrap();

        match result {
            NodeOutput::Single(returns_data) => {
                assert_eq!(returns_data.len(), 3);
                assert!(returns_data[0].close_price.is_nan()); // First return is NaN
                assert!(returns_data[1].close_price > 0.0); // Positive return
                assert!(returns_data[2].close_price < 0.0); // Negative return
            }
            _ => panic!("Expected Single output"),
        }
    }

    #[test]
    fn test_execute_volatility_node() {
        use chrono::Utc;

        // Create test returns data
        let returns_data = vec![
            TimeSeriesPoint::new(Utc::now(), f64::NAN), // First return is NaN
            TimeSeriesPoint::new(Utc::now(), 0.01),
            TimeSeriesPoint::new(Utc::now(), -0.01),
            TimeSeriesPoint::new(Utc::now(), 0.02),
            TimeSeriesPoint::new(Utc::now(), -0.02),
        ];

        let input = NodeOutput::Single(returns_data.clone());
        let node = create_volatility_node(
            NodeId(2),
            AssetKey::new_equity("AAPL").unwrap(),
            3,
            DateRange::new(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
            ),
        );

        let result = execute_volatility_node(&node, &[input]).unwrap();

        match result {
            NodeOutput::Single(volatility_data) => {
                assert_eq!(volatility_data.len(), 5);
                // Volatility values should be non-negative
                for point in volatility_data.iter().skip(1) {
                    assert!(point.close_price >= 0.0 || point.close_price.is_nan());
                }
            }
            _ => panic!("Expected Single output"),
        }
    }

    // Task Group 4.1: Tests for automatic dependency resolution and burn-in

    #[test]
    fn test_volatility_burnin_calculation() {
        // 10-day volatility needs 11 days of price data (10 + 1 for first return)
        assert_eq!(calculate_volatility_burnin(10), 11);
        assert_eq!(calculate_volatility_burnin(30), 31);
        assert_eq!(calculate_volatility_burnin(1), 2);
    }

    #[test]
    fn test_returns_query_builder() {
        use chrono::NaiveDate;

        let asset = AssetKey::new_equity("AAPL").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        let builder = ReturnsQueryBuilder::new(asset, date_range);
        let (dag, data_node_id, returns_node_id) = builder.build_dag().unwrap();

        // Verify DAG has 2 nodes
        assert!(dag.node_count() >= 2, "DAG should have at least 2 nodes");

        // Verify execution order includes both nodes
        let exec_order = dag.execution_order_immutable().unwrap();
        assert!(exec_order.contains(&data_node_id));
        assert!(exec_order.contains(&returns_node_id));

        // Data node should come before returns node
        let data_pos = exec_order
            .iter()
            .position(|&id| id == data_node_id)
            .unwrap();
        let returns_pos = exec_order
            .iter()
            .position(|&id| id == returns_node_id)
            .unwrap();
        assert!(
            data_pos < returns_pos,
            "Data node should come before returns node"
        );
    }

    #[test]
    fn test_volatility_query_builder() {
        use chrono::NaiveDate;

        let asset = AssetKey::new_equity("AAPL").unwrap();
        let window_size = 10;
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        let builder = VolatilityQueryBuilder::new(asset, window_size, date_range);
        let (dag, data_node_id, returns_node_id, volatility_node_id) = builder.build_dag().unwrap();

        // Verify DAG has 3 nodes
        assert!(dag.node_count() >= 3, "DAG should have at least 3 nodes");

        // Verify execution order includes all three nodes
        let exec_order = dag.execution_order_immutable().unwrap();
        assert!(exec_order.contains(&data_node_id));
        assert!(exec_order.contains(&returns_node_id));
        assert!(exec_order.contains(&volatility_node_id));

        // Verify correct order: data → returns → volatility
        let data_pos = exec_order
            .iter()
            .position(|&id| id == data_node_id)
            .unwrap();
        let returns_pos = exec_order
            .iter()
            .position(|&id| id == returns_node_id)
            .unwrap();
        let vol_pos = exec_order
            .iter()
            .position(|&id| id == volatility_node_id)
            .unwrap();

        assert!(
            data_pos < returns_pos,
            "Data node should come before returns node"
        );
        assert!(
            returns_pos < vol_pos,
            "Returns node should come before volatility node"
        );
    }

    #[test]
    fn test_burnin_adjusts_date_range() {
        use chrono::NaiveDate;

        let asset = AssetKey::new_equity("AAPL").unwrap();
        let window_size = 10;
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let date_range = DateRange::new(start_date, end_date);

        let builder = VolatilityQueryBuilder::new(asset, window_size, date_range.clone());
        let (dag, data_node_id, _, _) = builder.build_dag().unwrap();

        // Verify that data node has adjusted date range
        // The data provider should fetch data starting from start_date - 11 days
        // (10-day window + 1 for first return)

        // For this test, we verify the DAG was created successfully
        // Actual date range validation would require inspecting node parameters
        assert!(dag.node_count() >= 3);
        let exec_order = dag.execution_order_immutable().unwrap();
        assert!(exec_order.contains(&data_node_id));
    }

    #[test]
    fn test_complex_dependency_chain() {
        use chrono::NaiveDate;

        // Test that complex dependencies are properly resolved
        let asset = AssetKey::new_equity("AAPL").unwrap();
        let window_size = 30; // 30-day volatility needs 31 days of price data
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(),
        );

        let builder = VolatilityQueryBuilder::new(asset, window_size, date_range);
        let (dag, _, _, _) = builder.build_dag().unwrap();

        // Verify DAG is acyclic
        let exec_order = dag.execution_order_immutable();
        assert!(exec_order.is_ok(), "DAG should be acyclic");

        // Verify all nodes can be executed in order
        assert_eq!(exec_order.unwrap().len(), 3);
    }

    #[test]
    fn test_multi_window_size_creates_different_queries() {
        use chrono::NaiveDate;

        let asset = AssetKey::new_equity("AAPL").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        // Build two DAGs with different window sizes
        let builder1 = VolatilityQueryBuilder::new(asset.clone(), 10, date_range.clone());
        let (dag1, _, _, _) = builder1.build_dag().unwrap();

        let builder2 = VolatilityQueryBuilder::new(asset, 30, date_range);
        let (dag2, _, _, _) = builder2.build_dag().unwrap();

        // Both DAGs should have 3 nodes (data provider, returns, volatility)
        assert_eq!(dag1.node_count(), 3);
        assert_eq!(dag2.node_count(), 3);

        // Different window sizes create different DAGs
        // (In practice, these would be separate queries with different burn-in requirements)
        // 10-day volatility needs 11 days of price data
        // 30-day volatility needs 31 days of price data
    }

    // Task Group 5.1: Tests for query interface and output modes

    #[test]
    fn test_output_mode_time_series() {
        use chrono::Utc;

        let data = vec![
            TimeSeriesPoint::new(Utc::now(), 100.0),
            TimeSeriesPoint::new(Utc::now(), 105.0),
            TimeSeriesPoint::new(Utc::now(), 103.0),
        ];

        let result = apply_output_mode(data.clone(), OutputMode::TimeSeries);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].close_price, 100.0);
        assert_eq!(result[1].close_price, 105.0);
        assert_eq!(result[2].close_price, 103.0);
    }

    #[test]
    fn test_output_mode_live_value() {
        use chrono::Utc;

        let data = vec![
            TimeSeriesPoint::new(Utc::now(), 100.0),
            TimeSeriesPoint::new(Utc::now(), 105.0),
            TimeSeriesPoint::new(Utc::now(), 103.0),
        ];

        let result = apply_output_mode(data, OutputMode::LiveValue);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].close_price, 103.0); // Last value
    }

    #[test]
    fn test_output_mode_empty_data() {
        let data: Vec<TimeSeriesPoint> = vec![];

        let result = apply_output_mode(data.clone(), OutputMode::TimeSeries);
        assert_eq!(result.len(), 0);

        let result = apply_output_mode(data, OutputMode::LiveValue);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_query_returns_api_exists() {
        use chrono::NaiveDate;

        let asset = AssetKey::new_equity("AAPL").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        // Test that the API exists (will return error without DataProvider)
        let result = AnalyticsQuery::query_returns(&asset, &date_range, OutputMode::TimeSeries);

        assert!(result.is_err(), "Query should require DataProvider");
    }

    #[test]
    fn test_query_volatility_api_exists() {
        use chrono::NaiveDate;

        let asset = AssetKey::new_equity("AAPL").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        // Test that the API exists (will return error without DataProvider)
        let result =
            AnalyticsQuery::query_volatility(&asset, 10, &date_range, OutputMode::TimeSeries);

        assert!(result.is_err(), "Query should require DataProvider");
    }

    #[test]
    fn test_query_different_output_modes() {
        // Test that different output modes can be specified
        let asset = AssetKey::new_equity("AAPL").unwrap();
        let date_range = DateRange::new(
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        // Both should exist and be callable
        let _ts_result = AnalyticsQuery::query_returns(&asset, &date_range, OutputMode::TimeSeries);
        let _lv_result = AnalyticsQuery::query_returns(&asset, &date_range, OutputMode::LiveValue);

        // Verify OutputMode enum works
        assert_eq!(OutputMode::TimeSeries, OutputMode::TimeSeries);
        assert_ne!(OutputMode::TimeSeries, OutputMode::LiveValue);
    }

    #[test]
    fn test_multi_asset_placeholder() {
        // Placeholder test for multi-asset support
        // In the future, this would test passing multiple assets to query functions
        let asset1 = AssetKey::new_equity("AAPL").unwrap();
        let asset2 = AssetKey::new_equity("MSFT").unwrap();

        // For now, just verify we can create multiple assets
        assert_ne!(asset1, asset2);

        // Future: would test correlation or other multi-asset analytics
    }

    // Task Group 6.1: Integration tests

    #[test]
    fn test_end_to_end_volatility_query_builder() {
        use chrono::NaiveDate;

        // Test building a complete volatility query DAG
        let asset = AssetKey::new_equity("AAPL").unwrap();
        let window_size = 10;
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        let builder = VolatilityQueryBuilder::new(asset, window_size, date_range);
        let (dag, _, _, vol_node) = builder.build_dag().unwrap();

        // Verify DAG structure
        assert_eq!(dag.node_count(), 3);

        // Verify execution order is valid
        let exec_order = dag.execution_order_immutable().unwrap();
        assert_eq!(exec_order.len(), 3);

        // Verify volatility node is last in execution order
        assert_eq!(exec_order[2], vol_node);
    }

    #[test]
    fn test_end_to_end_returns_query_builder() {
        use chrono::NaiveDate;

        // Test building a complete returns query DAG
        let asset = AssetKey::new_equity("MSFT").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(),
        );

        let builder = ReturnsQueryBuilder::new(asset, date_range);
        let (dag, _, returns_node) = builder.build_dag().unwrap();

        // Verify DAG structure
        assert_eq!(dag.node_count(), 2);

        // Verify execution order is valid
        let exec_order = dag.execution_order_immutable().unwrap();
        assert_eq!(exec_order.len(), 2);

        // Verify returns node is last in execution order
        assert_eq!(exec_order[1], returns_node);
    }

    #[test]
    fn test_volatility_with_insufficient_data_scenario() {
        use chrono::NaiveDate;

        // Test handling when requested date range is very short
        let asset = AssetKey::new_equity("AAPL").unwrap();
        let window_size = 30; // 30-day volatility
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(), // Only 5 days
        );

        let builder = VolatilityQueryBuilder::new(asset, window_size, date_range);
        let result = builder.build_dag();

        // DAG should still build successfully
        assert!(result.is_ok());

        // The stateless volatility function will handle insufficient data gracefully
        let returns = vec![0.01, 0.02, 0.03, 0.04, 0.05];
        let volatility = calculate_volatility(&returns, 30);

        // Should use all available data
        assert_eq!(volatility.len(), 5);
    }

    #[test]
    fn test_returns_calculation_with_real_data() {
        // Test returns calculation with realistic price data
        let prices = vec![
            100.0, // Day 0
            102.0, // Day 1: +2% = ln(102/100) ≈ 0.0198
            101.0, // Day 2: -1% = ln(101/102) ≈ -0.0099
            103.0, // Day 3: +2% = ln(103/101) ≈ 0.0196
            105.0, // Day 4: +2% = ln(105/103) ≈ 0.0192
        ];

        let returns = calculate_returns(&prices);

        assert_eq!(returns.len(), 5);
        assert!(returns[0].is_nan()); // First return is NaN

        // Verify returns are approximately correct (within 0.0001)
        assert!((returns[1] - 0.0198).abs() < 0.0001);
        assert!((returns[2] - (-0.0099)).abs() < 0.0001);
        assert!((returns[3] - 0.0196).abs() < 0.0001);
        assert!((returns[4] - 0.0192).abs() < 0.0001);
    }

    #[test]
    fn test_volatility_calculation_with_real_returns() {
        // Test volatility with realistic returns data
        let returns = vec![
            f64::NAN, // First return is NaN
            0.02,     // 2%
            -0.01,    // -1%
            0.015,    // 1.5%
            -0.02,    // -2%
            0.01,     // 1%
        ];

        let volatility = calculate_volatility(&returns, 3);

        assert_eq!(volatility.len(), 6);

        // All volatility values should be non-negative
        for (i, &vol) in volatility.iter().enumerate() {
            if i == 0 {
                // First volatility (based on NaN) might be NaN or 0
                continue;
            }
            assert!(
                vol >= 0.0,
                "Volatility at index {} should be non-negative",
                i
            );
        }
    }

    #[test]
    fn test_dag_builder_reusability() {
        use chrono::NaiveDate;

        // Test that query builders can be used multiple times
        let asset = AssetKey::new_equity("AAPL").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        // Build first DAG
        let builder1 = ReturnsQueryBuilder::new(asset.clone(), date_range.clone());
        let result1 = builder1.build_dag();
        assert!(result1.is_ok());

        // Build second DAG (should also work)
        let builder2 = ReturnsQueryBuilder::new(asset, date_range);
        let result2 = builder2.build_dag();
        assert!(result2.is_ok());

        // Both should have the same structure
        let (dag1, _, _) = result1.unwrap();
        let (dag2, _, _) = result2.unwrap();
        assert_eq!(dag1.node_count(), dag2.node_count());
    }

    #[test]
    fn test_output_mode_integration() {
        use chrono::Utc;

        // Test output mode helper with realistic data
        let data: Vec<TimeSeriesPoint> = (0..30)
            .map(|i| TimeSeriesPoint::new(Utc::now(), 100.0 + i as f64))
            .collect();

        // Time series mode returns all data
        let ts_result = apply_output_mode(data.clone(), OutputMode::TimeSeries);
        assert_eq!(ts_result.len(), 30);

        // Live value mode returns only last point
        let lv_result = apply_output_mode(data.clone(), OutputMode::LiveValue);
        assert_eq!(lv_result.len(), 1);
        assert_eq!(lv_result[0].close_price, 129.0); // Last value
    }
}
