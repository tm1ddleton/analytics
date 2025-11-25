# Specification: Returns and Volatility Analytics

## Goal
Implement stateless calculation functions for log returns and rolling volatility that integrate with the DAG computation framework, supporting windowed data processing, automatic burn-in calculation, and both time series and live value outputs for multi-asset analytics queries.

## User Stories
- As a quantitative trader, I want to calculate log returns and volatility for any asset over any date range so that I can analyze price movements and risk
- As a system developer, I want stateless analytics functions that integrate with the DAG framework so that computations can be cached, reused, and executed in parallel
- As a research analyst, I want rolling window calculations with automatic burn-in handling so that I can query analytics without worrying about data requirements

## Specific Requirements

### Returns Calculation
- Calculate log returns using formula: `ln(P_t / P_{t-1})`
- Output as decimal values (e.g., 0.05 for 5% return, not 5.0)
- First value in sequence should be NaN/None (no previous price for comparison)
- Handle NaN values by converting to 0 for now
- Stateless function signature: `fn calculate_returns(prices: &[f64]) -> Vec<f64>`
- Operates on raw f64 price values, not TimeSeriesPoint structs

### Volatility Calculation
- Calculate population standard deviation of returns over N-day rolling window
- Use formula: `σ = sqrt(sum((r_i - μ)²) / N)` where μ is mean of returns in window
- Do NOT annualize (no √252 multiplier)
- Use population standard deviation (divide by N, not N-1)
- Each output point shows volatility of past N days (rolling window)
- Depends on returns as separate node in DAG: `price data → returns → volatility`
- Stateless function signature: `fn calculate_volatility(returns: &[f64], window_size: usize) -> Vec<f64>`

### Stateless Function Design
- Pure functions with no internal state or side effects
- Data passed via function parameters (windowing containers)
- Returns calculation receives 2-point slices (current, previous)
- Volatility calculation receives N-day window of returns
- Functions operate on raw f64 arrays for performance
- Reusable across different assets and time periods

### DAG Integration
- Automatic dependency resolution: query "10-day volatility" creates chain `prices → returns → volatility`
- Node identification via hash of (assets + analytic type + date range)
- Example: "10-day vol for AAPL Jan 1-31" and "10-day vol for MSFT Jan 1-31" are different nodes
- Nodes host the analytic function (not strongly typed node classes)
- Different parameters create different node instances (10-day vs 30-day volatility)
- Integrate with existing `execute()` and `execute_incremental()` methods

### Burn-in Management
- Automatic burn-in calculation based on dependency chain
- 10-day volatility requires 11 days of price data (10 for window + 1 for first return)
- Formula: volatility_window + 1 = price_days_needed
- DataProvider wrapper node should know required date range from dependent analytics
- Gracefully handle insufficient data at start of range
- For now: Simple assumption-based burn-in, future: calculate max from entire DAG

### Query Support and Output Modes
- Support date range queries: "N-day volatility for ASSET from START to END"
- Query parameter determines output mode:
  - **Time series mode**: Return Vec<TimeSeriesPoint> with rolling calculations for each day
  - **Live value mode**: Return single current value (for streaming/real-time use)
- Example query: `query_volatility(asset: AssetKey, window: usize, range: DateRange, mode: QueryMode)`

### Multi-Asset Support
- Support single-asset calculations (returns, volatility)
- Design extensible for multi-asset analytics (future: correlation, covariance)
- Node can operate on multiple asset inputs
- Foundation for cross-asset analytics

### Edge Case Handling
- **Insufficient data**: If only 5 days available for 10-day window, use available data (calculate over 5 days)
- **No data**: Return NaN/None if no data available
- **NaN in data**: Convert NaN to 0 for now (temporary solution)
- **First return**: First return value is NaN/None (no previous price)
- **Window at start**: Gracefully degrade window size at beginning of time series

### Integration with Existing Systems
- Use `TimeSeriesPoint` for input/output at DAG layer (convert to/from f64 internally)
- Use `NodeOutput::Single(Vec<TimeSeriesPoint>)` for time series output
- Use `NodeOutput::Scalar(f64)` for live value output  
- Leverage `DataProvider` trait via wrapper nodes
- Use `AssetKey` for asset identification
- Use `DateRange` for date specifications
- Follow existing error handling patterns with `Result` types

### Testing Strategy
- **Unit tests**: Test stateless functions separately with known inputs/outputs
- **Integration tests**: Test DAG integration with end-to-end queries
- Test cases to cover:
  - Basic returns calculation (known price sequences)
  - Basic volatility calculation (known return sequences)
  - Rolling window behavior
  - Insufficient data handling
  - NaN/None handling
  - Burn-in calculation
  - Multi-asset scenarios
  - DAG dependency resolution

## Visual Design
No visual assets provided.

## Existing Code to Leverage

**DAG Computation Framework**
- `AnalyticsDag` for DAG construction and execution
- `Node` struct for representing analytics computations
- `NodeParams` for parameterization
- `execute()` and `execute_incremental()` methods
- Cycle detection and topological sorting
- Parallel execution with tokio

**Time Series Structures**
- `TimeSeriesPoint` struct with timestamp and close_price
- `DateRange` for specifying date ranges
- `DataProvider` trait for querying data
- `AssetKey` enum for asset identification

**Error Handling**
- `DagError` enum for DAG-related errors
- `DataProviderError` for data access errors
- Result types with clear error messages

## Out of Scope

### Phase 1 (Current Scope)
Only returns and volatility are in scope. The following are explicitly out of scope:

- Moving averages (separate feature)
- Correlation and covariance (separate feature)
- Momentum indicators (separate feature)
- Mean reversion analytics (separate feature)
- Statistical arbitrage signals (separate feature)
- Risk metrics beyond volatility (separate feature)
- Portfolio-level analytics (separate feature)
- Backtesting or strategy evaluation (separate feature)

### Technical Limitations
- Advanced NaN handling strategies (just convert to 0 for now)
- Automatic burn-in propagation through entire DAG (simple assumption for now)
- Multiple output types from single node (single output only)
- Caching of intermediate calculations (DAG handles this, not analytics)
- Performance optimization beyond stateless design (future work)
- Annualized volatility calculations (not in scope, can be added later)
- Sample standard deviation option (only population std dev for now)
- Weighted returns or volatility (equal weighting only)

### Integration Limitations
- Direct database queries from analytics functions (must use DataProvider wrapper)
- Custom windowing strategies (fixed rolling window only)
- Configurable NaN handling policies (zero conversion only for now)
- Real-time streaming execution (foundation exists via execute_incremental, but not fully implemented)
- Historical simulation with different burn-in strategies (future enhancement)


