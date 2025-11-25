# Requirements: Analytics Push-Mode Integration

## Goal

Wire existing analytics (returns, volatility) into the push-mode engine so they actually execute when data arrives.

## Current State

**We have all the pieces, but they're not connected:**

```
[Push-Mode Engine]   [Analytics Functions]   [Node Executors]
     push_data()      calculate_returns()    execute_returns_node()
         ↓                    ↓                       ↓
    (propagation        (stateless           (wrapper that
     framework)          functions)           calls functions)
         
         ❌ NOT CONNECTED ❌
```

**Need to connect them:**

```
push_data(AAPL, 150.0)
    ↓
Identify affected nodes (data_provider for AAPL)
    ↓
Execute in topological order:
    1. DataProvider: Store 150.0
    2. Returns: calculate_returns([previous, 150.0])
    3. Volatility: calculate_volatility(returns, window=30)
    ↓
Invoke callbacks with results
```

## Key Requirements

### 1. Node Type Mapping
Map node.node_type to executor functions:
- `"data_provider"` → Store incoming value
- `"returns"` → Call `execute_returns_node()`  
- `"volatility"` → Call `execute_volatility_node()`

### 2. DataProvider Node Behavior
When data arrives for an asset:
- Store value in node's output_history
- Push value to node's input_buffer (if it has one)
- Create TimeSeriesPoint(timestamp, value)

### 3. Data Flow Between Nodes
Pass outputs from parent nodes as inputs to children:
- Get parent node outputs from their output_history
- Convert to format expected by child (Vec<TimeSeriesPoint>)
- Pass as input to execute_XXX_node()

### 4. Buffer Population
For rolling window analytics:
- When returns node executes, push result to its buffer
- Volatility node reads from returns node's buffer for window
- Use CircularBuffer for efficient rolling windows

### 5. First Data Point Handling
- First price: DataProvider stores it, Returns outputs NaN
- Second price: Returns can calculate ln(P2/P1)
- After N prices: Volatility has N returns for window

### 6. Error Handling
- If node execution fails, set state to Failed
- Skip dependents of failed nodes
- Continue with other branches
- Log errors

### 7. Callback Invocation
- After each node executes successfully
- Pass NodeOutput::Single(Vec<TimeSeriesPoint>)
- Catch and log callback errors

## Integration Points

### Existing Code to Use

**From analytics.rs:**
- `execute_returns_node(node, inputs) -> Result<NodeOutput, DagError>`
- `execute_volatility_node(node, inputs) -> Result<NodeOutput, DagError>`
- `timeseries_to_prices()` and `prices_to_timeseries()` helpers

**From push_mode.rs:**
- `node_states: HashMap<NodeId, NodePushState>`
- `invoke_callbacks(node_id, output)`
- Propagation loop in `push_data()`

**From dag.rs:**
- `get_parents(node_id) -> Vec<NodeId>`
- `execution_order_immutable() -> Vec<NodeId>`
- Node struct with node_type, params, assets

## Success Criteria

✅ Can push AAPL price and get returns calculated
✅ Can push multiple prices and get volatility calculated
✅ Callbacks fire with real analytics values
✅ Buffers populate correctly for rolling windows
✅ End-to-end test: push 31 prices → get 30-day volatility
✅ Ready for replay system to feed historical data

## Out of Scope

- Moving averages (can add later)
- Pull-mode execution (separate Item 10)
- Performance optimizations
- Async execution
- Distributed execution

---

**Date:** 2025-11-25
**Status:** Requirements Complete

