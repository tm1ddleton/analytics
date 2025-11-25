# Specification: Pull-Mode Analytics Engine

**Date:** 2025-11-25  
**Status:** Approved  
**Roadmap Item:** 8

---

## Goal

Build a pull-mode analytics engine that computes complete historical analytics on-demand for specified date ranges. This provides batch computation capabilities that complement the existing push-mode incremental updates, enabling validation, baseline computation, and on-demand historical analysis.

The pull-mode engine should:
- Reuse existing DAG infrastructure with batch execution
- Return complete time series for any date range
- Support multi-asset parallel computation
- Automatically handle burn-in periods for rolling analytics
- Serve as ground truth for validating push-mode results
- Optimize for long date ranges (years of data)

---

## User Stories

### Story 1: Historical Baseline Computation
**As a** quantitative analyst  
**I want to** compute complete historical volatility for the past year  
**So that I can** establish a baseline before switching to real-time incremental updates

**Acceptance Criteria:**
- Can query any date range (e.g., 2023-01-01 to 2023-12-31)
- Returns complete time series with 252 data points
- Computation completes in seconds for a year of data
- Results match what incremental push-mode would produce

### Story 2: Push-Mode Validation
**As a** developer  
**I want to** validate push-mode results against pull-mode ground truth  
**So that I can** ensure incremental computation is correct

**Acceptance Criteria:**
- Pull-mode computes expected results for date range
- Push-mode computes same range via replay
- Integration test verifies results match within tolerance
- Can run validation on any date range

### Story 3: Multi-Asset Batch Analysis
**As a** portfolio manager  
**I want to** compute returns and volatility for 10 assets over 5 years  
**So that I can** analyze historical portfolio behavior

**Acceptance Criteria:**
- Single query computes all 10 assets in parallel
- Handles 5 years (1,260 trading days) efficiently
- Returns organized results per asset
- Completes in reasonable time (<30 seconds)

### Story 4: On-Demand Analytics
**As a** trader  
**I want to** query volatility for last 90 days when I need it  
**So that I can** make decisions based on recent history without maintaining state

**Acceptance Criteria:**
- Simple API call returns complete 90-day time series
- No need to maintain running state or engine
- Handles missing data gracefully (NaN for gaps)
- Results available immediately

---

## Specific Requirements

### 1. DAG Execution Method
- **Add** new method to `AnalyticsDag`: `execute_pull_mode(node_id, date_range, provider)`
- **Signature:** `pub fn execute_pull_mode(&self, node_id: NodeId, date_range: DateRange, provider: &dyn DataProvider) -> Result<Vec<TimeSeriesPoint>, DagError>`
- **Behavior:** Execute DAG in batch mode for entire date range, return complete time series

### 2. Batch Execution Strategy
- **Reuse** existing DAG structure (same nodes, edges, topology)
- **Different execution:** Process entire date range at once vs incremental updates
- **No state management:** Pull-mode is stateless, no buffers or incremental state
- **Single pass:** Execute each node once for the full range

### 3. Automatic Dependency Resolution
- **When executing node N:** Automatically execute all parent nodes in topological order
- **Example:** Volatility → automatically executes Returns → automatically executes DataProvider
- **Cache intermediate results:** Store parent outputs to avoid re-computation
- **One traversal:** Execute dependency chain once, cache results at each level

### 4. Burn-in Calculation
- **Automatic extension:** Extend date range backward to get burn-in data
- **Example:** 10-day volatility needs 11 days of prices (1 extra for returns calculation)
- **User requests:** 2024-01-01 to 2024-12-31
- **System queries:** 2023-12-21 to 2024-12-31 (11 extra days)
- **Returns:** Vec starting from 2024-01-01 (with NaN for insufficient burn-in if needed)

### 5. Output Format
- **Always return:** `Vec<TimeSeriesPoint>` regardless of range size
- **Single day:** Vec with 1 element
- **Multiple days:** Vec with N elements in chronological order
- **Missing data:** NaN in `close_price` field for gaps or insufficient burn-in
- **Consistent:** Same format as push-mode history output

### 6. Multi-Asset Parallel Execution
- **Support:** Multiple nodes computed in parallel
- **Method:** `execute_pull_mode_parallel(node_ids, date_range, provider)`
- **Returns:** `HashMap<NodeId, Vec<TimeSeriesPoint>>`
- **Parallelism:** Use `tokio` or `rayon` for parallel node execution
- **Independent nodes:** Nodes without dependencies execute concurrently

### 7. Data Loading
- **Reuse:** Same `DataProvider` trait and implementations
- **Node-by-node:** Each DataProvider node queries its data independently
- **Full range:** Load all data for date range (including burn-in) at once
- **No streaming:** Simple load-all approach for daily data (acceptable for years)

### 8. Missing Data Handling
- **Lenient:** Continue computation with partial data
- **NaN propagation:** Missing input data → NaN in outputs
- **No errors:** Don't fail entire computation for missing points
- **Metadata:** Return full Vec with NaN placeholders for gaps

### 9. Performance Optimization
- **Target:** Efficient for long ranges (years of data)
- **Vectorized:** Leverage stateless analytics functions (operate on slices)
- **Batch processing:** Process entire range at once (no per-point overhead)
- **Parallel assets:** Multiple assets computed simultaneously
- **Memory:** Keep full range in memory (fine for daily data, ~10KB per asset-year)

### 10. Integration with Push-Mode
- **Validation:** Pull-mode serves as ground truth for testing push-mode
- **Warm-up:** Pull-mode can compute baseline, then initialize push-mode
- **Shared functions:** Use same analytics calculation functions
- **Comparison API:** Enable easy comparison of pull vs push results

### 11. Error Handling
- **DagError:** Reuse existing error type
- **Variants needed:**
  - `NodeNotFound` - requested node doesn't exist
  - `CyclicDependency` - DAG has cycles (shouldn't happen)
  - `DataProviderError` - data loading failed
  - `ComputationError` - analytics calculation failed
- **Propagation:** Errors bubble up, but partial results preserved where possible

### 12. Caching Strategy
- **Intermediate results:** Cache parent outputs during single execution
- **No persistent cache:** Don't cache results across queries (for now)
- **Session-local:** Cache exists only for duration of one `execute_pull_mode` call
- **Future enhancement:** Add optional result caching by (node_id + date_range) hash

### 13. Testing Strategy
- **Unit tests:** Test individual components (burn-in calc, caching, parallel execution)
- **Integration tests:** Test with real DataProvider and analytics nodes
- **Validation tests:** Compare pull-mode vs push-mode results
- **Performance tests:** Benchmark on long ranges (1+ years, multiple assets)
- **Edge cases:** Empty data, single point, missing data, boundary dates

---

## Existing Code to Leverage

### AnalyticsDag
- **Path:** `src/dag.rs`
- **Usage:** Core DAG structure with nodes, edges, topological sorting
- **Methods:** `add_node()`, `add_edge()`, `get_node()`, `topological_sort()`
- **Integration:** Add new `execute_pull_mode()` method

### Analytics Functions
- **Path:** `src/analytics.rs`
- **Functions:** `calculate_returns(prices)`, `calculate_volatility(prices, window)`
- **Characteristics:** Stateless, operate on slices, vectorized
- **Usage:** Call directly on full data ranges in pull-mode

### DataProvider Trait
- **Path:** `src/time_series.rs`
- **Trait:** `DataProvider` with `get_time_series(asset, date_range)`
- **Implementations:** `SqliteDataProvider`, `InMemoryDataProvider`
- **Usage:** Query data for extended ranges (including burn-in)

### TimeSeriesPoint
- **Path:** `src/time_series.rs`
- **Structure:** `{ timestamp: DateTime<Utc>, close_price: f64 }`
- **Usage:** Standard output format for both pull and push modes

### DateRange
- **Path:** `src/time_series.rs`
- **Structure:** `{ start: NaiveDate, end: NaiveDate }`
- **Usage:** Specify query ranges, calculate burn-in extensions

### NodeParams & NodeOutput
- **Path:** `src/dag.rs`
- **Usage:** Node parameter specification and output types
- **Integration:** Reuse for pull-mode node execution

### Parallel Execution (from Push-Mode)
- **Path:** `src/push_mode.rs` and `src/dag.rs`
- **Pattern:** `tokio` for async parallel execution
- **Usage:** Adapt for multi-asset pull-mode queries

### Error Types
- **Path:** `src/dag.rs`, `src/analytics.rs`
- **Types:** `DagError`, `AnalyticsError`, `DataProviderError`
- **Usage:** Consistent error handling across pull-mode

---

## Out of Scope

### Not Included in This Iteration

1. **Persistent Result Caching**
   - Future: Cache by (node_id + date_range) hash
   - Current: No cross-query caching

2. **Streaming/Incremental Output**
   - Future: Stream results for very long ranges
   - Current: Load all in memory

3. **Query Optimization**
   - Future: Query planning, predicate pushdown
   - Current: Simple execution strategy

4. **Custom Output Formats**
   - Future: CSV, Parquet, DataFrame exports
   - Current: Only `Vec<TimeSeriesPoint>`

5. **Progress Callbacks**
   - Future: Real-time progress updates during long computations
   - Current: Synchronous execution, no progress

6. **Partial Re-computation**
   - Future: Smart re-computation when data changes
   - Current: Always recompute full range

7. **Query Language**
   - Future: SQL-like or DSL for analytics queries
   - Current: Direct method calls

8. **Distributed Execution**
   - Future: Distribute computation across nodes
   - Current: Single-machine parallelism only

9. **Result Persistence**
   - Future: Save computed results to storage
   - Current: In-memory only

10. **Advanced Scheduling**
    - Future: Background batch jobs, scheduling
    - Current: On-demand execution only

---

## Technical Design Notes

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Pull-Mode Execution Flow                  │
│                                                              │
│  1. User calls: dag.execute_pull_mode(node_id, range)      │
│  2. Calculate burn-in: extend range backward                │
│  3. Topological sort: determine execution order             │
│  4. For each node in order:                                 │
│     a. Check cache for parent outputs                       │
│     b. If DataProvider: query extended range                │
│     c. If Analytics: compute on full data slice             │
│     d. Cache result for downstream nodes                    │
│  5. Extract requested range from cached results             │
│  6. Return Vec<TimeSeriesPoint>                             │
└─────────────────────────────────────────────────────────────┘

Example: 10-Day Volatility Query
├─ User requests: 2024-01-01 to 2024-12-31
├─ Burn-in calc: Need 11 days of prices (10 for window + 1 for returns)
├─ Extended range: 2023-12-21 to 2024-12-31
├─ Execute DataProvider: Get prices for extended range
├─ Cache: [273 price points from 2023-12-21 to 2024-12-31]
├─ Execute Returns: Calculate returns from cached prices
├─ Cache: [272 return points from 2023-12-22 to 2024-12-31]
├─ Execute Volatility: Calculate 10-day vol from cached returns
├─ Cache: [262 volatility points from 2024-01-01 to 2024-12-31]
└─ Return: Extract requested range (2024-01-01 to 2024-12-31)
```

### Burn-in Calculation Logic

```rust
fn calculate_burn_in(node_params: &NodeParams, date_range: &DateRange) -> DateRange {
    match node_params {
        NodeParams::Analytics(AnalyticsNodeParams::DataProvider { .. }) => {
            // DataProvider has no burn-in
            date_range.clone()
        }
        NodeParams::Analytics(AnalyticsNodeParams::Returns) => {
            // Returns need 1 extra day for calculation
            DateRange::new(
                date_range.start - chrono::Duration::days(1),
                date_range.end,
            )
        }
        NodeParams::Analytics(AnalyticsNodeParams::Volatility { window_size }) => {
            // Volatility needs window_size days + 1 for returns
            DateRange::new(
                date_range.start - chrono::Duration::days(window_size + 1),
                date_range.end,
            )
        }
        _ => date_range.clone(),
    }
}
```

### Execution Strategy

**Single-Asset Query:**
1. Topological sort to get execution order
2. For each node in order:
   - Execute with cached parent outputs
   - Cache result for downstream nodes
3. Return final node output for requested range

**Multi-Asset Parallel Query:**
1. Group nodes by asset (assuming independent DAG branches)
2. Execute each asset's DAG in parallel using `tokio::spawn`
3. Collect results into HashMap
4. Return all results

### Caching During Execution

```rust
struct ExecutionCache {
    /// Node outputs cached during execution
    outputs: HashMap<NodeId, Vec<TimeSeriesPoint>>,
    /// Extended date ranges per node (including burn-in)
    extended_ranges: HashMap<NodeId, DateRange>,
}

impl ExecutionCache {
    fn get(&self, node_id: NodeId) -> Option<&Vec<TimeSeriesPoint>> {
        self.outputs.get(&node_id)
    }
    
    fn insert(&mut self, node_id: NodeId, output: Vec<TimeSeriesPoint>) {
        self.outputs.insert(node_id, output);
    }
    
    fn extract_range(&self, node_id: NodeId, requested: &DateRange) 
        -> Vec<TimeSeriesPoint> {
        // Filter cached output to requested range
        self.outputs.get(&node_id)
            .unwrap()
            .iter()
            .filter(|p| p.timestamp >= requested.start && p.timestamp <= requested.end)
            .cloned()
            .collect()
    }
}
```

### Pull vs Push Comparison

| Aspect | Pull-Mode | Push-Mode |
|--------|-----------|-----------|
| **Execution** | Batch (entire range) | Incremental (per point) |
| **State** | Stateless | Stateful (buffers, history) |
| **Use Case** | Historical analysis | Real-time updates |
| **Performance** | High throughput | Low latency |
| **Memory** | Load full range | Circular buffers |
| **Output** | Complete time series | Latest values + history |
| **Validation** | Ground truth | Being validated |

### Performance Considerations

**Daily Data Scale:**
- 1 year = 252 trading days
- 5 years = 1,260 trading days
- 10 assets × 5 years = 12,600 data points
- Memory: ~10KB per asset-year (negligible)

**Computation Estimates:**
- Returns: O(n) - single pass
- Volatility: O(n × w) - rolling window
- 1 year, 10-day vol: 252 × 10 = 2,520 operations
- 10 assets parallel: ~2,520 ops each, concurrent

**Target Performance:**
- 1 asset, 1 year: < 100ms
- 10 assets, 5 years (parallel): < 5 seconds
- Bottleneck: Data loading from SQLite, not computation

---

## Implementation Checklist

### Phase 1: Core Pull-Mode Execution
- [ ] Add `execute_pull_mode()` method to `AnalyticsDag`
- [ ] Implement burn-in calculation logic
- [ ] Implement execution cache structure
- [ ] Implement single-node execution with caching
- [ ] Handle NaN propagation for missing data

### Phase 2: Multi-Asset Support
- [ ] Implement `execute_pull_mode_parallel()` method
- [ ] Add parallel execution using `tokio`
- [ ] Handle independent DAG branches
- [ ] Return results as HashMap

### Phase 3: Testing & Validation
- [ ] Unit tests for burn-in calculation
- [ ] Unit tests for execution cache
- [ ] Integration tests with real DataProvider
- [ ] Validation tests vs push-mode
- [ ] Performance benchmarks

### Phase 4: Examples & Documentation
- [ ] Create `examples/pull_mode_query.rs`
- [ ] Create `examples/pull_vs_push_validation.rs`
- [ ] Add documentation to public methods
- [ ] Update README with pull-mode usage

---

**Next Steps:** Create tasks.md with detailed task breakdown.

