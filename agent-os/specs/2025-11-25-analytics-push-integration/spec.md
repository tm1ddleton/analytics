# Specification: Analytics Push-Mode Integration

## Goal
Wire existing analytics functions (returns, volatility) into the push-mode engine so they actually execute when data arrives, enabling end-to-end working push-mode analytics.

## User Stories
- As a developer, I want to push a price and see returns calculated automatically
- As a trader, I want to push historical prices and see volatility update incrementally
- As a system integrator, I want the replay system to work by just calling push_data()
- As a UI developer, I want callbacks to fire with real analytics values

## Specific Requirements

### 1. Node Executor Dispatch
- Map node.node_type string to executor function
- Support types: "data_provider", "returns", "volatility"
- Call appropriate executor during propagation
- Handle unknown node types gracefully (log warning, skip)

### 2. DataProvider Node Execution
- When push_data(asset, timestamp, value) called
- Find DataProvider nodes for that asset
- Store value as TimeSeriesPoint in output_history
- Push value to input_buffer if buffer exists
- Set last_computed_timestamp to timestamp
- Set state to Ready

### 3. Analytics Node Execution
- Get inputs from parent nodes' output_history
- Pass inputs to execute_XXX_node() functions
- Store result in node's output_history
- Update last_computed_timestamp
- Invoke callbacks with result

### 4. Data Flow Implementation
```rust
// In propagation loop for each node:
fn execute_node(node_id: NodeId) -> Result<NodeOutput, PushError> {
    let node = get_node(node_id);
    
    match node.node_type.as_str() {
        "data_provider" => {
            // Store incoming value
            create_timeseries_point(timestamp, value)
        }
        "returns" => {
            // Get parent prices
            let prices = get_parent_outputs(node_id);
            execute_returns_node(&node, &[prices])
        }
        "volatility" => {
            // Get parent returns
            let returns = get_parent_outputs(node_id);
            execute_volatility_node(&node, &[returns])
        }
        _ => Err("Unknown node type")
    }
}
```

### 5. Buffer Management During Execution
- Returns node: Push each return value to its buffer
- Volatility node: Read from returns buffer (or output_history)
- Use get_buffer_slice() to get window for analytics
- Buffers automatically handle wraparound

### 6. Parent Output Retrieval
- Implement `get_parent_outputs(node_id) -> Vec<NodeOutput>`
- Use dag.get_parents(node_id) to find parents
- Get output_history from each parent's NodePushState
- Convert to NodeOutput::Single(Vec<TimeSeriesPoint>)
- Handle case where parent has no outputs yet (first iteration)

### 7. Error Handling in Execution
- Wrap node execution in Result
- On error:
  - Set node state to Failed(error_msg)
  - Log error with node details
  - Skip node's dependents in this propagation
  - Continue with other branches
- On success:
  - Set state to Ready
  - Append output to history
  - Invoke callbacks

### 8. Callback Integration
- After successful node execution
- Pass real NodeOutput to callbacks
- Example: NodeOutput::Single(vec![TimeSeriesPoint { timestamp, value }])
- Callbacks can access latest analytics values

### 9. Initialization Integration
- DataProvider nodes start in Uninitialized state
- After first data point, transition to Ready
- Analytics nodes wait for parent data
- Graceful handling of insufficient data for windows

### 10. End-to-End Workflow
```rust
// User code:
let mut engine = PushModeEngine::new(dag);
engine.initialize(&provider, end_date, 30)?;

// Register callback
engine.register_callback(volatility_node_id, Box::new(|output| {
    println!("Volatility updated: {:?}", output);
}))?;

// Push data
for (ts, price) in historical_prices {
    engine.push_data(asset.clone(), ts, price)?;
    // → DataProvider stores price
    // → Returns calculates log return
    // → Volatility updates rolling window
    // → Callback fires with new volatility value
}
```

## Implementation Details

### Modified push_data() Propagation Loop
```rust
// In PushModeEngine::push_data(), after getting sorted_affected nodes:
for node_id in sorted_affected {
    // Set state to Computing
    if let Some(state) = self.node_states.get_mut(&node_id) {
        state.set_state(NodeState::Computing);
    }
    
    // Execute node based on type
    let result = self.execute_node(node_id, asset.clone(), timestamp, value);
    
    match result {
        Ok(output) => {
            // Store output
            if let Some(state) = self.node_states.get_mut(&node_id) {
                // Extract TimeSeriesPoint from NodeOutput
                if let NodeOutput::Single(points) = &output {
                    for point in points {
                        state.append_output(point.clone());
                    }
                }
                state.set_state(NodeState::Ready);
            }
            
            // Invoke callbacks
            self.invoke_callbacks(node_id, &output);
        }
        Err(e) => {
            // Handle error
            if let Some(state) = self.node_states.get_mut(&node_id) {
                state.set_state(NodeState::Failed(e.to_string()));
            }
            // Continue with other nodes
        }
    }
}
```

### New execute_node() Method
```rust
fn execute_node(
    &self,
    node_id: NodeId,
    asset: AssetKey,
    timestamp: DateTime<Utc>,
    value: f64,
) -> Result<NodeOutput, PushError> {
    // Get node from DAG
    let node = self.get_node_from_dag(node_id)?;
    
    match node.node_type.as_str() {
        "data_provider" => {
            // Create TimeSeriesPoint directly
            let point = TimeSeriesPoint::new(timestamp, value);
            Ok(NodeOutput::Single(vec![point]))
        }
        "returns" => {
            // Get parent outputs
            let inputs = self.get_parent_outputs(node_id)?;
            execute_returns_node(&node, &inputs)
                .map_err(|e| PushError::PropagationFailed { node_id, error: e.to_string() })
        }
        "volatility" => {
            // Get parent outputs
            let inputs = self.get_parent_outputs(node_id)?;
            execute_volatility_node(&node, &inputs)
                .map_err(|e| PushError::PropagationFailed { node_id, error: e.to_string() })
        }
        _ => {
            Err(PushError::PropagationFailed {
                node_id,
                error: format!("Unknown node type: {}", node.node_type),
            })
        }
    }
}
```

## Testing Strategy

### Unit Tests
- Test execute_node() for each node type
- Test get_parent_outputs() with various scenarios
- Test error handling in node execution
- Test state transitions during execution

### Integration Tests
- Test DataProvider → Returns chain
- Test DataProvider → Returns → Volatility chain
- Test with InMemoryDataProvider
- Test callback invocation with real data
- Test 31 prices → 30-day volatility end-to-end

### Example Test
```rust
#[test]
fn test_end_to_end_returns_calculation() {
    let mut dag = AnalyticsDag::new();
    let asset = AssetKey::new_equity("AAPL").unwrap();
    
    // Add nodes
    let data_node = dag.add_node("data_provider".to_string(), ...);
    let returns_node = dag.add_node("returns".to_string(), ...);
    dag.add_edge(data_node, returns_node).unwrap();
    
    // Create engine
    let mut engine = PushModeEngine::new(dag);
    engine.is_initialized = true;
    
    // Initialize node states
    engine.node_states.insert(data_node, NodePushState::new(None));
    engine.node_states.insert(returns_node, NodePushState::new(None));
    
    // Push first price
    engine.push_data(asset.clone(), Utc::now(), 100.0).unwrap();
    
    // Push second price
    let ts2 = Utc::now() + Duration::seconds(1);
    engine.push_data(asset, ts2, 105.0).unwrap();
    
    // Check returns node has output
    let history = engine.get_history(returns_node).unwrap();
    assert_eq!(history.len(), 2);
    assert!(history[0].close_price.is_nan()); // First return is NaN
    assert!(history[1].close_price > 0.0); // Positive return
}
```

## Success Criteria

- ✅ Can push prices and see returns calculated
- ✅ Can push 31 prices and see 30-day volatility
- ✅ Callbacks fire with real NodeOutput values
- ✅ Buffers populate correctly
- ✅ Error handling works (failed nodes don't halt system)
- ✅ All tests pass (estimated 10-15 new integration tests)
- ✅ Ready for Item 7 (Replay System) to use

## Out of Scope

- Moving averages (can add after)
- Pull-mode execution (Item 10)
- Multiple data providers per node
- Async node execution
- Performance optimizations

---

**Version:** 1.0  
**Date:** 2025-11-25  
**Status:** Ready for Implementation

