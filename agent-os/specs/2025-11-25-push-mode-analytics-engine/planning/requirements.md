# Requirements Gathering: Push-Mode Analytics Engine

## Clarifying Questions & Answers

### 1. Data Arrival & Triggering
**Q:** How should new data trigger the system?  
**A:** Option A - Explicit API call: `engine.push_data(asset, timestamp, value)` for simulation/replay

**Decision Rationale:** Enables precise control for replay simulations from DataProvider

### 2. State Management
**Q:** How do we track what's been computed?  
**A:** 
- Each node maintains its own state
- Track last computed timestamp per node
- Ignore out-of-order data arrivals

**Decision Rationale:** Simple, node-local state management. Out-of-order handling deferred.

### 3. Rolling Window Analytics
**Q:** How do we maintain state for analytics needing historical data?  
**A:** Nodes maintain fixed-size buffers

**Decision Rationale:** Efficient memory usage, predictable performance

### 4. Propagation Strategy
**Q:** How do updates propagate through DAG?  
**A:** Option A - Immediate/synchronous computation of all dependents

**Decision Rationale:** Simplifies reasoning, ensures consistency

### 5. Subscription/Notification
**Q:** How do consumers get notified of updates?  
**A:** Option A - Callback functions registered per node

**Decision Rationale:** Direct, explicit notification mechanism

### 6. Multi-Asset Coordination
**Q:** If multiple assets update, how do we process them?  
**A:** Sequential processing in arrival order

**Decision Rationale:** Deterministic, easier to reason about and debug

### 7. Historical vs. Incremental
**Q:** What modes should push-mode handle?  
**A:** Initialization from backfill (historical data), then incremental updates

**Decision Rationale:** Warm start with historical data, then switch to real-time updates

### 8. Error Handling
**Q:** If node fails during update, what happens?  
**A:** Skip failed node and continue to other branches

**Decision Rationale:** Resilient system, partial failures don't halt entire pipeline

### 9. Buffer Management
**Q:** How should rolling window buffers work?  
**A:** 
- Fixed-size circular buffers
- Use partial data when buffer not full (divide by N where N = available data)
- Pre-allocated for efficiency

**Decision Rationale:** Matches existing volatility calculation behavior, memory-efficient

### 10. Simulation/Replay Integration
**Q:** How does replay system integrate?  
**A:** 
- `push_data()` accepts timestamp parameter
- Engine respects historical order
- Updates visible to naked eye (human-readable speed, not instant)

**Decision Rationale:** Enables visualization and debugging of analytics during replay

### 11. Node Output Caching
**Q:** What output should nodes store?  
**A:** 
- Store full history for now
- Future: Async writes to datastore

**Decision Rationale:** Simplifies initial implementation, prepares for future persistence

### 12. DAG Initialization
**Q:** How to handle cold start?  
**A:** Bootstrap by querying historical DataProvider to warm up buffers and initial state

**Decision Rationale:** Ensures analytics have necessary historical context from start

### 13. Notification Timing
**Q:** When should callbacks be invoked?  
**A:** 
- After each individual node updates
- Nodes execute asynchronously
- Data points should move visualization incrementally

**Decision Rationale:** Fine-grained updates for responsive UI, async execution for performance

---

## Summary of Key Design Decisions

### Core Architecture
- **Explicit Push API:** `engine.push_data(asset, timestamp, value)`
- **Node-Local State:** Each node manages own state, buffers, and last timestamp
- **Synchronous Propagation:** Updates cascade immediately through DAG
- **Sequential Multi-Asset:** Process assets one at a time in arrival order

### State & Memory Management
- **Fixed-Size Circular Buffers:** Pre-allocated for rolling windows
- **Full History Storage:** Nodes keep complete output history (for now)
- **Partial Buffer Handling:** Use available data when buffer not full

### Initialization & Warmup
- **Historical Bootstrap:** Query DataProvider to populate initial buffers
- **Two-Phase Operation:**
  1. Warmup phase: Load historical data
  2. Update phase: Process incremental data

### Error Handling & Resilience
- **Skip-on-Failure:** Failed nodes don't block other branches
- **Out-of-Order Rejection:** Ignore data with timestamp < last_computed
- **Graceful Degradation:** System continues with partial failures

### Notification & Observability
- **Per-Node Callbacks:** Register callbacks for specific analytics
- **Immediate Notification:** Callbacks triggered after each node update
- **Async Execution:** Nodes can update concurrently within constraints

---

## Existing Code to Reference

### From DAG Framework
- ✅ `AnalyticsDag` - DAG structure and topological sorting
- ✅ `Node` with `NodeParams` and assets
- ✅ `execute()` method - batch execution
- ✅ `execute_incremental()` placeholder - needs implementation
- ✅ `get_descendants()` - for dependency propagation

### From Analytics Module
- ✅ `calculate_returns()` - stateless function
- ✅ `calculate_volatility()` - rolling window calculation
- ✅ `NodeOutput` enum - output types
- ✅ Query builders - DAG construction

### From Data Layer
- ✅ `DataProvider` trait - for historical queries
- ✅ `TimeSeriesPoint` - data structure
- ✅ `DateRange` - time ranges

---

## Out of Scope (For This Iteration)

1. Out-of-order data handling (timestamps < last_computed)
2. Async datastore writes (nodes store history in-memory only)
3. Parallel multi-asset processing
4. Retry mechanisms for failed nodes
5. Backpressure handling for rapid data streams
6. Distributed/multi-node execution
7. Persistent state recovery after restart
8. Complex event patterns (time windows, aggregations)
9. Real-time data ingestion (separate feature)
10. Performance optimizations (SIMD, GPU)

---

## Visual Assets
None provided.

---

## Additional Context

This push-mode engine is the foundation for:
- **Item 7:** High-Speed Data Replay System (will use `push_data()` API)
- **Item 8:** REST API with WebSocket (will subscribe to callbacks)
- **Item 9:** React UI Dashboard (will visualize updates)

The engine must support **simulation** scenarios where historical data is replayed at controlled speeds for visualization and testing.

---

**Date Created:** 2025-11-25  
**Status:** Requirements Gathered

