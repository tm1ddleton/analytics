# Requirements Gathering: Pull-Mode Analytics Engine

## Clarifying Questions & Answers

### 1. Query Interface
**Q:** How should users request pull-mode analytics?  
**A:** Option D - Reuse DAG with batch execution mode

**Decision Rationale:** Leverage existing DAG infrastructure. Same DAG structure, different execution strategy (batch compute entire range vs incremental updates).

**Example:**
```rust
// Use same DAG structure as push-mode
let mut dag = AnalyticsDag::new();
let vol_node = dag.add_node(..., Volatility params);
// ... add edges ...

// Execute in batch/pull mode for entire date range
let results = dag.execute_pull_mode(vol_node, date_range)?;
// Returns: Vec<TimeSeriesPoint> with complete time series
```

### 2. Execution Model
**Q:** How should pull-mode computation work?  
**A:** Option B - Reuse existing DAG infrastructure, execute in "batch mode"

**Decision Rationale:** Use same DAG structure, switch execution mode from incremental to batch. DAG persists and can be reused, more efficient for repeated queries.

### 3. Output Format
**Q:** What should pull-mode return?  
**A:** Option A - `Vec<TimeSeriesPoint>` (complete time series)

**Note:** The last point is just a single date query (query for one day returns Vec with one point)

**Decision Rationale:** Consistent format regardless of range size. Single-day query returns Vec with one element.

### 4. Caching Strategy
**Q:** Should pull-mode cache results?  
**A:** Not bothered about caching for now

**Decision Rationale:** Keep it simple for initial implementation. Can add caching later if needed.

### 5. Relationship to Push-Mode
**Q:** How should pull-mode relate to the existing push-mode engine?  
**A:** Option C - Pull-mode can "warm up" push-mode (compute historical, then switch to incremental)

**Decision Rationale:** Pull-mode computes historical baseline, then push-mode takes over for incremental updates.

**Use Case:**
```rust
// 1. Use pull-mode to compute historical baseline
let historical_results = dag.execute_pull_mode(vol_node, historical_range)?;

// 2. Initialize push-mode with those results
engine.initialize_from_pull_mode(historical_results)?;

// 3. Switch to incremental updates
engine.push_data(asset, new_timestamp, new_value)?;
```

### 6. Multi-Asset Queries
**Q:** Should pull-mode support querying multiple assets at once?  
**A:** Option C - Multiple assets, computed in parallel

**Decision Rationale:** Maximize performance for multi-asset queries using parallel execution.

### 7. Partial Computation
**Q:** What if data is missing for part of the requested date range?  
**A:** Option A - Return partial results (NaN for missing data)

**Decision Rationale:** Lenient behavior. User gets what's available with NaN placeholders for gaps.

### 8. Performance Optimization
**Q:** What performance characteristics are important?  
**A:** Option B - Efficient for long ranges (years)

**Decision Rationale:** Optimize for computing large historical datasets. Pull-mode is typically used for batch computation of extended periods.

### 9. Comparison Use Case
**Q:** What comparison scenarios do you want to show?  
**A:** Just call the pull API with the same data and get the same results

**Use Case:** Integration test / validation
- Pull-mode: Compute complete time series for date range
- Push-mode: Incrementally compute same range via replay
- Compare: Results should match

**Decision Rationale:** Pull-mode serves as ground truth for validating push-mode correctness.

### 10. Integration with Replay
**Q:** Should pull-mode integrate with the replay system?  
**A:** Option B - Yes, use pull-mode to validate push-mode results

**Decision Rationale:** Pull-mode computes "expected" results, push-mode (via replay) computes "actual" results, compare for correctness.

**Example Integration Test:**
```rust
// 1. Pull-mode: compute ground truth
let expected = dag.execute_pull_mode(vol_node, date_range)?;

// 2. Push-mode: compute via replay
let mut engine = PushModeEngine::new(dag.clone());
replay.run(assets, date_range, |a, ts, v| {
    engine.push_data(a, ts, v)
})?;
let actual = engine.get_history(vol_node)?;

// 3. Compare results
assert_eq!(expected, actual);
```

---

## Follow-Up Questions & Answers

### Follow-Up 1: DAG Execution Method
**Q:** Should pull-mode execution be a new method on AnalyticsDag, a wrapper, or extension of existing method?  
**A:** Option A - A new method on `AnalyticsDag` (e.g., `dag.execute_pull_mode(node_id, date_range)`)

**Decision Rationale:** Keep it simple for now. Direct method on DAG is most straightforward.

### Follow-Up 2: Data Loading Strategy
**Q:** How should data be loaded for pull-mode?  
**A:** Same as push-mode (Option D) - node-by-node via DataProvider

**Decision Rationale:** Reuse existing patterns for now. Leverage DataProvider infrastructure that's already working.

### Follow-Up 3: Burn-in Handling
**Q:** For rolling analytics (e.g., 10-day volatility), how should burn-in be handled?  
**A:** Option D - Automatically extend date range backward to get burn-in data + return NaN for first available points

**Decision Rationale:** User convenience (automatic burn-in) + graceful degradation (NaN for insufficient data).

**Example:**
```rust
// User requests: 2024-01-01 to 2024-12-31 for 10-day volatility
// System automatically queries: 2023-12-21 to 2024-12-31 (extra 11 days for burn-in)
// Returns: Vec starting from 2024-01-01 with NaN for first few days if needed
```

### Follow-Up 4: Node Dependencies
**Q:** When executing a node (e.g., Volatility), should pull-mode automatically execute parents?  
**A:** Option C - Cache intermediate results during traversal

**Decision Rationale:** Automatic execution + efficiency. When computing Volatility:
1. Execute DataProvider node → cache price data
2. Execute Returns node using cached prices → cache returns
3. Execute Volatility node using cached returns → final result

This avoids re-computation and maximizes efficiency.

---

## Additional Context

### Data Characteristics
- **Source:** Same as push-mode (SqliteDataProvider, InMemoryDataProvider)
- **Frequency:** Daily data from Yahoo Finance
- **Scale:** Optimize for years of data (252+ trading days)
- **Assets:** Multiple assets computed in parallel

### Key Design Implications

**Batch Computation:**
- Process entire date range at once
- No incremental state management
- Optimized for throughput over latency

**Validation Use Case:**
- Pull-mode = expected (ground truth)
- Push-mode = actual (incremental)
- Integration tests verify correctness

**DAG Reuse:**
- Same DAG structure as push-mode
- Different execution strategy
- Leverage existing analytics functions

**Output Consistency:**
- Always returns `Vec<TimeSeriesPoint>`
- Single-day query returns Vec with 1 element
- Missing data represented as NaN

---

## Existing Code to Reference

### From DAG Framework
- ✅ `AnalyticsDag` - existing DAG structure
- ✅ `Node`, `NodeParams` - node definitions
- ✅ Topological sorting - execution order
- ✅ Parallel execution infrastructure

### From Analytics Library
- ✅ `calculate_returns()` - stateless function
- ✅ `calculate_volatility()` - stateless function
- ✅ Analytics work with raw data slices

### From Push-Mode Engine
- ✅ `PushModeEngine` - for comparison/validation
- ✅ Node execution logic
- ✅ State management patterns

### From Data Providers
- ✅ `DataProvider` trait - query interface
- ✅ `SqliteDataProvider` - historical data access
- ✅ `InMemoryDataProvider` - testing

---

## Out of Scope (For This Iteration)

1. Caching of results or intermediate computations
2. Streaming results for memory efficiency
3. Query language or SQL interface
4. Custom output formats (CSV, Parquet, etc.)
5. Distributed computation across nodes
6. Real-time progress callbacks during computation
7. Query optimization or planning
8. Partial re-computation when data changes
9. Result persistence to storage
10. UI-specific output formatting

---

## Visual Assets
None provided.

---

## Example API Usage

### Basic Pull-Mode Query
```rust
// Set up DAG (same as push-mode)
let mut dag = AnalyticsDag::new();
let vol_node = setup_volatility_dag(&mut dag, aapl.clone())?;

// Execute in pull mode for date range
let results = dag.execute_pull_mode(
    vol_node,
    DateRange::new(
        NaiveDate::from_ymd(2024, 1, 1),
        NaiveDate::from_ymd(2024, 12, 31),
    )
)?;

// Results: Vec<TimeSeriesPoint> with 252 days of volatility
for point in results {
    println!("{}: {}", point.timestamp, point.close_price);
}
```

### Multi-Asset Parallel Query
```rust
let assets = vec![aapl, msft, goog];
let vol_nodes = setup_multi_asset_dag(&mut dag, assets)?;

// Compute all assets in parallel
let results = dag.execute_pull_mode_parallel(vol_nodes, date_range)?;

// Results: HashMap<NodeId, Vec<TimeSeriesPoint>>
for (node_id, time_series) in results {
    println!("Asset {}: {} points", node_id, time_series.len());
}
```

### Validation / Integration Test
```rust
// Ground truth from pull-mode
let expected = dag.execute_pull_mode(vol_node, date_range)?;

// Incremental computation via push-mode + replay
let mut engine = PushModeEngine::new(dag.clone());
replay.run(assets, date_range, |a, ts, v| {
    engine.push_data(a, ts, v)?;
    Ok(())
})?;
let actual = engine.get_history(vol_node)?;

// Validate results match
assert_eq!(expected.len(), actual.len());
for (exp, act) in expected.iter().zip(actual.iter()) {
    assert!((exp.close_price - act.close_price).abs() < 1e-10);
}
```

---

**Date:** 2025-11-25  
**Status:** Requirements Complete

