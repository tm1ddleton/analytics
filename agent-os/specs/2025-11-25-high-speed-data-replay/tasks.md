# Tasks: High-Speed Data Replay System

**Spec:** `agent-os/specs/2025-11-25-high-speed-data-replay/spec.md`  
**Date:** 2025-11-25  
**Status:** Ready for Implementation

---

## Task Groups Overview

- **Task Group 1:** Core ReplayEngine Structure (Foundation)
- **Task Group 2:** Data Loading and Sorting (Multi-Asset Support)
- **Task Group 3:** Replay Execution Loop (Main Logic)
- **Task Group 4:** Progress and Callback System (Observability)
- **Task Group 5:** Error Handling and Resilience (Robustness)
- **Task Group 6:** Integration Testing (Quality)
- **Task Group 7:** Examples and Documentation (Usability)

**Estimated Total Complexity:** Medium (7 task groups, ~30 sub-tasks)

---

## Task Group 1: Core ReplayEngine Structure

**Goal:** Create the foundational `ReplayEngine` struct with configuration API

**Dependencies:** None (can start immediately)

**Acceptance Criteria:**
- `ReplayEngine` struct compiles with all required fields
- Configuration methods work correctly
- Basic instantiation tests pass

---

### Tasks

#### Task 1.1: Define ReplayEngine struct and error types
- [x] Create `src/replay.rs` file
- [x] Define `ReplayEngine` struct with fields:
  - `provider: Arc<dyn DataProvider>` (for querying historical data)
  - `delay: Duration` (configurable delay between data points)
  - `progress_callback: Option<Box<dyn Fn(DateTime<Utc>)>>` (optional progress notification)
- [x] Define `ReplayError` enum:
  - `DataLoadFailed(String)` - failed to load data from provider
  - `NoDataFound` - no data available for specified assets/range
  - `InvalidDateRange` - invalid date range specification
  - `CallbackError(String)` - error in user callback
- [x] Define `ReplayResult` struct:
  - `total_points: usize` - total data points attempted
  - `successful: usize` - successfully replayed
  - `failed: usize` - failed to replay
  - `start_time: DateTime<Utc>` - when replay started
  - `end_time: DateTime<Utc>` - when replay finished
  - `elapsed: Duration` - wall-clock time
  - `simulated_start: DateTime<Utc>` - first data timestamp
  - `simulated_end: DateTime<Utc>` - last data timestamp
- [x] Implement `Display` for `ReplayError`
- [x] Add `#[derive(Debug, Clone)]` where appropriate

**Estimated Time:** 30 minutes

---

#### Task 1.2: Implement ReplayEngine constructor and configuration
- [x] Implement `ReplayEngine::new(provider: Arc<dyn DataProvider>) -> Self`
  - Default delay: 100ms
  - No callbacks initially
- [x] Implement `set_delay(&mut self, delay: Duration) -> &mut Self`
  - Builder pattern for chaining
  - Validate delay > 0
- [x] Implement `set_progress_callback<F>(&mut self, callback: F) -> &mut Self`
  - Where `F: Fn(DateTime<Utc>) + 'static`
  - Store as `Option<Box<dyn Fn(DateTime<Utc>)>>`
- [x] Implement builder pattern for ergonomic configuration
- [x] Add `pub use replay::{ReplayEngine, ReplayError, ReplayResult};` to `src/lib.rs`

**Estimated Time:** 30 minutes

---

#### Task 1.3: Write basic tests for ReplayEngine construction
- [x] Test `new()` creates engine with default delay (100ms)
- [x] Test `set_delay()` updates delay correctly
- [x] Test `set_delay()` chains properly (builder pattern)
- [x] Test `set_progress_callback()` stores callback
- [x] Test configuration with method chaining

**Test Count:** 5 tests

**Estimated Time:** 20 minutes

---

## Task Group 2: Data Loading and Sorting

**Goal:** Implement multi-asset data loading with chronological interleaving

**Dependencies:** Task Group 1

**Acceptance Criteria:**
- Loads data for multiple assets correctly
- Merges and sorts by timestamp
- Handles empty data gracefully
- Tests verify correct chronological ordering

---

### Tasks

#### Task 2.1: Implement data loading for single asset
- [x] Create helper method `load_asset_data()`
  - Parameters: `asset: &AssetKey, date_range: DateRange`
  - Query `self.provider.query(asset, date_range)`
  - Return `Result<Vec<(AssetKey, TimeSeriesPoint)>, ReplayError>`
  - Map `DataProviderError` to `ReplayError::DataLoadFailed`
- [x] Tag each `TimeSeriesPoint` with its `AssetKey`
- [x] Handle empty results (return empty Vec, not error)

**Estimated Time:** 20 minutes

---

#### Task 2.2: Implement multi-asset loading and merging
- [x] Create method `load_and_sort_data()`
  - Parameters: `assets: Vec<AssetKey>, date_range: DateRange`
  - Call `load_asset_data()` for each asset
  - Collect results into `Vec<(AssetKey, TimeSeriesPoint)>`
  - Return error if any query fails
- [x] Merge all asset data into single vector
- [x] Sort by `TimeSeriesPoint.timestamp` (chronological order)
- [x] Return `Err(ReplayError::NoDataFound)` if no data for any asset

**Estimated Time:** 30 minutes

---

#### Task 2.3: Write tests for data loading
- [x] Test loading single asset returns correct data points
- [x] Test loading multiple assets merges correctly
- [x] Test chronological sorting with interleaved timestamps
  - Create mock data: AAPL on day 1, MSFT on day 2, AAPL on day 3
  - Verify sorted order: day 1 (AAPL), day 2 (MSFT), day 3 (AAPL)
- [x] Test empty data returns `NoDataFound` error
- [x] Test `DataProviderError` maps to `ReplayError::DataLoadFailed`
- [x] Use `InMemoryDataProvider` for testing

**Test Count:** 5-6 tests

**Estimated Time:** 30 minutes

---

## Task Group 3: Replay Execution Loop

**Goal:** Implement the core `run()` method that replays data with timing

**Dependencies:** Task Groups 1, 2

**Acceptance Criteria:**
- `run()` method executes replay synchronously
- Invokes data callback for each point
- Respects delay between data points
- Returns `ReplayResult` with accurate counts and timing

---

### Tasks

#### Task 3.1: Implement run() method signature
- [x] Define `run<F>()` method signature:
  ```rust
  pub fn run<F>(
      &mut self,
      assets: Vec<AssetKey>,
      date_range: DateRange,
      mut data_callback: F,
  ) -> Result<ReplayResult, ReplayError>
  where
      F: FnMut(AssetKey, DateTime<Utc>, f64) -> Result<(), Box<dyn std::error::Error>>,
  ```
- [x] Document parameters and return type
- [x] Add usage example in docstring

**Estimated Time:** 15 minutes

---

#### Task 3.2: Implement replay loop logic
- [x] Load and sort data using `load_and_sort_data()`
- [x] Initialize counters: `total`, `successful`, `failed`
- [x] Record `start_time` before loop
- [x] For each `(asset, point)` in sorted data:
  - Invoke `data_callback(asset, point.timestamp, point.close_price)`
  - If callback succeeds: increment `successful`
  - If callback fails: increment `failed`, continue (don't return error)
  - Sleep for `self.delay` using `std::thread::sleep()`
  - Invoke progress callback if set
- [x] Record `end_time` after loop
- [x] Calculate `elapsed = end_time - start_time`
- [x] Build and return `ReplayResult`

**Estimated Time:** 45 minutes

---

#### Task 3.3: Implement ReplayResult construction
- [x] Extract `simulated_start` from first data point timestamp
- [x] Extract `simulated_end` from last data point timestamp
- [x] Populate all `ReplayResult` fields
- [x] Implement `Display` for `ReplayResult` to show summary:
  ```
  Replay complete: 180 points (178 successful, 2 failed)
  Simulated: 2024-01-01 to 2024-03-31
  Elapsed: 18.2s
  ```

**Estimated Time:** 20 minutes

---

#### Task 3.4: Write tests for replay execution
- [x] Test `run()` calls data callback for each point
  - Use counter in callback to verify invocation count
- [x] Test `run()` respects delay between points
  - Set 50ms delay, verify elapsed time ≥ (num_points × 50ms)
- [x] Test `run()` returns correct `ReplayResult` counts
- [x] Test `run()` with zero data points returns `NoDataFound`
- [x] Use `InMemoryDataProvider` for deterministic testing

**Test Count:** 4-5 tests

**Estimated Time:** 40 minutes

---

## Task Group 4: Progress and Callback System

**Goal:** Implement progress notifications and callback management

**Dependencies:** Task Group 3

**Acceptance Criteria:**
- Progress callback invoked with correct timestamps
- Callback can be optional (None)
- Callback errors don't crash replay

---

### Tasks

#### Task 4.1: Implement progress callback invocation
- [x] In replay loop, after each data point:
  - Check if `self.progress_callback.is_some()`
  - If yes, call `callback(point.timestamp)`
- [x] Wrap callback invocation in panic catch (std::panic::catch_unwind)
  - If callback panics, log warning and continue
- [x] Add test: progress callback receives all timestamps in order

**Estimated Time:** 25 minutes

---

#### Task 4.2: Implement error callback (optional enhancement)
- [x] Add `error_callback: Option<Box<dyn Fn(&AssetKey, &DateTime<Utc>, &str)>>` to `ReplayEngine`
- [x] Implement `set_error_callback()` method
- [x] In replay loop, when data callback fails:
  - Invoke error callback with asset, timestamp, error message
- [x] Add test: error callback receives failed data point info

**Estimated Time:** 25 minutes

---

#### Task 4.3: Write tests for callback system
- [x] Test progress callback invoked for each data point
- [x] Test progress callback receives correct timestamps
- [x] Test replay works without progress callback (None)
- [x] Test callback panic doesn't crash replay
- [x] Test error callback receives failure information

**Test Count:** 5 tests

**Estimated Time:** 30 minutes

---

## Task Group 5: Error Handling and Resilience

**Goal:** Ensure replay handles errors gracefully and continues when possible

**Dependencies:** Task Groups 3, 4

**Acceptance Criteria:**
- Failed data callbacks don't halt replay
- Errors are logged with context
- ReplayResult accurately reflects failures
- Tests verify skip-and-continue behavior

---

### Tasks

#### Task 5.1: Implement skip-on-error logic
- [x] In replay loop, wrap data callback in `match` or `if let Err(e)`
- [x] On callback error:
  - Log error: `log::warn!("Failed to replay {asset} at {timestamp}: {error}")`
  - Increment `failed` counter
  - Continue to next data point (don't break or return)
- [x] Ensure `ReplayResult.failed` count is accurate

**Estimated Time:** 20 minutes

---

#### Task 5.2: Add comprehensive error logging
- [x] Log start of replay: `log::info!("Starting replay: {num_assets} assets, {num_points} points")`
- [x] Log each error with full context: asset, timestamp, error message
- [x] Log end of replay with summary: `log::info!("Replay complete: {successful}/{total} successful")`
- [x] Use appropriate log levels (info, warn, error)

**Estimated Time:** 20 minutes

---

#### Task 5.3: Write error handling tests
- [x] Test data callback that always fails
  - Verify replay completes without crashing
  - Verify `failed` count equals total points
  - Verify `successful` count is zero
- [x] Test data callback that fails on specific asset
  - Verify only that asset's points fail
  - Verify other assets succeed
- [x] Test data callback that fails intermittently
  - Verify mixed success/failure counts are correct
- [x] Test error logging output (capture logs in test)

**Test Count:** 4-5 tests

**Estimated Time:** 40 minutes

---

## Task Group 6: Integration Testing

**Goal:** Verify ReplayEngine works with real PushModeEngine and SqliteDataProvider

**Dependencies:** Task Groups 1-5

**Acceptance Criteria:**
- Integration test with `PushModeEngine` passes
- Integration test with `SqliteDataProvider` passes
- Multi-asset replay produces correct analytics
- End-to-end workflow documented

---

### Tasks

#### Task 6.1: Write integration test with PushModeEngine
- [x] Create test `test_replay_with_push_mode_engine()`
- [x] Set up:
  - Create `InMemoryDataProvider` with 20 days of AAPL data
  - Build DAG with DataProvider → Returns → Volatility nodes
  - Create `PushModeEngine` and initialize
  - Create `ReplayEngine` with 10ms delay
- [x] Execute:
  - Run replay with callback to `engine.push_data()`
  - Track analytics updates via callbacks
- [x] Verify:
  - All 20 data points replayed successfully
  - Final volatility matches expected value
  - `ReplayResult` shows correct counts

**Estimated Time:** 45 minutes

---

#### Task 6.2: Write integration test with SqliteDataProvider
- [x] Create test `test_replay_with_sqlite_provider()`
- [x] Set up:
  - Create temp SQLite database
  - Insert 30 days of test data for AAPL and MSFT
  - Create `SqliteDataProvider` pointing to temp DB
  - Create `ReplayEngine`
- [x] Execute:
  - Run replay for both assets
  - Verify chronological interleaving
- [x] Verify:
  - Data loaded correctly from SQLite
  - Both assets replayed in timestamp order
  - `ReplayResult` shows 30 points per asset = 60 total

**Estimated Time:** 45 minutes

---

#### Task 6.3: Write multi-asset analytics integration test
- [x] Create test `test_multi_asset_replay_with_analytics()`
- [x] Set up:
  - Create data for AAPL, MSFT, GOOG (20 days each)
  - Build separate analytics DAG for each asset
  - Create `PushModeEngine` with all DAGs
  - Create `ReplayEngine`
- [x] Execute:
  - Run replay with callback to push data
  - Collect final analytics for all assets
- [x] Verify:
  - Each asset's analytics computed correctly
  - Cross-asset ordering is chronological
  - No interference between assets

**Estimated Time:** 50 minutes

---

#### Task 6.4: Write performance benchmark test
- [x] Create test `test_replay_performance()`
- [x] Set up:
  - Create 252 days of data (1 year) for 5 assets = 1,260 points
  - Create `ReplayEngine` with 10ms delay
- [x] Execute:
  - Run replay and measure elapsed time
  - Calculate points per second
- [x] Verify:
  - Elapsed time ≈ 1,260 × 10ms = 12.6 seconds (allow ±20% variance)
  - No performance degradation
  - Memory usage stable

**Estimated Time:** 30 minutes

---

## Task Group 7: Examples and Documentation

**Goal:** Create example programs demonstrating replay usage and update documentation

**Dependencies:** Task Groups 1-6 (all implementation complete)

**Acceptance Criteria:**
- Example programs compile and run correctly
- Examples demonstrate key use cases
- Documentation updated with replay usage
- README includes quick start guide

---

### Tasks

Task 7.1: Create examples/replay_volatility.rs
- [x] Create example program that:
  - Downloads 3 months of AAPL data via Yahoo Finance (if not cached)
  - Stores in SQLite database
  - Creates DAG with Returns → Volatility (10-day) nodes
  - Creates `PushModeEngine` and initializes
  - Creates `ReplayEngine` with 100ms delay
  - Adds progress callback to print current date
  - Adds analytics callback to print volatility updates
  - Runs replay and prints final volatility
- [x] Add clear console output showing replay progress
- [x] Add comments explaining each step
- [x] Test by running: `cargo run --example replay_volatility`

**Estimated Time:** 45 minutes

---

Task 7.2: Create examples/replay_multi_asset.rs
- [x] Create example program that:
  - Downloads data for AAPL, MSFT, GOOG (3 months each)
  - Creates separate analytics for each asset (returns + volatility)
  - Creates `ReplayEngine` with 50ms delay
  - Replays all three assets chronologically
  - Prints updates showing which asset is being processed
  - Displays final analytics for all three assets
- [x] Demonstrate chronological interleaving in output
- [x] Add table summarizing final analytics per asset
- [x] Test by running: `cargo run --example replay_multi_asset`

**Estimated Time:** 50 minutes

---

#### Task 7.3: Create examples/replay_with_errors.rs
- [ ] Create example program that:
  - Creates test data with some intentionally invalid values
  - Creates data callback that rejects invalid values
  - Runs replay with error callback that logs failures
  - Prints `ReplayResult` showing successful and failed counts
- [ ] Demonstrates error handling and skip-and-continue behavior
- [ ] Shows how to use error callback for debugging
- [ ] Test by running: `cargo run --example replay_with_errors`

**Estimated Time:** 40 minutes

---

#### Task 7.4: Update documentation
- [ ] Add "Data Replay" section to main README.md
  - Overview of replay system
  - Quick start example
  - Link to example programs
- [ ] Add doc comments to all public methods in `src/replay.rs`
  - Include usage examples in docstrings
  - Document error conditions
  - Document performance characteristics
- [ ] Update CHANGELOG.md with new feature
- [ ] Add replay to architecture diagram (if exists)

**Estimated Time:** 40 minutes

---

#### Task 7.5: Create integration example with roadmap items 8 & 9
- [x] Add placeholder comment in `replay.rs`:
  ```rust
  // TODO (Item 8): Add WebSocket streaming support
  // TODO (Item 9): Add UI controls for pause/resume
  ```
- [x] Document how replay will integrate with future items
- [x] Create stub for future `ReplayHandle` (async control)

**Estimated Time:** 15 minutes

---

## Summary

### Task Group Completion Order

```
Task Group 1 (Core Structure)
     ↓
Task Group 2 (Data Loading)
     ↓
Task Group 3 (Replay Loop)
     ↓
Task Group 4 (Callbacks) + Task Group 5 (Error Handling)
     ↓
Task Group 6 (Integration Tests)
     ↓
Task Group 7 (Examples & Docs)
```

### Estimated Total Time

| Task Group | Estimated Time |
|------------|----------------|
| TG1: Core Structure | 1.5 hours |
| TG2: Data Loading | 1.5 hours |
| TG3: Replay Loop | 2 hours |
| TG4: Callbacks | 1.5 hours |
| TG5: Error Handling | 1.5 hours |
| TG6: Integration Tests | 3 hours |
| TG7: Examples & Docs | 3 hours |
| **Total** | **~14 hours** |

### Key Milestones

- ✅ **Milestone 1** (TG1): Basic `ReplayEngine` compiles
- ✅ **Milestone 2** (TG2): Multi-asset data loading works
- ✅ **Milestone 3** (TG3): First successful replay completes
- ✅ **Milestone 4** (TG4-5): Callbacks and error handling robust
- ✅ **Milestone 5** (TG6): Integration with `PushModeEngine` verified
- ✅ **Milestone 6** (TG7): Production-ready with examples

### Testing Strategy

- **Unit Tests:** 25-30 tests covering individual components
- **Integration Tests:** 4-5 tests with real `PushModeEngine` and `SqliteDataProvider`
- **Example Programs:** 3 runnable examples demonstrating use cases
- **Performance Tests:** Benchmark replay speed and memory usage

---

**Status:** Ready for implementation  
**Next Command:** `/agent-os/implement-tasks` to begin building!

