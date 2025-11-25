# Task Breakdown: Analytics Push-Mode Integration

## Overview
Total Tasks: 4 task groups

This completes Roadmap Item 6: Basic Analytics Library that works in push mode.

## Task List

### Core Integration

#### Task Group 1: Node Executor Dispatch
**Dependencies:** None

- [x] 1.0 Complete node executor dispatch
  - [ ] 1.1 Write 3-5 tests for node execution
    - Test execute_node() for data_provider type
    - Test execute_node() for returns type
    - Test execute_node() for volatility type
    - Test unknown node type handling
  - [ ] 1.2 Implement execute_node() method in PushModeEngine
    - Match on node.node_type string
    - Handle "data_provider", "returns", "volatility"
    - Return Result<NodeOutput, PushError>
  - [ ] 1.3 Implement get_node_from_dag() helper
    - Access DAG to get Node by NodeId
    - Return error if node not found
  - [ ] 1.4 Ensure executor dispatch tests pass

**Acceptance Criteria:**
- execute_node() dispatches to correct handler
- All node types supported
- Tests pass

---

#### Task Group 2: Parent Output Retrieval
**Dependencies:** None

- [x] 2.0 Complete parent output retrieval
  - [ ] 2.1 Write 3-5 tests for parent retrieval
    - Test get_parent_outputs() with single parent
    - Test get_parent_outputs() with multiple parents
    - Test get_parent_outputs() with no outputs yet
    - Test get_parent_outputs() with no parents
  - [ ] 2.2 Implement get_parent_outputs() method
    - Use dag.get_parents(node_id)
    - Get output_history from each parent's NodePushState
    - Convert to Vec<NodeOutput>
    - Handle empty history gracefully
  - [ ] 2.3 Integrate get_parents() from DAG
    - Use existing get_parents() or implement if missing
  - [ ] 2.4 Ensure parent retrieval tests pass

**Acceptance Criteria:**
- Can retrieve parent outputs
- Handles edge cases (no parents, no outputs)
- Tests pass

---

#### Task Group 3: Integration with Propagation Loop
**Dependencies:** Task Groups 1, 2

- [x] 3.0 Complete propagation loop integration
  - [ ] 3.1 Write 5-8 integration tests
    - Test DataProvider → Returns chain
    - Test DataProvider → Returns → Volatility chain
    - Test with multiple data points
    - Test first data point (NaN returns)
    - Test 31 prices → 30-day volatility
    - Test callback invocation with real data
    - Test error handling (failed node)
  - [ ] 3.2 Modify push_data() propagation loop
    - Set node state to Computing
    - Call execute_node()
    - Store output in node_states
    - Update last_computed_timestamp
    - Set state to Ready or Failed
    - Invoke callbacks with output
  - [ ] 3.3 Initialize node states during engine creation
    - Create NodePushState for each node in DAG
    - Extract buffer size from NodeParams if needed
    - Set initial state to Uninitialized
  - [ ] 3.4 Implement find_nodes_with_asset() properly
    - Traverse DAG to find nodes with matching asset
    - Use node.assets field
    - Return Vec<NodeId>
  - [ ] 3.5 Ensure integration tests pass

**Acceptance Criteria:**
- Full end-to-end execution works
- Data flows through DAG
- Callbacks fire with real values
- All integration tests pass

---

#### Task Group 4: Examples and Documentation
**Dependencies:** Task Groups 1-3

- [x] 4.0 Complete examples and documentation
  - [ ] 4.1 Create examples/push_mode_volatility.rs
    - Build DAG with data → returns → volatility
    - Initialize engine
    - Push 31 prices
    - Register callback to print volatility
    - Show incremental updates
  - [ ] 4.2 Create examples/returns_calculation.rs
    - Simpler example with just returns
    - Push prices and show log returns
  - [ ] 4.3 Update module documentation
    - Add examples to push_mode.rs module docs
    - Document node types and execution
    - Show end-to-end workflow
  - [ ] 4.4 Run all tests to verify
    - Run complete test suite
    - Verify no regressions
    - Verify all new tests pass

**Acceptance Criteria:**
- Examples compile and run
- Examples demonstrate push-mode analytics
- Documentation complete
- All tests pass

---

## Implementation Notes

### Key Methods to Add

```rust
// In PushModeEngine:

fn execute_node(
    &self,
    node_id: NodeId,
    asset: AssetKey,
    timestamp: DateTime<Utc>,
    value: f64,
) -> Result<NodeOutput, PushError>

fn get_parent_outputs(&self, node_id: NodeId) -> Result<Vec<NodeOutput>, PushError>

fn get_node_from_dag(&self, node_id: NodeId) -> Result<&Node, PushError>

fn find_nodes_with_asset(&self, asset: &AssetKey) -> Vec<NodeId>  // Fix existing stub

fn initialize_node_states(&mut self)  // Call in new() or initialize()
```

### Modified Methods

```rust
// Update push_data() to actually execute nodes in propagation loop
pub fn push_data(&mut self, asset: AssetKey, timestamp: DateTime<Utc>, value: f64) -> Result<(), PushError>
```

### Testing Strategy
- Unit tests for each new method
- Integration tests for full chains
- End-to-end test with 31 prices → volatility
- Callback tests with real data

---

## Success Criteria Summary

- ✅ Node execution dispatch works
- ✅ Parent outputs can be retrieved
- ✅ Full propagation executes analytics
- ✅ DataProvider → Returns → Volatility chain works
- ✅ Callbacks fire with real values
- ✅ Examples demonstrate working system
- ✅ All tests pass (~15 new tests)
- ✅ Ready for Item 7 (Replay System)

---

**Total Task Groups:** 4  
**Estimated New Tests:** ~15  
**Feature Size:** M (Medium)  
**Date Created:** 2025-11-25

