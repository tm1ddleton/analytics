# Returns and Volatility Analytics - Requirements

## Initial Questions and Answers

### 1. Returns Calculation Type
**Answer:** Log returns `ln(P_t / P_{t-1})`, expressed as decimals (0.05 for 5%). If NaN issue, make it zero.

### 2. Volatility Calculation
**Answer:** Standard deviation of returns over the N-day window. Do NOT annualize. Use population standard deviation (divide by N, not N-1).

### 3. Windowing and Dependencies
**Answer:** Volatility should depend on returns as a separate node. DAG structure: `price data → returns → volatility`

### 4. Edge Cases and Error Handling
**Answer:** 
- If insufficient data (e.g., 5 days for 10-day window): Use available data (average over 5 days)
- If no data: Return NaN/None
- NaN handling: Make it zero for now

### 5. Output Format
**Answer:** Time series for the date range requested, OR the updating live value depending on the request type.

### 6. DAG Node Design
**Answer:**
- Separate nodes for different analytics (returns, volatility)
- Nodes host the analytic function (not "typed" in that sense)
- Different parameters = different node instances
- Node identification: Hash of (parameters + analytic code)

### 7. Data Source Integration
**Answer:** Data should be provided by parent nodes that wrap the DataProvider trait.

## Summary of Key Requirements

### Functional Requirements
1. **Log Returns Calculation**
   - Formula: `ln(P_t / P_{t-1})`
   - Output: Decimal values
   - First value: NaN/None (no previous price)
   - NaN handling: Convert to 0

2. **Volatility Calculation**
   - Formula: Population standard deviation of returns over N-day window
   - Divide by N (not N-1)
   - NOT annualized
   - Depends on returns node

3. **Stateless Functions**
   - Pure functions with no internal state
   - Data passed in via windowing containers
   - Returns: 2-point slice (current, previous)
   - Volatility: N-day window of returns

4. **DAG Integration**
   - Automatic dependency resolution
   - Node structure: DataProvider wrapper → Returns → Volatility
   - Node identification via hash(parameters + analytic code)

5. **Query Support**
   - Date range queries (e.g., "10-day volatility for AAPL from Jan 1 to Jan 31")
   - Returns time series OR live value (depending on request)

### Non-Functional Requirements
- Graceful degradation with insufficient data
- Clean integration with existing DAG framework
- Reusable stateless calculation functions

## Follow-up Questions and Answers

### 1. Request Type Differentiation
**Answer:** Query parameter determines time series vs. live value output.

### 2. Node Identification and Caching
**Answer:** Hash should include: assets + analytic + date range
- Example: "10-day volatility for AAPL Jan 1-31" vs "10-day volatility for MSFT Jan 1-31" are different nodes

### 3. Stateless Function Interface
**Answer:** Raw `f64` values
```rust
fn calculate_returns(prices: &[f64]) -> Vec<f64>
fn calculate_volatility(returns: &[f64], window_size: usize) -> Vec<f64>
```

### 4. DataProvider Wrapper Node
**Answer:** Generic "data fetcher" node is fine. Should know date range from dependent analytics.
- **Burn-in calculation**: Volatility needs burn-in awareness
  - 10-day volatility → needs 10 days of returns → needs 11 days of prices (1 extra for first return)
  - For now: Can be simple and assume, but eventually need to calculate max burn-in requirement

### 5. Window Rolling Behavior
**Answer:** Rolling window - each day shows volatility of the past N days

### 6. Multiple Assets
**Answer:** Yes, support multiple assets (e.g., correlation between AAPL and MSFT returns)

### 7. Testing Strategy
**Answer:** Unit tests for stateless functions separate from DAG integration tests

### 8. Existing Code Integration
**Answer:** Yes, use existing structures:
- `TimeSeriesPoint` for input/output
- `NodeOutput::Single` vs `NodeOutput::Collection`
- `execute()` and `execute_incremental()` methods

## Summary of Additional Requirements

### Functional Requirements (cont'd)
9. **Query Parameters**
   - Query parameter determines output mode (time series vs live value)
   - Standard query format with date range and asset specification

10. **Node Identification**
    - Unique nodes per: (assets + analytic type + date range)
    - Hash-based identification

11. **Burn-in Management**
    - Automatic burn-in calculation based on dependency chain
    - 10-day volatility = 11 days of price data needed
    - Future: Calculate max burn-in from all dependencies

12. **Rolling Window Calculation**
    - Each output point uses rolling N-day window
    - Graceful handling of insufficient data at start of range

13. **Multi-Asset Analytics**
    - Support calculations across multiple assets
    - Foundation for future correlation analytics

### Technical Requirements
- Stateless functions use raw `f64` arrays
- Integration with `TimeSeriesPoint` structs at DAG layer
- Use `NodeOutput` variants appropriately
- Leverage existing `execute()` and `execute_incremental()` methods
- Unit tests for pure functions + integration tests for DAG

## Existing Code to Leverage
- `AnalyticsDag` and DAG framework from previous spec
- `TimeSeriesPoint` struct
- `DataProvider` trait (via wrapper nodes)
- `NodeOutput` enum
- `AssetKey` for asset identification
- `DateRange` for date specifications

## Visual Assets
Status: No visual assets provided

