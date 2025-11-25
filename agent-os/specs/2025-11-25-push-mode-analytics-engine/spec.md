# Specification: Push-Mode Analytics Engine

## Goal
Implement an incremental computation system where analytics automatically update when new data arrives, propagating changes through the DAG dependency chain with node-local state management, callback notifications, and support for simulation/replay scenarios at human-visible speeds.

## User Stories
- As a simulation operator, I want to replay historical data through the analytics engine at controlled speeds so that I can visualize how analytics evolve over time
- As a strategy developer, I want to receive immediate notifications when analytics update so that my strategies can react to new information in real-time
- As a system architect, I want nodes to maintain their own state and buffers so that the system scales independently per analytic
- As a developer, I want the engine to warm up from historical data so that analytics have proper context from the first incremental update

## Specific Requirements

### 1. Push Data API
- Explicit API: `push_data(asset: AssetKey, timestamp: DateTime, value: f64) -> Result<(), PushError>`
- Accept timestamped data points for simulation/replay scenarios
- Validate that timestamp > last_computed_timestamp per node (reject out-of-order)
- Support sequential processing (one data point fully propagates before next)
- Return error if propagation fails

### 2. Node State Management
- Each node maintains:
  - `last_computed_timestamp: Option<DateTime>` - tracks latest computation
  - `output_history: Vec<TimeSeriesPoint>` - full output history (for now)
  - `input_buffer: CircularBuffer<f64>` - fixed-size buffer for rolling windows
  - `state: NodeState` enum (Uninitialized, Ready, Computing, Failed)
- State is node-local (no global state manager)
- Nodes track their own dependencies and readiness
- Out-of-order data (timestamp < last_computed) is rejected/ignored

### 3. Circular Buffer Implementation
- Fixed-size pre-allocated buffers for rolling window analytics
- Support partial fills: use available data when buffer not full
- Example: 30-day volatility with only 10 days → compute std dev of 10 days
- Efficient push/pop operations with O(1) complexity
- Automatic wraparound for circular behavior

### 4. Synchronous Propagation
- When `push_data()` called:
  1. Identify affected nodes (nodes with this asset as input)
  2. Update source nodes (data provider wrappers)
  3. Propagate through DAG in topological order
  4. Each node computes synchronously before next node starts
  5. Return when entire propagation wave completes
- Use existing `topological_sort()` from DAG framework
- Leverage `get_descendants()` to find affected subgraph

### 5. Callback Notification System
- API: `register_callback(node_id: NodeId, callback: Box<dyn Fn(&NodeOutput)>)`
- Callbacks invoked immediately after each node computes
- Support multiple callbacks per node
- Pass `NodeOutput` to callback (includes updated value and metadata)
- Callbacks execute synchronously (async execution of nodes is separate concern)
- Errors in callbacks logged but don't halt propagation

### 6. Multi-Asset Sequential Processing
- Process assets one at a time in arrival order
- If AAPL updates, complete entire AAPL propagation before processing MSFT
- No parallelism across assets (simplifies reasoning, deterministic)
- Within single asset update, nodes can be marked for async execution (future)
- Queue data points if they arrive during active propagation

### 7. Historical Initialization (Warmup)
- Before first incremental update, bootstrap from DataProvider:
  1. Query historical data for required lookback period
  2. Populate node buffers (e.g., 30 days for 30-day volatility)
  3. Compute initial states for all nodes
  4. Set `last_computed_timestamp` to last historical point
- Warmup API: `initialize(end_date: DateTime, lookback_days: usize) -> Result<(), InitError>`
- Use existing `DataProvider` trait for historical queries
- Calculate required lookback from DAG (max of all node requirements)

### 8. Error Handling & Resilience
- Node computation errors:
  - Log error with node details
  - Mark node as `Failed` state
  - Skip failed node's dependents in this propagation
  - Continue propagating to other branches
  - Next update may recover (node resets to `Ready`)
- Callback errors:
  - Log error but continue propagation
  - Don't re-throw to caller
- Data validation errors:
  - Reject invalid data (NaN, negative prices, out-of-order)
  - Return descriptive error to caller
  - Don't update any node state

### 9. Buffer Management Details
- Pre-allocate buffers during node creation
- Size determined by node parameters (e.g., `window_size: 30`)
- Circular buffer operations:
  - `push(value)` - add new value, overwrite oldest if full
  - `get_slice()` - return view of current buffer contents
  - `len()` - current fill level (may be < capacity)
  - `is_full()` - check if at capacity
- For partial buffers, analytics use all available data
- Example: volatility with 10/30 days → std dev of those 10 days

### 10. Simulation/Replay Support
- `push_data()` respects timestamp ordering
- Engine validates timestamp > last_computed per node
- Updates designed to be visible at human speeds (not instant)
- Support for replay system to control data feed rate
- Timestamps used for analytics that depend on time (e.g., annualized returns)
- Historical order enforced (critical for accurate analytics)

### 11. Output History Storage
- Each node stores `Vec<TimeSeriesPoint>` of all computed outputs
- Append-only (no deletion or modification)
- Query API: `get_history(node_id: NodeId) -> Vec<TimeSeriesPoint>`
- Query API: `get_latest(node_id: NodeId) -> Option<TimeSeriesPoint>`
- Future: Async writes to persistent datastore (out of scope for now)
- Memory management: Consider limits for long-running simulations

### 12. Cold Start & Initialization Workflow
```rust
// 1. Create engine with DAG
let mut engine = PushModeEngine::new(dag);

// 2. Warmup from historical data
engine.initialize(
    end_date: NaiveDate::from_ymd(2024, 1, 1),
    lookback_days: 30,  // Max required by any node
)?;

// 3. Now ready for incremental updates
engine.push_data(
    asset: AssetKey::new_equity("AAPL")?,
    timestamp: datetime(2024, 1, 2, 9, 30),
    value: 150.0,
)?;
```

### 13. Notification Timing & Async Execution
- Callbacks invoked after each individual node completes
- Enables fine-grained UI updates (data points move graph incrementally)
- Node execution is synchronous within propagation wave
- Future: Mark nodes for async/parallel execution (out of scope)
- Callback execution is synchronous (blocks propagation)
- For async callbacks, use channels/queues internally (future)

---

## Existing Code to Leverage

### DAG Framework (`src/dag.rs`)
- `AnalyticsDag` - DAG construction and management
- `Node` struct with `NodeParams`, assets, node_type
- `NodeId` for identification
- `topological_sort()` - execution order
- `get_descendants(node_id)` - dependency chains
- `execute_incremental()` - placeholder method to implement
- `DagError` - error types

### Analytics Module (`src/analytics.rs`)
- `calculate_returns()` - stateless returns function
- `calculate_volatility()` - rolling window volatility
- `execute_returns_node()`, `execute_volatility_node()` - node executors
- `NodeOutput` enum - output types
- Query builders for DAG construction

### Data Layer
- `DataProvider` trait - historical data access
- `InMemoryDataProvider`, `SqliteDataProvider` implementations
- `TimeSeriesPoint` - (timestamp, value) structure
- `DateRange` - date range queries
- `AssetKey` - asset identification

### Time Series (`src/time_series.rs`)
- Existing time series structures
- Date/time handling with `chrono`

---

## Out of Scope

1. **Out-of-Order Data Handling** - Timestamps must be monotonically increasing
2. **Async Datastore Writes** - History stays in-memory for now
3. **Parallel Multi-Asset Processing** - Sequential only
4. **Node Retry Mechanisms** - Failed nodes stay failed until next update
5. **Backpressure Handling** - No flow control for rapid streams
6. **Distributed Execution** - Single-process only
7. **Persistent State Recovery** - No state serialization/recovery
8. **Complex Event Processing** - Simple data point updates only
9. **Real-Time Data Ingestion** - Separate feature (Item 15)
10. **Performance Optimizations** - No SIMD, GPU, or advanced optimizations

---

## Implementation Notes

### Circular Buffer
Consider using `std::collections::VecDeque` or custom implementation with:
- Fixed capacity
- Efficient push/pop
- Slice view without copying

### Callback Storage
```rust
type Callback = Box<dyn Fn(&NodeOutput) + Send + Sync>;
HashMap<NodeId, Vec<Callback>>
```

### Node State Enum
```rust
enum NodeState {
    Uninitialized,  // Before warmup
    Ready,          // Ready for updates
    Computing,      // Currently updating
    Failed(String), // Computation failed
}
```

### Error Types
```rust
enum PushError {
    OutOfOrder { timestamp, last_computed },
    InvalidData(String),
    PropagationFailed { node_id, error },
    EngineNotInitialized,
}

enum InitError {
    DataProviderError(DataProviderError),
    InsufficientHistoricalData,
    NodeInitializationFailed { node_id, error },
}
```

---

## Integration Points

### With High-Speed Replay (Item 7)
Replay system will:
1. Read historical data from SQLite
2. Call `engine.push_data()` for each data point
3. Control timing to make updates visible
4. Callbacks update UI/metrics

### With REST API/WebSocket (Item 8)
WebSocket subscriptions will:
1. Register callbacks via `register_callback()`
2. Serialize `NodeOutput` to JSON
3. Push to WebSocket clients
4. Handle client disconnections

### With React UI (Item 9)
UI will:
1. Subscribe to analytics via WebSocket
2. Render incremental updates as they arrive
3. Display charts that update point-by-point
4. Control replay speed

---

## Success Criteria

- ✅ `push_data()` API accepts timestamped data and propagates through DAG
- ✅ Nodes maintain their own state and circular buffers
- ✅ Out-of-order data is rejected
- ✅ Callbacks fire after each node update
- ✅ Failed nodes don't halt propagation
- ✅ `initialize()` warms up from historical DataProvider
- ✅ Sequential multi-asset processing
- ✅ Nodes store full output history
- ✅ Integration tests with returns and volatility analytics
- ✅ Simulation replay scenario works end-to-end

---

**Version:** 1.0  
**Date:** 2025-11-25  
**Status:** Ready for Implementation

