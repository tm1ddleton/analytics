# Tasks: Pull-Mode Analytics Engine

**Spec:** `agent-os/specs/2025-11-25-pull-mode-analytics-engine/spec.md`  
**Date:** 2025-11-25  
**Status:** Ready for Implementation

---

## Task Groups Overview

- **Task Group 1:** Core Pull-Mode Execution (Foundation)
- **Task Group 2:** Burn-in Calculation (Auto-extension)
- **Task Group 3:** Execution Cache System (Performance)
- **Task Group 4:** Multi-Asset Parallel Execution (Scalability)
- **Task Group 5:** Integration with Existing Systems (Compatibility)
- **Task Group 6:** Validation & Testing (Quality)
- **Task Group 7:** Examples and Documentation (Usability)

**Estimated Total Complexity:** Large (7 task groups, ~35 sub-tasks)

---

## Task Group 1: Core Pull-Mode Execution

**Goal:** Implement basic pull-mode execution method on AnalyticsDag

**Dependencies:** None (can start immediately, leverages existing DAG infrastructure)

**Acceptance Criteria:**
- `execute_pull_mode()` method exists on `AnalyticsDag`
- Can execute single node for date range
- Returns `Vec<TimeSeriesPoint>` with complete time series
- Basic tests pass for simple cases

---

### Tasks

#### Task 1.1: Add execute_pull_mode method signature
- [x] Add `execute_pull_mode()` to `AnalyticsDag` in `src/dag.rs`
- [x] Signature: `pub fn execute_pull_mode(&self, node_id: NodeId, date_range: DateRange, provider: &dyn DataProvider) -> Result<Vec<TimeSeriesPoint>, DagError>`
- [x] Add documentation explaining batch execution vs incremental
- [x] Add example usage in docstring

**Estimated Time:** 20 minutes

---

#### Task 1.2: Implement basic execution logic
- [x] Get node from DAG by `node_id`
- [x] Return error if node not found
- [x] Implement simple single-node execution (no dependencies yet)
- [x] Handle DataProvider node: query data for date range
- [x] Return results as `Vec<TimeSeriesPoint>`

**Estimated Time:** 30 minutes

---

#### Task 1.3: Add topological dependency resolution
- [x] Get all ancestors of target node using existing topological sort
- [x] Build execution order (parents before children)
- [x] Execute nodes in order
- [x] Pass parent outputs to child nodes
- [x] Handle missing dependencies with clear errors

**Estimated Time:** 45 minutes

---

#### Task 1.4: Write basic tests
- [x] Test single DataProvider node execution
- [x] Test Returns node (DataProvider → Returns)
- [x] Test Volatility node (DataProvider → Returns → Volatility)
- [x] Test node not found error
- [x] Test with `InMemoryDataProvider` for determinism

**Test Count:** 5 tests

**Estimated Time:** 40 minutes

---

## Task Group 2: Burn-in Calculation

**Goal:** Automatically extend date ranges to include burn-in periods for rolling analytics

**Dependencies:** Task Group 1

**Acceptance Criteria:**
- Burn-in automatically calculated based on node type
- Date range extended backward for data queries
- Results trimmed to requested range before return
- Tests verify correct burn-in for different window sizes

---

### Tasks

#### Task 2.1: Implement burn-in calculation function
- [x] Create `calculate_burn_in()` helper function
- [x] For DataProvider nodes: no burn-in (return original range)
- [x] For Returns nodes: extend 1 day backward
- [x] For Volatility nodes: extend `window_size + 1` days backward
- [x] Handle other node types appropriately

**Estimated Time:** 30 minutes

---

#### Task 2.2: Integrate burn-in into execution
- [x] Calculate extended range for each node based on its params
- [x] Query DataProvider with extended range
- [x] Execute analytics with full extended data
- [x] Trim final results to user-requested range
- [x] Store extended ranges for debugging

**Estimated Time:** 35 minutes

---

#### Task 2.3: Write burn-in tests
- [x] Test Returns: verify 1 extra day queried
- [x] Test 5-day Volatility: verify 6 extra days queried
- [x] Test 10-day Volatility: verify 11 extra days queried
- [x] Test result trimming to requested range
- [x] Test with boundary dates (start of available data)

**Test Count:** 5 tests

**Estimated Time:** 35 minutes

---

## Task Group 3: Execution Cache System

**Goal:** Cache intermediate node results to avoid re-computation during single execution

**Dependencies:** Task Groups 1, 2

**Acceptance Criteria:**
- Intermediate results cached during traversal
- Parent outputs reused by multiple children
- Cache cleared after execution completes
- Tests verify caching reduces redundant computation

---

### Tasks

#### Task 3.1: Create ExecutionCache structure
- [x] Define `ExecutionCache` struct
- [x] Field: `outputs: HashMap<NodeId, Vec<TimeSeriesPoint>>`
- [x] Field: `extended_ranges: HashMap<NodeId, DateRange>`
- [x] Implement `get()`, `insert()`, `clear()` methods
- [x] Implement `extract_range()` to filter cached output to requested range

**Estimated Time:** 25 minutes

---

#### Task 3.2: Integrate cache into execution flow
- [x] Create cache at start of `execute_pull_mode()`
- [x] Before executing node, check cache for existing result
- [x] After executing node, insert result into cache
- [x] Pass cached parent outputs to child nodes
- [x] Extract and return results for requested range from cache

**Estimated Time:** 40 minutes

---

#### Task 3.3: Handle multiple parent outputs
- [x] When node has multiple parents, retrieve all from cache
- [x] Combine parent outputs as needed for node execution
- [x] Handle case where parents have different date ranges
- [x] Align parent outputs before passing to child

**Estimated Time:** 30 minutes

---

#### Task 3.4: Write caching tests
- [x] Test cache stores and retrieves results correctly
- [x] Test parent result reused by multiple children
- [x] Test extract_range filters to requested range
- [x] Test cache cleared after execution
- [x] Test with complex DAG (diamond dependency)

**Test Count:** 5 tests

**Estimated Time:** 40 minutes

---

## Task Group 4: Multi-Asset Parallel Execution

**Goal:** Support querying multiple assets/nodes in parallel for performance

**Dependencies:** Task Groups 1, 2, 3

**Acceptance Criteria:**
- `execute_pull_mode_parallel()` method exists
- Multiple nodes executed concurrently
- Returns `HashMap<NodeId, Vec<TimeSeriesPoint>>`
- Tests verify parallel execution and correctness

---

### Tasks

#### Task 4.1: Add parallel execution method
- [x] Add `execute_pull_mode_parallel()` to `AnalyticsDag`
- [x] Signature: `pub async fn execute_pull_mode_parallel(&self, node_ids: Vec<NodeId>, date_range: DateRange, provider: Arc<dyn DataProvider>) -> Result<HashMap<NodeId, Vec<TimeSeriesPoint>>, DagError>`
- [x] Document parallel execution behavior
- [x] Add usage example

**Estimated Time:** 20 minutes

---

#### Task 4.2: Implement parallel execution logic
- [x] Identify independent node branches (no shared dependencies)
- [x] Use `tokio::spawn` to execute branches in parallel
- [x] Collect results from all spawned tasks
- [x] Combine into `HashMap<NodeId, Vec<TimeSeriesPoint>>`
- [x] Handle errors from any parallel task

**Estimated Time:** 50 minutes

---

#### Task 4.3: Handle shared dependencies
- [x] Detect nodes with shared ancestors
- [x] Execute shared ancestors first (sequentially)
- [x] Cache shared results
- [x] Execute independent branches in parallel using cached ancestors
- [x] Ensure thread-safe cache access

**Estimated Time:** 45 minutes

---

#### Task 4.4: Write parallel execution tests
- [x] Test 2 independent nodes execute in parallel
- [x] Test 3+ assets with same analytics (parallel)
- [x] Test nodes with shared dependencies
- [x] Test error in one branch doesn't crash others
- [x] Test performance: parallel faster than sequential

**Test Count:** 5 tests

**Estimated Time:** 50 minutes

---

## Task Group 5: Integration with Existing Systems

**Goal:** Ensure pull-mode works seamlessly with existing DAG, analytics, and data providers

**Dependencies:** Task Groups 1-4

**Acceptance Criteria:**
- Works with existing `DataProvider` implementations
- Works with existing analytics nodes (Returns, Volatility)
- Compatible with existing DAG structure
- Integration tests pass

---

### Tasks

#### Task 5.1: Test with SqliteDataProvider
- [x] Create integration test with `SqliteDataProvider`
- [x] Set up test database with sample data
- [x] Execute pull-mode query for 1 year of data
- [x] Verify results match expected values
- [x] Clean up test database

**Estimated Time:** 40 minutes

---

#### Task 5.2: Test with InMemoryDataProvider
- [x] Create tests using `InMemoryDataProvider`
- [x] Test various date ranges (1 day, 1 month, 1 year)
- [x] Test with multiple assets
- [x] Test with missing data (gaps)
- [x] Verify NaN handling

**Estimated Time:** 35 minutes

---

#### Task 5.3: Test with existing analytics nodes
- [x] Test Returns calculation matches existing function
- [x] Test Volatility calculation matches existing function
- [x] Test with various window sizes (5, 10, 20 days)
- [x] Verify output format matches push-mode
- [x] Test edge cases (single point, all NaN)

**Estimated Time:** 40 minutes

---

#### Task 5.4: Test DAG compatibility
- [x] Build complex DAG with multiple analytics
- [x] Test pull-mode execution on various nodes
- [x] Verify topological order respected
- [x] Test with DAG modifications (add/remove nodes)
- [x] Ensure no interference with existing DAG methods

**Estimated Time:** 35 minutes

---

## Task Group 6: Validation & Testing

**Goal:** Validate pull-mode results against push-mode and create comprehensive test suite

**Dependencies:** Task Groups 1-5

**Acceptance Criteria:**
- Pull-mode results match push-mode results (integration test)
- Comprehensive unit and integration tests
- Performance benchmarks meet targets
- Edge cases handled correctly

---

### Tasks

#### Task 6.1: Create pull vs push validation test
- [x] Set up identical DAG for pull and push modes
- [x] Use same data and date range
- [x] Execute pull-mode to get ground truth
- [x] Execute push-mode via replay
- [x] Compare results within tolerance (< 1e-10)
- [x] Test with multiple assets and analytics

**Estimated Time:** 50 minutes

---

#### Task 6.2: Write edge case tests
- [x] Test with empty date range
- [x] Test with single data point
- [x] Test with date range before data availability
- [x] Test with date range after data availability
- [x] Test with all NaN data
- [x] Test with very long range (10+ years)

**Test Count:** 6 tests

**Estimated Time:** 45 minutes

---

#### Task 6.3: Create performance benchmarks
- [x] Benchmark 1 asset, 1 year (target: < 100ms)
- [x] Benchmark 10 assets, 5 years parallel (target: < 5s)
- [x] Benchmark complex DAG (multiple analytics)
- [x] Compare pull vs push performance characteristics
- [x] Document performance results

**Estimated Time:** 40 minutes

---

#### Task 6.4: Test error handling
- [x] Test invalid node ID
- [x] Test cyclic DAG (should be impossible but verify)
- [x] Test DataProvider failures
- [x] Test analytics computation failures
- [x] Test partial failures in parallel execution
- [x] Verify error messages are clear

**Test Count:** 6 tests

**Estimated Time:** 40 minutes

---

## Task Group 7: Examples and Documentation

**Goal:** Create example programs and documentation for pull-mode usage

**Dependencies:** Task Groups 1-6 (all implementation complete)

**Acceptance Criteria:**
- Example programs compile and run correctly
- Documentation covers common use cases
- API documentation complete
- Integration with replay demonstrated

---

### Tasks

#### Task 7.1: Create examples/pull_mode_query.rs
- [x] Create example program that:
  - Sets up sample data for AAPL
  - Builds DAG with Returns → Volatility
  - Queries 1 year of data using pull-mode
  - Displays results (first 10, last 10 points)
  - Shows execution time
- [x] Add clear console output
- [x] Add comments explaining each step
- [x] Test: `cargo run --example pull_mode_query`

**Estimated Time:** 40 minutes

---

#### Task 7.2: Create examples/pull_mode_multi_asset.rs
- [x] Create example program that:
  - Sets up data for AAPL, MSFT, GOOG
  - Builds multi-asset DAG
  - Queries all assets in parallel using pull-mode
  - Displays results per asset
  - Shows performance comparison vs sequential
- [x] Demonstrate parallel execution benefits
- [x] Test: `cargo run --example pull_mode_multi_asset`

**Estimated Time:** 45 minutes

---

#### Task 7.3: Create examples/pull_vs_push_validation.rs
- [x] Create example program that:
  - Sets up same DAG for both modes
  - Computes using pull-mode (ground truth)
  - Computes using push-mode via replay
  - Compares results point-by-point
  - Reports match/mismatch statistics
- [x] Demonstrate validation use case
- [x] Test: `cargo run --example pull_vs_push_validation`

**Estimated Time:** 50 minutes

---

#### Task 7.4: Update documentation
- [x] Add doc comments to `execute_pull_mode()`
  - Include usage examples
  - Document error conditions
  - Explain burn-in behavior
  - Show performance characteristics
- [x] Add doc comments to `execute_pull_mode_parallel()`
- [x] Update main README.md with pull-mode section
- [x] Add comparison table (pull vs push modes)
- [x] Link to example programs

**Estimated Time:** 45 minutes

---

#### Task 7.5: Create integration guide
- [x] Document how to use pull-mode for validation
- [x] Document how to warm up push-mode with pull results
- [x] Document best practices (when to use pull vs push)
- [x] Add troubleshooting section
- [x] Include performance tuning tips

**Estimated Time:** 30 minutes

---

## Summary

### Task Group Completion Order

```
Task Group 1 (Core Execution)
     ↓
Task Group 2 (Burn-in)
     ↓
Task Group 3 (Caching)
     ↓
Task Group 4 (Parallel Execution)
     ↓
Task Group 5 (Integration)
     ↓
Task Group 6 (Validation & Testing)
     ↓
Task Group 7 (Examples & Docs)
```

### Estimated Total Time

| Task Group | Estimated Time |
|------------|----------------|
| TG1: Core Execution | 2.5 hours |
| TG2: Burn-in | 1.75 hours |
| TG3: Caching | 2.25 hours |
| TG4: Parallel Execution | 2.75 hours |
| TG5: Integration | 2.5 hours |
| TG6: Validation | 2.75 hours |
| TG7: Examples & Docs | 3.5 hours |
| **Total** | **~18 hours** |

### Key Milestones

- ✅ **Milestone 1** (TG1): Basic pull-mode execution works
- ✅ **Milestone 2** (TG2): Automatic burn-in calculation
- ✅ **Milestone 3** (TG3): Performance optimized with caching
- ✅ **Milestone 4** (TG4): Multi-asset parallel queries
- ✅ **Milestone 5** (TG5): Full integration with existing systems
- ✅ **Milestone 6** (TG6): Validated against push-mode
- ✅ **Milestone 7** (TG7): Production-ready with examples

### Testing Strategy

- **Unit Tests:** 30+ tests covering individual components
- **Integration Tests:** 10+ tests with real providers and complex DAGs
- **Validation Tests:** Pull vs push comparison tests
- **Performance Tests:** Benchmarks for various scales
- **Example Programs:** 3 runnable examples demonstrating use cases

### Key Features to Implement

1. ⚙️ `execute_pull_mode()` - batch execution method
2. 🔄 Automatic burn-in calculation
3. 💾 Execution cache for intermediate results
4. ⚡ Parallel multi-asset execution
5. ✅ Pull vs push validation framework
6. 📊 Complete time series output
7. 📝 Comprehensive examples and docs

---

**Status:** Ready for implementation  
**Next Command:** `/agent-os/implement-tasks` to begin building!

