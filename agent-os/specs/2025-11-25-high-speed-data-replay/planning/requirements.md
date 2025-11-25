# Requirements Gathering: High-Speed Data Replay System

## Clarifying Questions & Answers

### 1. Speed Control & Timing
**Q:** How should replay speed be specified?  
**A:** Option B - Delay between data points (10ms, 100ms, 1s)

**Decision Rationale:** Simple, direct control. Since data is daily from Yahoo Finance, delay between days is straightforward.

### 2. Execution Model
**Q:** Should replay be synchronous or async?  
**A:** Option B - Synchronous (blocks until complete)

**Decision Rationale:** Simpler implementation, easier to reason about. Good for initial POC.

### 3. Control Interface
**Q:** What controls are needed?  
**A:** Just start/stop

**Decision Rationale:** Minimal interface for POC. Pause/resume/skip can be added later if needed.

### 4. Multi-Asset Replay
**Q:** When replaying multiple assets?  
**A:** Option A - Replay in parallel (interleave by timestamp)

**Decision Rationale:** Realistic simulation of market data arriving in chronological order.

### 5. Data Source
**Q:** Should replay read directly from SqliteDataProvider?  
**A:** Don't care

**Decision Rationale:** Implementation detail - choose most efficient approach.

### 6. Progress & Callbacks
**Q:** What progress information is needed?  
**A:** Current date being replayed

**Decision Rationale:** Simple progress tracking. User can see simulation advancing through time.

### 7. Error Handling
**Q:** If push_data() fails during replay?  
**A:** Skip that data point and continue

**Decision Rationale:** Resilient replay - don't halt entire simulation for one bad data point.

### 8. Date Range
**Q:** Should user specify date range?  
**A:** Full date range upfront

**Decision Rationale:** Explicit control over simulation period.

---

## Additional Context

### Data Characteristics
- **Source:** Yahoo Finance via YahooFinanceDownloader
- **Frequency:** Daily data points only
- **Timestamp:** One data point per trading day per asset
- **No intraday data:** Simplifies timing model

### Key Design Implications

**Daily Data:**
- Delay is between trading days, not seconds/minutes
- Example: 100ms delay = 100ms between consecutive days
- 3 months (~60 trading days) at 100ms = 6 seconds total
- 3 months at 1s delay = 60 seconds total

**Synchronous Execution:**
- Replay method blocks until complete
- Simple API: `replay.run()` returns when done
- No threading complexity for initial version

**Timestamp Interleaving:**
- Sort all data points by timestamp across assets
- Replay in chronological order
- Realistic market simulation

**Progress Callback:**
- Simple callback with current date: `on_progress(DateTime<Utc>)`
- Called once per trading day
- Can update UI or log progress

---

## Existing Code to Reference

### From Yahoo Finance Downloader
- ✅ `YahooFinanceDownloader` - downloads historical data
- ✅ `download_to_sqlite()` - stores in SQLite
- ✅ Already have data ingestion working

### From SQLite Provider
- ✅ `SqliteDataProvider` - reads historical data
- ✅ `query()` method for date range queries
- ✅ Returns `Vec<TimeSeriesPoint>`

### From Push-Mode Engine
- ✅ `PushModeEngine` - target for replay
- ✅ `push_data(asset, timestamp, value)` - feed data
- ✅ Callbacks for analytics updates

### From Time Series
- ✅ `TimeSeriesPoint` - (timestamp, value) structure
- ✅ `DateRange` - date range specification

---

## Out of Scope (For This Iteration)

1. Pause/resume functionality
2. Skip forward/backward
3. Variable speed adjustment during replay
4. Intraday/high-frequency data
5. Real-time mode (1x speed matching actual time)
6. Replay from file formats other than SQLite
7. Memory-efficient streaming for very large datasets
8. Replay state persistence/recovery
9. UI controls (handled by Item 9)
10. WebSocket streaming (handled by Item 8)

---

## Visual Assets
None provided.

---

## Example Usage

```rust
// 1. Load historical data (already in SQLite from Yahoo Finance)
let provider = SqliteDataProvider::new("data.db")?;

// 2. Create push-mode engine with analytics DAG
let mut engine = PushModeEngine::new(dag);
engine.initialize(&provider, end_date, lookback)?;

// 3. Create replay engine
let mut replay = ReplayEngine::new(provider);

// 4. Configure replay
replay.set_delay(Duration::from_millis(100)); // 100ms between days
replay.set_progress_callback(|date| {
    println!("Replaying: {}", date);
});

// 5. Run replay (blocks until complete)
let assets = vec![
    AssetKey::new_equity("AAPL")?,
    AssetKey::new_equity("MSFT")?,
];

let date_range = DateRange::new(
    NaiveDate::from_ymd(2024, 1, 1),
    NaiveDate::from_ymd(2024, 3, 31),
);

replay.run(assets, date_range, |data_point| {
    engine.push_data(data_point.asset, data_point.timestamp, data_point.value)?;
    Ok(())
})?;

// 6. Query final analytics
let volatility = engine.get_latest(vol_node)?;
println!("Final volatility: {:?}", volatility);
```

---

**Date:** 2025-11-25  
**Status:** Requirements Complete

