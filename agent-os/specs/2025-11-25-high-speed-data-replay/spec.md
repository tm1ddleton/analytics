# Specification: High-Speed Data Replay System

**Date:** 2025-11-25  
**Status:** Approved  
**Roadmap Item:** 7

---

## Goal

Build a high-speed data replay system that reads historical market data from SQLite and feeds it into the push-mode analytics engine at configurable speeds. This enables realistic simulation of live market data arrival for backtesting, visualization, and debugging analytics pipelines.

The replay engine should:
- Support configurable delay between data points (e.g., 100ms, 1s per trading day)
- Interleave multi-asset data in chronological order
- Provide progress callbacks for monitoring
- Handle errors gracefully without halting the entire replay
- Integrate seamlessly with the existing `PushModeEngine` and `SqliteDataProvider`

---

## User Stories

### Story 1: Backtest Analytics Pipeline
**As a** quantitative analyst  
**I want to** replay 3 months of historical data at high speed (e.g., 100ms per day)  
**So that I can** verify my volatility calculations match expected results over the full period in seconds rather than waiting for live data

**Acceptance Criteria:**
- Replay completes 60 trading days in ~6 seconds at 100ms delay
- All analytics update correctly via push-mode callbacks
- Final analytics values match batch computation results

### Story 2: Visualize Analytics Evolution
**As a** developer debugging analytics  
**I want to** replay data at human-visible speed (e.g., 500ms per day)  
**So that I can** watch analytics values evolve day-by-day and spot anomalies visually

**Acceptance Criteria:**
- Progress callback prints current date being replayed
- Analytics callbacks show updated values in real-time
- Replay is observable and can be followed by eye

### Story 3: Multi-Asset Simulation
**As a** portfolio manager  
**I want to** replay multiple assets (AAPL, MSFT, GOOG) simultaneously  
**So that I can** see how portfolio-level analytics update as data arrives chronologically

**Acceptance Criteria:**
- Data from all assets is interleaved by timestamp
- Analytics for each asset update independently
- Cross-asset analytics (e.g., correlations) see realistic data ordering

### Story 4: Resilient Replay
**As a** system operator  
**I want** replay to continue even if individual data points fail  
**So that** one bad data point doesn't halt my entire simulation

**Acceptance Criteria:**
- Failed `push_data()` calls are logged and skipped
- Replay continues with remaining data
- Summary reports successful and failed data point counts

---

## Specific Requirements

### 1. ReplayEngine Structure
- **Create** a `ReplayEngine` struct that manages replay state and configuration
- **Fields:** `DataProvider` reference, delay duration, optional progress callback, optional error callback
- **Methods:** `new()`, `set_delay()`, `set_progress_callback()`, `run()`

### 2. Configurable Delay
- **Support** delay between data points as `std::time::Duration`
- **Default:** 100ms (configurable by user)
- **Behavior:** Sleep for specified duration after each `push_data()` call
- **Note:** Delay is between consecutive trading days (daily data from Yahoo Finance)

### 3. Synchronous Execution
- **Implement** `run()` as a synchronous, blocking method
- **Returns** when replay completes or is stopped
- **Simple API:** No background threads or async complexity for initial version

### 4. Multi-Asset Chronological Interleaving
- **Query** data for all assets in the specified date range
- **Merge** data from all assets into a single sorted timeline by timestamp
- **Replay** in chronological order across all assets
- **Realistic Simulation:** Mimics how market data arrives in timestamp order

### 5. Date Range Specification
- **Accept** `Vec<AssetKey>` and `DateRange` as parameters to `run()`
- **Query** `SqliteDataProvider.query(asset, date_range)` for each asset
- **Explicit Control:** User specifies exactly what period to replay

### 6. Progress Callback
- **Provide** `set_progress_callback()` to register a callback: `Fn(DateTime<Utc>)`
- **Invoke** callback with the timestamp of each data point being replayed
- **Use Case:** Log progress, update UI, track simulation time

### 7. Data Callback
- **Accept** a data callback in `run()`: `Fn(AssetKey, DateTime<Utc>, f64) -> Result<(), PushError>`
- **Invoke** callback for each data point
- **Typical Usage:** Calls `engine.push_data(asset, timestamp, value)`
- **Flexibility:** User can inject custom logic (e.g., logging, validation) before push

### 8. Error Handling
- **Skip** data points that fail the data callback (e.g., `push_data()` errors)
- **Log** errors (asset, timestamp, error message)
- **Continue** replay with remaining data points
- **Return** summary: total points, successful, failed

### 9. Replay Result Summary
- **Return** `ReplayResult` struct with:
  - Total data points attempted
  - Successful data points
  - Failed data points
  - Start time, end time, elapsed duration
  - Simulated time range covered

### 10. Simple Stop Mechanism
- **Support** basic stop via result return (no pause/resume for now)
- **Clean Exit:** `run()` completes normally and returns `ReplayResult`
- **Future:** Could add `ReplayHandle` for pause/resume in later versions

### 11. Integration with Existing Systems
- **Use** `SqliteDataProvider` to query historical data
- **Use** `TimeSeriesPoint` for data representation
- **Feed** `PushModeEngine.push_data()` via callback
- **Leverage** existing error types (`PushError`, `DataProviderError`)

### 12. Example Program
- **Create** `examples/replay_volatility.rs` demonstrating:
  - Download AAPL data for 3 months via Yahoo Finance
  - Set up DAG with returns + volatility nodes
  - Initialize `PushModeEngine`
  - Create `ReplayEngine` with 100ms delay
  - Run replay and print final volatility

### 13. Testing Strategy
- **Unit Tests:** Replay with mock `DataProvider`, verify callback invocations
- **Integration Tests:** Replay with real `SqliteDataProvider` and `InMemoryDataProvider`
- **Multi-Asset Tests:** Verify chronological interleaving for AAPL + MSFT
- **Error Tests:** Inject failing callback, verify skip-and-continue behavior
- **Performance Tests:** Measure replay speed (data points per second)

---

## Existing Code to Leverage

### SqliteDataProvider
- **Path:** `src/sqlite_provider.rs`
- **Usage:** Query historical data for date ranges
- **Method:** `query(asset: &AssetKey, date_range: DateRange) -> Result<Vec<TimeSeriesPoint>, DataProviderError>`

### PushModeEngine
- **Path:** `src/push_mode.rs`
- **Usage:** Target for replayed data
- **Method:** `push_data(asset: AssetKey, timestamp: DateTime<Utc>, value: f64) -> Result<(), PushError>`
- **Callbacks:** Already handles analytics update notifications

### TimeSeriesPoint
- **Path:** `src/time_series.rs`
- **Structure:** `{ timestamp: DateTime<Utc>, close_price: f64, ... }`
- **Usage:** Standard data representation returned by `query()`

### DateRange
- **Path:** `src/time_series.rs`
- **Structure:** `{ start: NaiveDate, end: NaiveDate }`
- **Usage:** Specify replay period

### AssetKey
- **Path:** `src/asset_key.rs`
- **Usage:** Identify assets (AAPL, MSFT, etc.)
- **Methods:** `new_equity(ticker)`, equality, hashing

### YahooFinanceDownloader
- **Path:** `src/yahoo_finance.rs`
- **Usage:** Download historical data for replay examples
- **Method:** `download_to_sqlite(assets, date_range, db_path)`

### Error Handling Patterns
- **Existing Errors:** `PushError`, `DataProviderError`, `DagError`
- **New Error:** `ReplayError` for replay-specific issues
- **Pattern:** Use `Result<T, E>` consistently, log errors, continue on recoverable failures

---

## Out of Scope

### Not Included in This Iteration

1. **Pause/Resume Controls**
   - Future enhancement when needed for interactive debugging
   - Current: Simple start/stop only

2. **Skip Forward/Backward**
   - Future enhancement for non-linear replay
   - Current: Linear chronological replay only

3. **Variable Speed Adjustment During Replay**
   - Future: Dynamic speed slider
   - Current: Fixed delay set before `run()`

4. **Intraday/High-Frequency Data**
   - Out of scope: Yahoo Finance provides daily data only
   - Future: Would require different data source

5. **Real-Time Mode (1x Speed)**
   - Future: Replay at actual clock time (24 hours = 1 trading day)
   - Current: High-speed only (milliseconds per day)

6. **Replay from File Formats Other Than SQLite**
   - Out of scope: CSV, Parquet, JSON replay
   - Current: SQLite via `SqliteDataProvider` only

7. **Memory-Efficient Streaming**
   - Out of scope: Optimize for replaying years of data
   - Current: Load all data into memory, sort, replay (fine for daily data)

8. **Replay State Persistence/Recovery**
   - Out of scope: Save/resume replay from checkpoint
   - Current: Replay runs from start to finish

9. **UI Controls**
   - Out of scope: Handled by Roadmap Item 9 (Web Interface)
   - Current: Programmatic API only

10. **WebSocket Streaming**
    - Out of scope: Handled by Roadmap Item 8 (WebSocket API)
    - Current: In-process replay only

---

## Technical Design Notes

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        ReplayEngine                          │
│                                                              │
│  1. Query SqliteDataProvider for all assets                 │
│  2. Merge & sort data by timestamp                          │
│  3. For each data point:                                    │
│     a. Invoke data callback → PushModeEngine.push_data()    │
│     b. Sleep for delay duration                             │
│     c. Invoke progress callback (optional)                  │
│  4. Return ReplayResult summary                             │
└─────────────────────────────────────────────────────────────┘
           │                                      │
           │ query()                              │ push_data()
           ▼                                      ▼
  ┌─────────────────────┐            ┌──────────────────────┐
  │ SqliteDataProvider  │            │  PushModeEngine      │
  │                     │            │                      │
  │ - Daily OHLCV data  │            │ - Node state mgmt    │
  │ - Date range query  │            │ - Propagation        │
  └─────────────────────┘            │ - Analytics updates  │
                                      │ - User callbacks     │
                                      └──────────────────────┘
```

### Data Flow

1. **Load Phase**
   - Query `SqliteDataProvider` for each asset in `Vec<AssetKey>`
   - Collect all `Vec<TimeSeriesPoint>` results
   - Merge into single `Vec<(AssetKey, TimeSeriesPoint)>`
   - Sort by `TimeSeriesPoint.timestamp`

2. **Replay Phase**
   - Iterate through sorted data points
   - For each `(asset, point)`:
     - Call `data_callback(asset, point.timestamp, point.close_price)`
     - If callback succeeds: increment success counter
     - If callback fails: log error, increment failure counter, continue
     - Sleep for `delay` duration
     - Call `progress_callback(point.timestamp)` if set

3. **Summary Phase**
   - Return `ReplayResult` with counts and timing

### Error Scenarios

| Scenario | Behavior |
|----------|----------|
| `SqliteDataProvider.query()` fails | Return `Err(ReplayError::DataLoadFailed)` before replay starts |
| No data found for assets | Return `Err(ReplayError::NoDataFound)` |
| `data_callback()` fails (e.g., `push_data()` error) | Log error, skip data point, continue replay |
| `progress_callback()` panics | Catch panic, log warning, continue replay |
| User interrupts (Ctrl+C) | Out of scope for now (future: graceful shutdown) |

### Performance Considerations

- **Daily Data:** 60 trading days (3 months) × 3 assets = 180 data points
- **Replay Speed:** 100ms delay → 18 seconds total
- **Memory:** ~10KB per asset-month (negligible for daily data)
- **Sorting:** O(n log n) where n = total data points (cheap for daily data)

### Example Timing

| Data Period | Assets | Data Points | Delay | Total Time |
|-------------|--------|-------------|-------|------------|
| 1 month | 1 | 20 | 100ms | 2s |
| 3 months | 1 | 60 | 100ms | 6s |
| 3 months | 3 | 180 | 100ms | 18s |
| 1 year | 5 | 1,260 | 50ms | 63s |
| 1 year | 5 | 1,260 | 10ms | 12.6s |

---

## Implementation Checklist

- [ ] Define `ReplayEngine` struct
- [ ] Implement `new()` and configuration methods
- [ ] Implement data loading and sorting
- [ ] Implement `run()` with replay loop
- [ ] Implement error handling and skip-on-error
- [ ] Define `ReplayResult` and return summary
- [ ] Add progress and error callbacks
- [ ] Write unit tests (mock data)
- [ ] Write integration tests (real SQLite)
- [ ] Create `examples/replay_volatility.rs`
- [ ] Create `examples/replay_multi_asset.rs`
- [ ] Document usage in `README.md`

---

**Next Steps:** Create tasks.md with detailed task breakdown.

