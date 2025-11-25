# Task Breakdown: Push-Mode Analytics Engine

## Overview
Total Tasks: 9 task groups

This is an XL (Extra Large) feature that implements incremental computation where analytics automatically update when new data arrives.

## Task List

### Foundation: Data Structures

#### Task Group 1: Circular Buffer Implementation
**Dependencies:** None

- [x] 1.0 Complete circular buffer implementation
  - [x] 1.1 Write 2-8 focused tests for circular buffer
    - Test creation with fixed capacity
    - Test push with wraparound behavior
    - Test partial fills (len < capacity)
    - Test get_slice() returns correct view
    - Test is_full() correctly reports state
  - [x] 1.2 Implement CircularBuffer struct
    - Create `CircularBuffer<T>` generic struct
    - Fixed capacity, pre-allocated storage
    - Use `VecDeque` or custom implementation
    - Track head, tail, and current length
  - [x] 1.3 Implement core buffer operations
    - `new(capacity: usize) -> Self` - create buffer
    - `push(&mut self, value: T)` - add value, overwrite oldest if full
    - `get_slice(&self) -> &[T]` - view of current contents
    - `len(&self) -> usize` - current fill level
    - `is_full(&self) -> bool` - check if at capacity
    - `capacity(&self) -> usize` - max capacity
  - [x] 1.4 Ensure circular buffer tests pass
    - Run ONLY the 2-8 tests written in 1.1
    - Verify O(1) push operations
    - Verify wraparound works correctly
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 1.1 pass
- CircularBuffer supports fixed capacity with efficient operations
- Partial fills handled correctly
- Wraparound behavior works

---

### Core State Management

#### Task Group 2: Node State Extension
**Dependencies:** Task Group 1

- [x] 2.0 Complete node state extension
  - [x] 2.1 Write 2-8 focused tests for node state
    - Test NodeState enum transitions (Uninitialized → Ready → Computing → Ready)
    - Test last_computed_timestamp tracking
    - Test output_history append operations
    - Test input_buffer integration with CircularBuffer
    - Test state per node is independent
  - [x] 2.2 Define NodeState enum
    - Create enum with variants: Uninitialized, Ready, Computing, Failed(String)
    - Implement Display trait for error messages
    - Add state transition validation
  - [x] 2.3 Extend Node struct with stateful fields
    - Add `last_computed_timestamp: Option<DateTime<Utc>>`
    - Add `output_history: Vec<TimeSeriesPoint>`
    - Add `input_buffer: Option<CircularBuffer<f64>>`
    - Add `state: NodeState`
    - Initialize in Node::new() or separate method
  - [x] 2.4 Implement buffer initialization
    - Extract window_size from NodeParams (if present)
    - Create CircularBuffer with appropriate capacity
    - Handle nodes without buffers (e.g., simple transformations)
  - [x] 2.5 Implement state query methods
    - `get_state(&self) -> &NodeState`
    - `set_state(&mut self, state: NodeState)`
    - `append_output(&mut self, point: TimeSeriesPoint)`
    - `get_last_timestamp(&self) -> Option<DateTime<Utc>>`
  - [x] 2.6 Ensure node state tests pass
    - Run ONLY the 2-8 tests written in 2.1
    - Verify state transitions work
    - Verify buffers are properly initialized
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 2.1 pass
- Node struct extended with state fields
- NodeState enum properly tracks node lifecycle
- Buffers initialized based on node parameters

---

### Engine Core

#### Task Group 3: PushModeEngine Structure and Push Data API
**Dependencies:** Task Group 2

- [x] 3.0 Complete PushModeEngine structure and push_data API
  - [ ] 3.1 Write 2-8 focused tests for PushModeEngine
    - Test engine creation with DAG
    - Test push_data() with single node
    - Test timestamp validation (reject out-of-order)
    - Test invalid data rejection (NaN, negative prices)
    - Test PushError variants
  - [ ] 3.2 Define error types
    - Create `PushError` enum: OutOfOrder, InvalidData, PropagationFailed, EngineNotInitialized
    - Create `InitError` enum: DataProviderError, InsufficientHistoricalData, NodeInitializationFailed
    - Implement Display and Error traits
  - [ ] 3.3 Implement PushModeEngine struct
    - Store `dag: AnalyticsDag`
    - Store `callbacks: HashMap<NodeId, Vec<Callback>>` where `Callback = Box<dyn Fn(&NodeOutput)>`
    - Store `is_initialized: bool`
    - Implement `new(dag: AnalyticsDag) -> Self`
  - [ ] 3.4 Implement push_data validation
    - Validate timestamp is not NaN
    - Validate value is not NaN or infinite
    - Validate engine is initialized
    - Return appropriate PushError on failure
  - [ ] 3.5 Implement basic push_data scaffolding
    - Signature: `push_data(&mut self, asset: AssetKey, timestamp: DateTime<Utc>, value: f64) -> Result<(), PushError>`
    - Identify affected nodes (nodes with this asset)
    - Validate timestamp > node.last_computed_timestamp
    - Placeholder for propagation (to be implemented in Task Group 4)
  - [ ] 3.6 Ensure PushModeEngine tests pass
    - Run ONLY the 2-8 tests written in 3.1
    - Verify engine creation works
    - Verify validation logic works
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 3.1 pass
- PushModeEngine struct created with necessary fields
- push_data() API defined with validation
- Error types properly defined

---

#### Task Group 4: Synchronous Propagation Implementation
**Dependencies:** Task Group 3

- [x] 4.0 Complete synchronous propagation implementation
  - [ ] 4.1 Write 2-8 focused tests for propagation
    - Test simple chain: data → returns → volatility
    - Test propagation respects topological order
    - Test only affected nodes are updated
    - Test parallel branches propagate correctly
    - Test propagation stops at failed nodes
  - [ ] 4.2 Implement affected node identification
    - Find nodes with matching AssetKey
    - Use existing node.assets field
    - Return Vec<NodeId> of affected nodes
  - [ ] 4.3 Implement topological propagation
    - Get topological sort of affected subgraph
    - Use existing `topological_sort()` method
    - Filter to only affected nodes and their descendants
    - Process nodes in correct order
  - [ ] 4.4 Implement node update logic
    - For each node in topological order:
      1. Check node.state is Ready
      2. Set state to Computing
      3. Get inputs from parent nodes
      4. Call node-specific execution function
      5. Append output to output_history
      6. Update last_computed_timestamp
      7. Set state to Ready (or Failed on error)
  - [ ] 4.5 Integrate with existing node executors
    - Use `execute_returns_node()` for returns nodes
    - Use `execute_volatility_node()` for volatility nodes
    - Match on node.node_type to dispatch
    - Pass inputs from parent nodes
  - [ ] 4.6 Implement get_descendants helper
    - Use existing `get_descendants()` method if available
    - Or implement graph traversal from affected nodes
    - Return full subgraph that needs updating
  - [ ] 4.7 Ensure propagation tests pass
    - Run ONLY the 2-8 tests written in 4.1
    - Verify correct execution order
    - Verify only affected nodes updated
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 4.1 pass
- Propagation follows topological order
- Only affected nodes and their descendants are updated
- Node state properly tracked during propagation

---

### Notification System

#### Task Group 5: Callback Registration and Notification
**Dependencies:** Task Group 4

- [x] 5.0 Complete callback system
  - [ ] 5.1 Write 2-8 focused tests for callbacks
    - Test registering single callback
    - Test multiple callbacks for same node
    - Test callbacks invoked after node update
    - Test callback receives correct NodeOutput
    - Test callback errors don't halt propagation
  - [ ] 5.2 Implement callback registration
    - Signature: `register_callback(&mut self, node_id: NodeId, callback: Box<dyn Fn(&NodeOutput)>)`
    - Store callbacks in HashMap<NodeId, Vec<Callback>>
    - Support multiple callbacks per node
    - Return error if node doesn't exist
  - [ ] 5.3 Implement callback invocation
    - After each node computes, look up callbacks
    - Invoke all callbacks with NodeOutput
    - Catch and log callback errors (use log crate)
    - Don't propagate callback errors to caller
  - [ ] 5.4 Implement callback error handling
    - Wrap callback invocation in try/catch
    - Log callback errors with node_id and details
    - Continue with remaining callbacks on error
    - Continue propagation after callback errors
  - [ ] 5.5 Ensure callback tests pass
    - Run ONLY the 2-8 tests written in 5.1
    - Verify callbacks are invoked
    - Verify error resilience
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 5.1 pass
- Callbacks can be registered per node
- Multiple callbacks per node supported
- Callbacks invoked after each node update
- Callback errors logged but don't halt system

---

### Initialization System

#### Task Group 6: Historical Initialization (Warmup)
**Dependencies:** Task Groups 2, 4

- [x] 6.0 Complete historical initialization
  - [ ] 6.1 Write 2-8 focused tests for initialization
    - Test initialize() with DataProvider
    - Test buffers populated from historical data
    - Test initial states computed for all nodes
    - Test last_computed_timestamp set correctly
    - Test insufficient historical data error
  - [ ] 6.2 Implement lookback calculation
    - Traverse DAG to find max window_size
    - Return required lookback_days
    - Account for dependency chain (e.g., volatility needs returns needs prices)
    - Method: `calculate_required_lookback(&self) -> usize`
  - [ ] 6.3 Implement initialize() method
    - Signature: `initialize(&mut self, data_provider: &dyn DataProvider, end_date: DateTime<Utc>, lookback_days: usize) -> Result<(), InitError>`
    - Query DataProvider for historical data
    - Populate node buffers with historical values
    - Compute initial states using batch execution
    - Set last_computed_timestamp to end_date
    - Set is_initialized = true
  - [ ] 6.4 Implement buffer population
    - For each node with input_buffer:
      - Query historical data from DataProvider
      - Push values into CircularBuffer
      - Handle partial data (buffer may not fill completely)
  - [ ] 6.5 Implement initial state computation
    - Use existing execute() method for batch processing
    - Compute all nodes with historical data
    - Populate output_history with results
    - Set NodeState to Ready for all successful nodes
  - [ ] 6.6 Ensure initialization tests pass
    - Run ONLY the 2-8 tests written in 6.1
    - Verify buffers are populated
    - Verify initial states computed
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 6.1 pass
- initialize() bootstraps from DataProvider
- Buffers populated with historical data
- Initial states computed for all nodes
- Engine ready for incremental updates after warmup

---

### Error Handling & Resilience

#### Task Group 7: Error Handling and Resilience
**Dependencies:** Task Groups 4, 5

- [x] 7.0 Complete error handling and resilience
  - [ ] 7.1 Write 2-8 focused tests for error handling
    - Test node computation error doesn't halt propagation
    - Test failed node's dependents are skipped
    - Test parallel branches continue after one fails
    - Test node recovery on next update
    - Test error logging
  - [ ] 7.2 Implement node error handling
    - Wrap node execution in try/catch
    - On error: log, set NodeState::Failed(error_msg)
    - Continue to next node in propagation
    - Don't update failed node's output_history
  - [ ] 7.3 Implement dependent skipping
    - When node is Failed, mark descendants as skipped
    - Don't execute nodes whose parents failed
    - Allow other branches to continue
    - Track skipped nodes for logging/debugging
  - [ ] 7.4 Implement node recovery mechanism
    - On next push_data(), reset Failed nodes to Ready
    - Attempt re-execution
    - If succeeds, node recovers
    - If fails again, remain in Failed state
  - [ ] 7.5 Implement comprehensive logging
    - Log all errors with node_id and details
    - Log propagation start/end
    - Log skipped nodes
    - Use log crate with appropriate levels (error, warn, info)
  - [ ] 7.6 Ensure error handling tests pass
    - Run ONLY the 2-8 tests written in 7.1
    - Verify resilience to failures
    - Verify correct error behavior
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 7.1 pass
- Node failures don't halt entire propagation
- Failed node's dependents are skipped
- Parallel branches continue after failures
- Nodes can recover on subsequent updates

---

### Query APIs

#### Task Group 8: Output History and Query APIs
**Dependencies:** Task Group 2

- [x] 8.0 Complete output history and query APIs
  - [ ] 8.1 Write 2-8 focused tests for query APIs
    - Test get_history() returns all outputs
    - Test get_latest() returns most recent output
    - Test get_history() with no outputs returns empty
    - Test get_latest() with no outputs returns None
    - Test query non-existent node returns error
  - [ ] 8.2 Implement get_history()
    - Signature: `get_history(&self, node_id: NodeId) -> Result<Vec<TimeSeriesPoint>, PushError>`
    - Look up node by NodeId
    - Return clone of output_history
    - Return error if node not found
  - [ ] 8.3 Implement get_latest()
    - Signature: `get_latest(&self, node_id: NodeId) -> Result<Option<TimeSeriesPoint>, PushError>`
    - Look up node by NodeId
    - Return last element of output_history
    - Return None if output_history is empty
    - Return error if node not found
  - [ ] 8.4 Implement helper query methods
    - `get_node_state(&self, node_id: NodeId) -> Result<&NodeState, PushError>`
    - `get_buffer_contents(&self, node_id: NodeId) -> Result<&[f64], PushError>`
    - `is_initialized(&self) -> bool`
  - [ ] 8.5 Ensure query API tests pass
    - Run ONLY the 2-8 tests written in 8.1
    - Verify APIs return correct data
    - Verify error handling
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 8.1 pass
- get_history() returns complete output history
- get_latest() returns most recent output
- Query APIs handle non-existent nodes gracefully

---

### Integration & Testing

#### Task Group 9: End-to-End Integration Testing
**Dependencies:** Task Groups 1-8

- [x] 9.0 Complete integration testing
  - [ ] 9.1 Write 5-10 integration tests
    - Test full workflow: initialize → push_data → query
    - Test with returns analytic
    - Test with volatility analytic (rolling window)
    - Test with complete chain: data → returns → volatility
    - Test multiple assets updating
    - Test callbacks notification flow
    - Test warmup with insufficient lookback
    - Test error recovery scenario
    - Test simulation replay scenario
  - [ ] 9.2 Create end-to-end simulation test
    - Use InMemoryDataProvider with sample data
    - Build DAG with returns and volatility nodes
    - Initialize engine with historical warmup
    - Replay data points one at a time
    - Register callbacks to track updates
    - Verify analytics update correctly at each step
  - [ ] 9.3 Create multi-asset test
    - Test AAPL and MSFT updating in sequence
    - Verify sequential processing
    - Verify independent analytics for each asset
  - [ ] 9.4 Performance validation
    - Test with realistic data volume (365 days)
    - Measure propagation time per update
    - Ensure acceptable performance (< 1ms per update for simple analytics)
  - [ ] 9.5 Add module to src/lib.rs
    - Create `src/push_mode.rs` module
    - Export PushModeEngine, errors, and types
    - Add to lib.rs with `pub mod push_mode;`
    - Export key types: `pub use push_mode::{PushModeEngine, PushError, InitError};`
  - [ ] 9.6 Run complete test suite
    - Run ALL tests (unit + integration)
    - Verify no regressions in existing code
    - Verify all new tests pass
    - Expected total: 202 existing + ~45 new = ~247 tests

**Acceptance Criteria:**
- All 5-10 integration tests pass
- Complete simulation scenario works end-to-end
- Multi-asset processing works correctly
- Performance is acceptable
- All 247+ tests pass with no regressions
- Module properly exported in lib.rs

---

## Implementation Notes

### Module Organization
```
src/
├── push_mode.rs          # New module for push-mode engine
│   ├── engine           # PushModeEngine struct
│   ├── circular_buffer  # CircularBuffer implementation
│   ├── errors           # PushError, InitError
│   └── tests            # Unit tests
```

### Key Data Structures

```rust
// Circular buffer for rolling windows
pub struct CircularBuffer<T> {
    data: VecDeque<T>,
    capacity: usize,
}

// Node state tracking
pub enum NodeState {
    Uninitialized,
    Ready,
    Computing,
    Failed(String),
}

// Push-mode engine
pub struct PushModeEngine {
    dag: AnalyticsDag,
    callbacks: HashMap<NodeId, Vec<Callback>>,
    is_initialized: bool,
}

type Callback = Box<dyn Fn(&NodeOutput) + Send + Sync>;
```

### Error Types

```rust
#[derive(Debug)]
pub enum PushError {
    OutOfOrder { timestamp: DateTime<Utc>, last_computed: DateTime<Utc> },
    InvalidData(String),
    PropagationFailed { node_id: NodeId, error: String },
    EngineNotInitialized,
    NodeNotFound(NodeId),
}

#[derive(Debug)]
pub enum InitError {
    DataProviderError(DataProviderError),
    InsufficientHistoricalData { required: usize, available: usize },
    NodeInitializationFailed { node_id: NodeId, error: String },
}
```

### Testing Strategy

- **Task Groups 1-8:** Each writes 2-8 focused tests, runs only those
- **Task Group 9:** Integration tests covering full workflows
- **Total Expected:** ~45 new tests (8 groups × ~5 tests + 10 integration tests)
- **Final Test Count:** ~247 tests (202 existing + 45 new)

---

## Success Criteria Summary

- ✅ All task groups complete with tests passing
- ✅ CircularBuffer efficient and correct
- ✅ Nodes maintain state and buffers
- ✅ push_data() validates and propagates
- ✅ Callbacks fire after each update
- ✅ initialize() warms up from DataProvider
- ✅ Error handling is resilient
- ✅ Query APIs work correctly
- ✅ End-to-end simulation works
- ✅ All 247+ tests pass
- ✅ No regressions in existing code

---

**Total Task Groups:** 9  
**Estimated New Tests:** ~45  
**Final Test Count:** ~247  
**Feature Size:** XL (Extra Large)  
**Date Created:** 2025-11-25

