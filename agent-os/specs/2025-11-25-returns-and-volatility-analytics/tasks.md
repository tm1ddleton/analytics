# Task Breakdown: Returns and Volatility Analytics

## Overview
Total Tasks: 6 task groups

## Task List

### Core Analytics Functions

#### Task Group 1: Returns Calculation Implementation
**Dependencies:** None

- [x] 1.0 Complete returns calculation implementation
  - [x] 1.1 Write 2-8 focused tests for returns calculation
    - Test log returns with known price sequences
    - Test first value is NaN/None (no previous price)
    - Test NaN handling (convert to 0)
    - Test returns on flat prices (ln(1) = 0)
    - Test returns on increasing/decreasing sequences
  - [x] 1.2 Implement stateless returns calculation function
    - Create `fn calculate_returns(prices: &[f64]) -> Vec<f64>`
    - Use log returns formula: ln(P_t / P_{t-1})
    - Return decimal values (0.05 for 5%, not 5.0)
    - First value should be NaN (no previous price)
    - Handle NaN values by converting to 0
  - [x] 1.3 Add returns calculation module
    - Create `src/analytics.rs` module
    - Export returns calculation function
    - Add documentation with examples
    - Include mathematical formula in doc comments
  - [x] 1.4 Ensure returns calculation tests pass
    - Run ONLY the 2-8 tests written in 1.1
    - Verify correct log returns calculation
    - Verify NaN handling works
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 1.1 pass
- Returns calculation produces correct log returns
- First value is NaN, subsequent values are ln(P_t/P_{t-1})
- NaN values are converted to 0

#### Task Group 2: Volatility Calculation Implementation
**Dependencies:** Task Group 1

- [x] 2.0 Complete volatility calculation implementation
  - [x] 2.1 Write 2-8 focused tests for volatility calculation
    - Test volatility with known return sequences
    - Test rolling window behavior (each point uses past N days)
    - Test population standard deviation (divide by N)
    - Test insufficient data handling (use available data)
    - Test edge case: window size larger than data
  - [x] 2.2 Implement stateless volatility calculation function
    - Create `fn calculate_volatility(returns: &[f64], window_size: usize) -> Vec<f64>`
    - Use population standard deviation formula: sqrt(sum((r_i - μ)²) / N)
    - Do NOT annualize (no √252 multiplier)
    - Implement rolling window (each point = past N days)
    - Handle insufficient data gracefully
  - [x] 2.3 Add rolling window helper function
    - Create helper for extracting rolling windows from data
    - Handle edge cases at start of sequence
    - Return available data when window exceeds data length
  - [x] 2.4 Ensure volatility calculation tests pass
    - Run ONLY the 2-8 tests written in 2.1
    - Verify correct standard deviation calculation
    - Verify rolling window behavior
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 2.1 pass
- Volatility uses population std dev (÷N, not N-1)
- Rolling window produces correct values
- Graceful handling of insufficient data

### DAG Integration

#### Task Group 3: DAG Node Wrappers for Analytics
**Dependencies:** Task Groups 1, 2

- [x] 3.0 Complete DAG node wrappers implementation
  - [x] 3.1 Write 2-8 focused tests for node wrappers
    - Test DataProvider wrapper node fetches price data
    - Test Returns node converts TimeSeriesPoint to f64, calculates, converts back
    - Test Volatility node receives returns and calculates volatility
    - Test node identification hashing (assets + analytic + date range)
  - [x] 3.2 Implement DataProvider wrapper node
    - Create node that wraps DataProvider trait
    - Fetch price data for specified asset and date range
    - Convert to Vec<TimeSeriesPoint> output
    - Handle burn-in requirements from dependent nodes
  - [x] 3.3 Implement Returns analytic node
    - Create node that converts TimeSeriesPoint to prices (f64)
    - Call calculate_returns() function
    - Convert results back to TimeSeriesPoint
    - Output as NodeOutput::Single(Vec<TimeSeriesPoint>)
  - [x] 3.4 Implement Volatility analytic node
    - Create node that receives returns from parent node
    - Extract f64 values from TimeSeriesPoint
    - Call calculate_volatility() function
    - Convert results back to TimeSeriesPoint
    - Output as NodeOutput::Single(Vec<TimeSeriesPoint>)
  - [x] 3.5 Implement node identification hashing
    - Create hash function for (assets + analytic + date range)
    - Use for node deduplication in DAG
    - Ensure consistent hashing
  - [x] 3.6 Ensure node wrapper tests pass
    - Run ONLY the 2-8 tests written in 3.1
    - Verify data flows correctly through nodes
    - Verify type conversions work (TimeSeriesPoint ↔ f64)
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 3.1 pass
- DataProvider wrapper node fetches data correctly
- Returns and Volatility nodes integrate with stateless functions
- Node identification via hash works

#### Task Group 4: Automatic Dependency Resolution
**Dependencies:** Task Group 3

- [x] 4.0 Complete automatic dependency resolution
  - [x] 4.1 Write 2-8 focused tests for dependency resolution
    - Test query "10-day volatility" creates chain: prices → returns → volatility
    - Test burn-in calculation (10-day vol needs 11 days of prices)
    - Test node reuse (same query twice uses same nodes)
    - Test different queries create different nodes
  - [x] 4.2 Implement query builder for volatility
    - Create function that builds DAG from high-level query
    - Example: `query_volatility(asset, window, date_range) -> DAG`
    - Automatically create DataProvider → Returns → Volatility chain
    - Handle node identification and deduplication
  - [x] 4.3 Implement burn-in calculation
    - Calculate required burn-in from parameters
    - Formula: volatility_window + 1 = price_days_needed
    - Adjust DataProvider date range accordingly
    - Document burn-in logic clearly
  - [x] 4.4 Implement query builder for returns
    - Create function that builds DAG for returns query
    - Example: `query_returns(asset, date_range) -> DAG`
    - Create DataProvider → Returns chain
    - Handle burn-in (need 1 extra day for first return)
  - [x] 4.5 Ensure dependency resolution tests pass
    - Run ONLY the 2-8 tests written in 4.1
    - Verify correct DAG structure created
    - Verify burn-in calculation correct
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 4.1 pass
- High-level query creates correct DAG structure
- Burn-in calculation works (N-day vol needs N+1 days of prices)
- Node deduplication works correctly

### Query Interface and Output Modes

#### Task Group 5: Query Interface and Output Modes
**Dependencies:** Task Group 4

- [x] 5.0 Complete query interface and output modes
  - [x] 5.1 Write 2-8 focused tests for query interface
    - Test time series mode returns Vec<TimeSeriesPoint>
    - Test live value mode returns single Scalar
    - Test query with different date ranges
    - Test multi-asset query support
  - [x] 5.2 Define QueryMode enum
    - Create enum with TimeSeries and LiveValue variants
    - Document behavior of each mode
  - [x] 5.3 Implement query execution with mode selection
    - Add mode parameter to query functions
    - TimeSeries mode: Return NodeOutput::Single(Vec<TimeSeriesPoint>)
    - LiveValue mode: Return NodeOutput::Scalar(latest_value)
    - Execute DAG and extract appropriate output
  - [x] 5.4 Implement end-to-end query API
    - Create user-facing API for queries
    - Example: `calculate_volatility(asset, window, range, provider, mode)`
    - Build DAG, execute, return results
    - Handle errors gracefully
  - [x] 5.5 Add multi-asset query support
    - Support queries with multiple assets
    - Foundation for future correlation analytics
    - Return appropriate output structure
  - [x] 5.6 Ensure query interface tests pass
    - Run ONLY the 2-8 tests written in 5.1
    - Verify mode selection works
    - Verify end-to-end queries work
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 5.1 pass
- QueryMode enum implemented and used
- Time series and live value modes work correctly
- End-to-end API functional

### Integration and Testing

#### Task Group 6: Integration Testing and Documentation
**Dependencies:** Task Groups 1-5

- [x] 6.0 Complete integration testing and documentation
  - [x] 6.1 Write 2-8 integration tests
    - Test end-to-end: Query 10-day volatility for AAPL over 30 days
    - Test end-to-end: Query returns for MSFT over 60 days
    - Test with real DataProvider (InMemoryDataProvider)
    - Test with insufficient data scenarios
    - Test DAG caching and reuse
  - [x] 6.2 Add example queries to documentation
    - Document common query patterns
    - Provide code examples for returns and volatility
    - Show both TimeSeries and LiveValue modes
    - Document burn-in requirements
  - [x] 6.3 Add mathematical documentation
    - Document log returns formula with examples
    - Document volatility formula with examples
    - Explain rolling window behavior
    - Add references to financial formulas
  - [x] 6.4 Verify all tests pass
    - Run complete test suite for analytics module
    - Run integration tests
    - Verify no regressions in existing code
  - [x] 6.5 Performance validation
    - Test with realistic data volumes (1 year daily data)
    - Ensure acceptable performance
    - Document any performance considerations

**Acceptance Criteria:**
- All 2-8 integration tests pass
- Complete test suite passes (unit + integration)
- Documentation includes examples and formulas
- Performance is acceptable for typical use cases

## Execution Order

Recommended implementation sequence:
1. Returns Calculation (Task Group 1) - Foundation for all analytics
2. Volatility Calculation (Task Group 2) - Depends on returns
3. DAG Node Wrappers (Task Group 3) - Integrate with DAG framework
4. Automatic Dependency Resolution (Task Group 4) - Smart query building
5. Query Interface and Output Modes (Task Group 5) - User-facing API
6. Integration Testing and Documentation (Task Group 6) - Validation and docs

## Notes

- Follow test-driven development: Write tests first, then implementation
- Keep functions stateless and pure for testability
- Leverage existing DAG framework - don't reinvent
- Use TimeSeriesPoint for DAG layer, f64 for calculations
- Document formulas and edge cases clearly
- Each task group should be completable independently after dependencies are met


