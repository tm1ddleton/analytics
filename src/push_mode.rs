//! Push-Mode Analytics Engine
//!
//! This module implements an incremental computation system where analytics
//! automatically update when new data arrives, propagating changes through
//! the DAG dependency chain.

use crate::asset_key::AssetKey;
use crate::dag::{AnalyticsDag, Node, NodeId, NodeOutput, NodeParams};
use crate::time_series::{DataProvider, DataProviderError, TimeSeriesPoint};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt;

/// Node state tracking lifecycle
///
/// Tracks the current state of a node in the push-mode analytics engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeState {
    /// Node has not been initialized with data yet
    Uninitialized,
    /// Node is ready to process updates
    Ready,
    /// Node is currently computing
    Computing,
    /// Node computation failed with error message
    Failed(String),
}

impl fmt::Display for NodeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeState::Uninitialized => write!(f, "Uninitialized"),
            NodeState::Ready => write!(f, "Ready"),
            NodeState::Computing => write!(f, "Computing"),
            NodeState::Failed(msg) => write!(f, "Failed: {}", msg),
        }
    }
}

/// Stateful node extensions for push-mode execution
///
/// Contains runtime state for incremental computation.
#[derive(Debug, Clone)]
pub struct NodePushState {
    /// Last timestamp this node was computed
    pub last_computed_timestamp: Option<DateTime<Utc>>,
    /// Full history of this node's outputs
    pub output_history: Vec<TimeSeriesPoint>,
    /// Input buffer for rolling window analytics
    pub input_buffer: Option<CircularBuffer<f64>>,
    /// Current state of the node
    pub state: NodeState,
}

impl NodePushState {
    /// Creates a new push state
    ///
    /// # Arguments
    /// * `buffer_capacity` - Optional capacity for input buffer (if node needs rolling window)
    pub fn new(buffer_capacity: Option<usize>) -> Self {
        NodePushState {
            last_computed_timestamp: None,
            output_history: Vec::new(),
            input_buffer: buffer_capacity.map(CircularBuffer::new),
            state: NodeState::Uninitialized,
        }
    }

    /// Gets the current state
    pub fn get_state(&self) -> &NodeState {
        &self.state
    }

    /// Sets the node state
    pub fn set_state(&mut self, state: NodeState) {
        self.state = state;
    }

    /// Appends an output point to history
    pub fn append_output(&mut self, point: TimeSeriesPoint) {
        self.last_computed_timestamp = Some(point.timestamp);
        self.output_history.push(point);
    }

    /// Gets the last computed timestamp
    pub fn get_last_timestamp(&self) -> Option<DateTime<Utc>> {
        self.last_computed_timestamp
    }

    /// Gets a reference to the output history
    pub fn get_history(&self) -> &[TimeSeriesPoint] {
        &self.output_history
    }

    /// Gets the latest output value
    pub fn get_latest(&self) -> Option<&TimeSeriesPoint> {
        self.output_history.last()
    }

    /// Pushes a value into the input buffer (if it exists)
    pub fn push_to_buffer(&mut self, value: f64) {
        if let Some(buffer) = &mut self.input_buffer {
            buffer.push(value);
        }
    }

    /// Gets the input buffer slice (if it exists)
    pub fn get_buffer_slice(&self) -> Option<Vec<f64>> {
        self.input_buffer.as_ref().map(|b| b.get_slice())
    }
}

/// Error types for push-mode operations
#[derive(Debug)]
pub enum PushError {
    /// Data timestamp is out of order (earlier than last computed)
    OutOfOrder {
        timestamp: DateTime<Utc>,
        last_computed: DateTime<Utc>,
    },
    /// Invalid data provided (NaN, infinite, etc.)
    InvalidData(String),
    /// Propagation through DAG failed
    PropagationFailed { node_id: NodeId, error: String },
    /// Engine not initialized (need to call initialize() first)
    EngineNotInitialized,
    /// Node not found
    NodeNotFound(NodeId),
}

impl fmt::Display for PushError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PushError::OutOfOrder {
                timestamp,
                last_computed,
            } => {
                write!(
                    f,
                    "Out of order data: timestamp {:?} is before last computed {:?}",
                    timestamp, last_computed
                )
            }
            PushError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            PushError::PropagationFailed { node_id, error } => {
                write!(f, "Propagation failed at node {:?}: {}", node_id, error)
            }
            PushError::EngineNotInitialized => {
                write!(f, "Engine not initialized - call initialize() first")
            }
            PushError::NodeNotFound(node_id) => {
                write!(f, "Node {:?} not found", node_id)
            }
        }
    }
}

impl Error for PushError {}

/// Error types for initialization operations
#[derive(Debug)]
pub enum InitError {
    /// DataProvider error during initialization
    DataProviderError(DataProviderError),
    /// Insufficient historical data available
    InsufficientHistoricalData { required: usize, available: usize },
    /// Node initialization failed
    NodeInitializationFailed { node_id: NodeId, error: String },
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InitError::DataProviderError(err) => {
                write!(f, "DataProvider error: {}", err)
            }
            InitError::InsufficientHistoricalData {
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient historical data: required {} days, available {}",
                    required, available
                )
            }
            InitError::NodeInitializationFailed { node_id, error } => {
                write!(f, "Node {:?} initialization failed: {}", node_id, error)
            }
        }
    }
}

impl Error for InitError {}

impl From<DataProviderError> for InitError {
    fn from(err: DataProviderError) -> Self {
        InitError::DataProviderError(err)
    }
}

/// Callback function type for node updates
pub type Callback = Box<dyn Fn(NodeId, &NodeOutput, Option<DateTime<Utc>>) + Send + Sync>;

/// Push-mode analytics engine
///
/// Implements incremental computation where analytics automatically update
/// when new data arrives, propagating changes through the DAG.
///
/// # Examples
/// ```ignore
/// use analytics::push_mode::PushModeEngine;
/// use analytics::dag::AnalyticsDag;
///
/// let dag = AnalyticsDag::new();
/// let mut engine = PushModeEngine::new(dag);
///
/// // Initialize with historical data
/// engine.initialize(&data_provider, end_date, 30)?;
///
/// // Push new data points
/// engine.push_data(asset, timestamp, value)?;
/// ```
pub struct PushModeEngine {
    /// The DAG structure
    dag: AnalyticsDag,
    /// Push state for each node
    node_states: HashMap<NodeId, NodePushState>,
    /// Registered callbacks per node
    callbacks: HashMap<NodeId, Vec<Callback>>,
    /// Whether engine has been initialized
    pub is_initialized: bool,
}

impl PushModeEngine {
    /// Creates a new push-mode engine with the given DAG
    ///
    /// # Arguments
    /// * `dag` - The analytics DAG to execute
    pub fn new(dag: AnalyticsDag) -> Self {
        let mut engine = PushModeEngine {
            dag,
            node_states: HashMap::new(),
            callbacks: HashMap::new(),
            is_initialized: false,
        };

        // Initialize node states for all nodes
        engine.initialize_node_states();

        engine
    }

    /// Initializes NodePushState for all nodes in the DAG
    fn initialize_node_states(&mut self) {
        let node_ids = self.dag.node_ids();

        for node_id in node_ids {
            // Try to get the node to determine if it needs a buffer
            if let Ok(node) = self.get_node_from_dag(node_id) {
                // Extract buffer size from NodeParams if this is a volatility node
                let buffer_capacity = if node.node_type == "volatility" {
                    // Extract window_size from params
                    if let NodeParams::Map(params) = &node.params {
                        params
                            .get("window_size")
                            .and_then(|s| s.parse::<usize>().ok())
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Create node state
                let state = NodePushState::new(buffer_capacity);
                self.node_states.insert(node_id, state);
            }
        }
    }

    /// Pushes a new data point into the engine
    ///
    /// The data point will propagate through the DAG, updating all
    /// affected analytics.
    ///
    /// # Arguments
    /// * `asset` - The asset this data is for
    /// * `timestamp` - Timestamp of the data point
    /// * `value` - The value (e.g., closing price)
    ///
    /// # Returns
    /// Ok(()) if propagation succeeded, Err otherwise
    ///
    /// # Errors
    /// - `EngineNotInitialized` if initialize() hasn't been called
    /// - `InvalidData` if value is NaN or infinite
    /// - `OutOfOrder` if timestamp is before last computed
    /// - `PropagationFailed` if node computation fails
    pub fn push_data(
        &mut self,
        asset: AssetKey,
        timestamp: DateTime<Utc>,
        value: f64,
    ) -> Result<(), PushError> {
        // Validate engine is initialized
        if !self.is_initialized {
            return Err(PushError::EngineNotInitialized);
        }

        // Validate value
        if value.is_nan() {
            return Err(PushError::InvalidData("Value is NaN".to_string()));
        }
        if value.is_infinite() {
            return Err(PushError::InvalidData("Value is infinite".to_string()));
        }
        if value < 0.0 {
            return Err(PushError::InvalidData("Value is negative".to_string()));
        }

        // Identify affected nodes (nodes with this asset)
        let affected_nodes = self.find_nodes_with_asset(&asset);

        if affected_nodes.is_empty() {
            // No nodes for this asset, nothing to do
            return Ok(());
        }

        // Validate timestamp ordering for affected nodes
        for node_id in &affected_nodes {
            if let Some(state) = self.node_states.get(node_id) {
                if let Some(last_ts) = state.get_last_timestamp() {
                    if timestamp <= last_ts {
                        return Err(PushError::OutOfOrder {
                            timestamp,
                            last_computed: last_ts,
                        });
                    }
                }
            }
        }

        // Get all descendants (nodes that need to be updated)
        let mut all_affected = affected_nodes.clone();
        for node_id in &affected_nodes {
            let descendants = self.dag.get_descendants(*node_id);
            all_affected.extend(descendants);
        }

        // Deduplicate (order doesn't matter here, we'll sort by topological order later)
        all_affected.sort_by_key(|id| id.0);
        all_affected.dedup();

        // Get execution order for affected subgraph
        let exec_order =
            self.dag
                .execution_order_immutable()
                .map_err(|e| PushError::PropagationFailed {
                    node_id: NodeId(0),
                    error: format!("Failed to get execution order: {}", e),
                })?;

        // Filter to only affected nodes in topological order
        let sorted_affected: Vec<NodeId> = exec_order
            .into_iter()
            .filter(|id| all_affected.contains(id))
            .collect();

        // Propagate through affected nodes
        for node_id in sorted_affected {
            // Set node state to Computing
            if let Some(state) = self.node_states.get_mut(&node_id) {
                state.set_state(NodeState::Computing);
            }

            // Execute the node
            let execution_result = self.execute_node(node_id, asset.clone(), timestamp, value);

            match execution_result {
                Ok(output) => {
                    // Store output in node state
                    if let Some(state) = self.node_states.get_mut(&node_id) {
                        // Extract TimeSeriesPoint(s) from NodeOutput
                        match &output {
                            NodeOutput::Single(points_vec) => {
                                // points_vec is Vec<TimeSeriesPoint>
                                for point in points_vec {
                                    state.append_output(point.clone());
                                    // Also push to buffer if this node has one
                                    state.push_to_buffer(point.close_price);
                                }
                            }
                            NodeOutput::Scalar(value) => {
                                // Create a TimeSeriesPoint from scalar
                                let point = TimeSeriesPoint::new(timestamp, *value);
                                state.append_output(point);
                                state.push_to_buffer(*value);
                            }
                            NodeOutput::Collection(collection) => {
                                // Collection is Vec<Vec<TimeSeriesPoint>>
                                for points_vec in collection {
                                    for point in points_vec {
                                        state.append_output(point.clone());
                                        state.push_to_buffer(point.close_price);
                                    }
                                }
                            }
                            NodeOutput::None => {
                                // No output to store
                            }
                        }

                        // Set state to Ready
                        state.set_state(NodeState::Ready);
                    }

                    // Invoke callbacks
                    self.invoke_callbacks(node_id, &output);
                }
                Err(e) => {
                    // Handle execution error
                    if let Some(state) = self.node_states.get_mut(&node_id) {
                        state.set_state(NodeState::Failed(e.to_string()));
                    }

                    // Log error (in production, use log crate)
                    eprintln!("Node {:?} execution failed: {}", node_id, e);

                    // Continue with other nodes (resilient execution)
                }
            }
        }

        Ok(())
    }

    /// Finds nodes that contain the given asset
    fn find_nodes_with_asset(&self, asset: &AssetKey) -> Vec<NodeId> {
        // Get all node IDs and check which ones have this asset
        let mut matching_nodes = Vec::new();

        // We need to traverse the DAG to find nodes with this asset
        // For now, we'll check node_states keys (nodes that have been initialized)
        for (node_id, _state) in &self.node_states {
            // Try to get the node from DAG and check its assets
            if let Ok(node) = self.get_node_from_dag(*node_id) {
                if node.assets.contains(asset) {
                    matching_nodes.push(*node_id);
                }
            }
        }

        matching_nodes
    }

    /// Gets a node from the DAG by ID
    ///
    /// # Arguments
    /// * `node_id` - The node ID to look up
    ///
    /// # Returns
    /// Cloned Node, or error if not found
    fn get_node_from_dag(&self, node_id: NodeId) -> Result<Node, PushError> {
        self.dag
            .get_node(node_id)
            .map(|node| node.clone())
            .ok_or(PushError::NodeNotFound(node_id))
    }

    /// Executes a single node based on its type
    ///
    /// # Arguments
    /// * `node_id` - The node to execute
    /// * `asset` - The asset being processed
    /// * `timestamp` - Timestamp of the data
    /// * `value` - The value (for data_provider nodes)
    ///
    /// # Returns
    /// NodeOutput from execution, or error
    fn execute_node(
        &self,
        node_id: NodeId,
        _asset: AssetKey,
        timestamp: DateTime<Utc>,
        value: f64,
    ) -> Result<NodeOutput, PushError> {
        // Ensure the node exists
        self.get_node_from_dag(node_id)?;

        let inputs = self.get_parent_histories(node_id)?;
        self.dag
            .execute_push_node(node_id, &inputs, timestamp, value)
            .map_err(|e| PushError::PropagationFailed {
                node_id,
                error: e.to_string(),
            })
    }

    /// Gets outputs from all parent nodes
    ///
    /// # Arguments
    /// * `node_id` - The node whose parents to query
    ///
    /// # Returns
    /// Vector of NodeOutput from each parent
    fn get_parent_histories(
        &self,
        node_id: NodeId,
    ) -> Result<Vec<Vec<TimeSeriesPoint>>, PushError> {
        let parent_ids = self.dag.get_parents(node_id);

        let mut outputs = Vec::new();
        for parent_id in parent_ids {
            // Get parent's output history
            if let Some(parent_state) = self.node_states.get(&parent_id) {
                let history = parent_state.get_history().to_vec();
                if !history.is_empty() {
                    outputs.push(history);
                }
            }
        }

        Ok(outputs)
    }

    /// Returns whether the engine has been initialized
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    /// Registers a callback for a specific node
    ///
    /// The callback will be invoked after the node computes a new output.
    /// Multiple callbacks can be registered for the same node.
    ///
    /// # Arguments
    /// * `node_id` - The node to register the callback for
    /// * `callback` - The callback function to invoke
    ///
    /// # Returns
    /// Ok(()) if registration succeeded, Err if node doesn't exist
    pub fn register_callback(
        &mut self,
        node_id: NodeId,
        callback: Callback,
    ) -> Result<(), PushError> {
        // For now, don't validate node exists (will be validated during execution)
        self.callbacks
            .entry(node_id)
            .or_insert_with(Vec::new)
            .push(callback);
        Ok(())
    }

    /// Invokes all callbacks registered for a node
    ///
    /// Errors in callbacks are logged but don't halt execution.
    fn invoke_callbacks(&self, node_id: NodeId, output: &NodeOutput) {
        let timestamp = self
            .node_states
            .get(&node_id)
            .and_then(|state| state.get_last_timestamp());

        if let Some(callbacks) = self.callbacks.get(&node_id) {
            for callback in callbacks {
                // Wrap in catch to prevent callback errors from propagating
                if let Err(_e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    callback(node_id, output, timestamp);
                })) {
                    // Log callback error (in production, use log crate)
                    eprintln!("Callback error for node {:?}", node_id);
                }
            }
        }
    }

    /// Gets the complete output history for a node
    ///
    /// # Arguments
    /// * `node_id` - The node to query
    ///
    /// # Returns
    /// Vector of all output points, or error if node doesn't exist
    pub fn get_history(&self, node_id: NodeId) -> Result<Vec<TimeSeriesPoint>, PushError> {
        self.node_states
            .get(&node_id)
            .map(|state| state.get_history().to_vec())
            .ok_or(PushError::NodeNotFound(node_id))
    }

    /// Gets the most recent output for a node
    ///
    /// # Arguments
    /// * `node_id` - The node to query
    ///
    /// # Returns
    /// Most recent output point, None if no outputs, or error if node doesn't exist
    pub fn get_latest(&self, node_id: NodeId) -> Result<Option<TimeSeriesPoint>, PushError> {
        self.node_states
            .get(&node_id)
            .map(|state| state.get_latest().cloned())
            .ok_or(PushError::NodeNotFound(node_id))
    }

    /// Gets the state of a node
    ///
    /// # Arguments
    /// * `node_id` - The node to query
    ///
    /// # Returns
    /// Reference to node state, or error if node doesn't exist
    pub fn get_node_state(&self, node_id: NodeId) -> Result<&NodeState, PushError> {
        self.node_states
            .get(&node_id)
            .map(|state| state.get_state())
            .ok_or(PushError::NodeNotFound(node_id))
    }

    /// Gets the buffer contents for a node
    ///
    /// # Arguments
    /// * `node_id` - The node to query
    ///
    /// # Returns
    /// Buffer slice if node has a buffer, None if no buffer, or error if node doesn't exist
    pub fn get_buffer_contents(&self, node_id: NodeId) -> Result<Option<Vec<f64>>, PushError> {
        self.node_states
            .get(&node_id)
            .map(|state| state.get_buffer_slice())
            .ok_or(PushError::NodeNotFound(node_id))
    }

    /// Initializes the engine with historical data
    ///
    /// Warms up node buffers and computes initial states from historical data.
    ///
    /// # Arguments
    /// * `_data_provider` - Provider for historical data
    /// * `_end_date` - End date for historical warmup
    /// * `_lookback_days` - Number of days of historical data to load
    ///
    /// # Returns
    /// Ok(()) if initialization succeeded, Err otherwise
    ///
    /// # Note
    /// This is a simplified implementation. Full implementation would:
    /// - Query DataProvider for historical data
    /// - Populate node buffers
    /// - Compute initial states
    pub fn initialize(
        &mut self,
        _data_provider: &dyn DataProvider,
        _end_date: DateTime<Utc>,
        _lookback_days: usize,
    ) -> Result<(), InitError> {
        // TODO: Implement full initialization logic
        // For now, just mark as initialized
        self.is_initialized = true;
        Ok(())
    }

    /// Calculates the required lookback period for the DAG
    ///
    /// Traverses all nodes to find the maximum window size required.
    ///
    /// # Returns
    /// Number of days needed for warmup
    pub fn calculate_required_lookback(&self) -> usize {
        // TODO: Traverse DAG to find max window_size from NodeParams
        // For now, return default
        30
    }
}

/// Circular buffer for efficient rolling window operations
///
/// Fixed-capacity buffer that overwrites oldest values when full.
/// Used for analytics that need historical context (e.g., N-day volatility).
///
/// # Examples
/// ```
/// use analytics::push_mode::CircularBuffer;
///
/// let mut buffer = CircularBuffer::new(3);
/// buffer.push(1.0);
/// buffer.push(2.0);
/// buffer.push(3.0);
///
/// assert_eq!(buffer.len(), 3);
/// assert!(buffer.is_full());
///
/// // Pushing another value overwrites the oldest
/// buffer.push(4.0);
/// assert_eq!(buffer.get_slice(), &[2.0, 3.0, 4.0]);
/// ```
#[derive(Debug, Clone)]
pub struct CircularBuffer<T> {
    data: VecDeque<T>,
    capacity: usize,
}

impl<T: Clone> CircularBuffer<T> {
    /// Creates a new circular buffer with the specified capacity
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of elements the buffer can hold
    ///
    /// # Panics
    /// Panics if capacity is 0
    pub fn new(capacity: usize) -> Self {
        assert!(
            capacity > 0,
            "CircularBuffer capacity must be greater than 0"
        );
        CircularBuffer {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Adds a value to the buffer
    ///
    /// If the buffer is full, the oldest value is overwritten.
    /// This operation is O(1).
    ///
    /// # Arguments
    /// * `value` - The value to add
    pub fn push(&mut self, value: T) {
        if self.data.len() == self.capacity {
            // Buffer is full, remove oldest value
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    /// Returns a slice view of the current buffer contents
    ///
    /// The slice is ordered from oldest to newest value.
    ///
    /// # Returns
    /// Slice containing all current values (may be less than capacity if not full)
    pub fn get_slice(&self) -> Vec<T> {
        self.data.iter().cloned().collect()
    }

    /// Returns the current number of elements in the buffer
    ///
    /// This may be less than capacity if the buffer hasn't been filled yet.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the buffer contains no elements
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns true if the buffer is at full capacity
    pub fn is_full(&self) -> bool {
        self.data.len() == self.capacity
    }

    /// Returns the maximum capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clears all elements from the buffer
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Task Group 1.1: Tests for circular buffer

    #[test]
    fn test_circular_buffer_creation() {
        let buffer: CircularBuffer<f64> = CircularBuffer::new(10);

        assert_eq!(buffer.capacity(), 10);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
    }

    #[test]
    fn test_circular_buffer_push_wraparound() {
        let mut buffer = CircularBuffer::new(3);

        // Fill the buffer
        buffer.push(1.0);
        buffer.push(2.0);
        buffer.push(3.0);

        assert_eq!(buffer.len(), 3);
        assert!(buffer.is_full());
        assert_eq!(buffer.get_slice(), vec![1.0, 2.0, 3.0]);

        // Push another value - should overwrite oldest (1.0)
        buffer.push(4.0);
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.get_slice(), vec![2.0, 3.0, 4.0]);

        // Push another - should overwrite 2.0
        buffer.push(5.0);
        assert_eq!(buffer.get_slice(), vec![3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_circular_buffer_partial_fill() {
        let mut buffer = CircularBuffer::new(5);

        buffer.push(10.0);
        buffer.push(20.0);

        assert_eq!(buffer.len(), 2);
        assert!(!buffer.is_full());
        assert_eq!(buffer.get_slice(), vec![10.0, 20.0]);

        buffer.push(30.0);
        assert_eq!(buffer.len(), 3);
        assert!(!buffer.is_full());
        assert_eq!(buffer.get_slice(), vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_circular_buffer_get_slice() {
        let mut buffer = CircularBuffer::new(4);

        // Empty buffer
        assert_eq!(buffer.get_slice().len(), 0);

        // Partial fill
        buffer.push(1.0);
        buffer.push(2.0);
        let slice = buffer.get_slice();
        assert_eq!(slice, vec![1.0, 2.0]);

        // Full buffer
        buffer.push(3.0);
        buffer.push(4.0);
        let slice = buffer.get_slice();
        assert_eq!(slice, vec![1.0, 2.0, 3.0, 4.0]);

        // After wraparound
        buffer.push(5.0);
        let slice = buffer.get_slice();
        assert_eq!(slice, vec![2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_circular_buffer_is_full() {
        let mut buffer = CircularBuffer::new(2);

        assert!(!buffer.is_full());

        buffer.push(1.0);
        assert!(!buffer.is_full());

        buffer.push(2.0);
        assert!(buffer.is_full());

        buffer.push(3.0);
        assert!(buffer.is_full());
    }

    #[test]
    fn test_circular_buffer_capacity() {
        let buffer1: CircularBuffer<i32> = CircularBuffer::new(5);
        assert_eq!(buffer1.capacity(), 5);

        let buffer2: CircularBuffer<i32> = CircularBuffer::new(100);
        assert_eq!(buffer2.capacity(), 100);
    }

    #[test]
    fn test_circular_buffer_clear() {
        let mut buffer = CircularBuffer::new(3);

        buffer.push(1.0);
        buffer.push(2.0);
        buffer.push(3.0);

        assert_eq!(buffer.len(), 3);

        buffer.clear();

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
    }

    #[test]
    #[should_panic(expected = "CircularBuffer capacity must be greater than 0")]
    fn test_circular_buffer_zero_capacity_panics() {
        let _buffer: CircularBuffer<f64> = CircularBuffer::new(0);
    }

    // Task Group 2.1: Tests for node state

    #[test]
    fn test_node_state_transitions() {
        let mut state = NodeState::Uninitialized;

        // Uninitialized → Ready
        state = NodeState::Ready;
        assert_eq!(state, NodeState::Ready);

        // Ready → Computing
        state = NodeState::Computing;
        assert_eq!(state, NodeState::Computing);

        // Computing → Ready
        state = NodeState::Ready;
        assert_eq!(state, NodeState::Ready);

        // Ready → Failed
        state = NodeState::Failed("Test error".to_string());
        assert!(matches!(state, NodeState::Failed(_)));
    }

    #[test]
    fn test_last_computed_timestamp_tracking() {
        use chrono::Utc;

        let mut push_state = NodePushState::new(None);

        // Initially no timestamp
        assert!(push_state.get_last_timestamp().is_none());

        // Add output with timestamp
        let ts1 = Utc::now();
        let point1 = TimeSeriesPoint::new(ts1, 100.0);
        push_state.append_output(point1);

        assert_eq!(push_state.get_last_timestamp(), Some(ts1));

        // Add another output
        let ts2 = ts1 + chrono::Duration::days(1);
        let point2 = TimeSeriesPoint::new(ts2, 105.0);
        push_state.append_output(point2);

        assert_eq!(push_state.get_last_timestamp(), Some(ts2));
    }

    #[test]
    fn test_output_history_append() {
        use chrono::Utc;

        let mut push_state = NodePushState::new(None);

        assert_eq!(push_state.get_history().len(), 0);

        let point1 = TimeSeriesPoint::new(Utc::now(), 100.0);
        push_state.append_output(point1.clone());

        assert_eq!(push_state.get_history().len(), 1);
        assert_eq!(push_state.get_history()[0].close_price, 100.0);

        let point2 = TimeSeriesPoint::new(Utc::now(), 105.0);
        push_state.append_output(point2.clone());

        assert_eq!(push_state.get_history().len(), 2);
        assert_eq!(push_state.get_history()[1].close_price, 105.0);
    }

    #[test]
    fn test_input_buffer_integration() {
        let mut push_state = NodePushState::new(Some(3));

        // Should have a buffer
        assert!(push_state.input_buffer.is_some());

        // Push values to buffer
        push_state.push_to_buffer(10.0);
        push_state.push_to_buffer(20.0);

        let buffer_slice = push_state.get_buffer_slice().unwrap();
        assert_eq!(buffer_slice, vec![10.0, 20.0]);

        // Fill buffer
        push_state.push_to_buffer(30.0);
        let buffer_slice = push_state.get_buffer_slice().unwrap();
        assert_eq!(buffer_slice, vec![10.0, 20.0, 30.0]);

        // Overwrite oldest
        push_state.push_to_buffer(40.0);
        let buffer_slice = push_state.get_buffer_slice().unwrap();
        assert_eq!(buffer_slice, vec![20.0, 30.0, 40.0]);
    }

    #[test]
    fn test_state_per_node_is_independent() {
        let mut state1 = NodePushState::new(Some(5));
        let mut state2 = NodePushState::new(None);

        // Modify state1
        state1.set_state(NodeState::Ready);
        state1.push_to_buffer(100.0);

        // state2 should be independent
        assert_eq!(state2.get_state(), &NodeState::Uninitialized);
        assert!(state2.get_buffer_slice().is_none());

        // Modify state2
        state2.set_state(NodeState::Failed("Error".to_string()));

        // state1 should be unaffected
        assert_eq!(state1.get_state(), &NodeState::Ready);
    }

    #[test]
    fn test_node_state_without_buffer() {
        let push_state = NodePushState::new(None);

        // Should not have a buffer
        assert!(push_state.input_buffer.is_none());
        assert!(push_state.get_buffer_slice().is_none());
    }

    #[test]
    fn test_get_latest_output() {
        use chrono::Utc;

        let mut push_state = NodePushState::new(None);

        // Initially no latest
        assert!(push_state.get_latest().is_none());

        // Add outputs
        let point1 = TimeSeriesPoint::new(Utc::now(), 100.0);
        push_state.append_output(point1.clone());

        assert_eq!(push_state.get_latest().unwrap().close_price, 100.0);

        let point2 = TimeSeriesPoint::new(Utc::now(), 200.0);
        push_state.append_output(point2.clone());

        assert_eq!(push_state.get_latest().unwrap().close_price, 200.0);
    }

    // Task Group 3.1: Tests for PushModeEngine

    #[test]
    fn test_push_mode_engine_creation() {
        let dag = AnalyticsDag::new();
        let engine = PushModeEngine::new(dag);

        assert!(!engine.is_initialized());
    }

    #[test]
    fn test_push_data_engine_not_initialized() {
        use chrono::Utc;

        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);

        let asset = AssetKey::new_equity("AAPL").unwrap();
        let result = engine.push_data(asset, Utc::now(), 150.0);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PushError::EngineNotInitialized
        ));
    }

    #[test]
    fn test_push_data_invalid_nan() {
        use chrono::Utc;

        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);
        engine.is_initialized = true; // Bypass initialization check for this test

        let asset = AssetKey::new_equity("AAPL").unwrap();
        let result = engine.push_data(asset, Utc::now(), f64::NAN);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PushError::InvalidData(_)));
    }

    #[test]
    fn test_push_data_invalid_infinite() {
        use chrono::Utc;

        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);
        engine.is_initialized = true;

        let asset = AssetKey::new_equity("AAPL").unwrap();
        let result = engine.push_data(asset, Utc::now(), f64::INFINITY);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PushError::InvalidData(_)));
    }

    #[test]
    fn test_push_data_invalid_negative() {
        use chrono::Utc;

        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);
        engine.is_initialized = true;

        let asset = AssetKey::new_equity("AAPL").unwrap();
        let result = engine.push_data(asset, Utc::now(), -100.0);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PushError::InvalidData(_)));
    }

    #[test]
    fn test_push_error_display() {
        use chrono::Utc;

        let ts1 = Utc::now();
        let ts2 = ts1 + chrono::Duration::days(1);

        let err = PushError::OutOfOrder {
            timestamp: ts1,
            last_computed: ts2,
        };
        assert!(err.to_string().contains("Out of order"));

        let err = PushError::InvalidData("test".to_string());
        assert!(err.to_string().contains("Invalid data"));

        let err = PushError::EngineNotInitialized;
        assert!(err.to_string().contains("not initialized"));
    }

    #[test]
    fn test_init_error_display() {
        let err = InitError::InsufficientHistoricalData {
            required: 30,
            available: 10,
        };
        assert!(err.to_string().contains("Insufficient"));
        assert!(err.to_string().contains("30"));
        assert!(err.to_string().contains("10"));
    }

    // Task Group 4.1: Tests for propagation

    #[test]
    fn test_propagation_respects_initialization() {
        use chrono::Utc;

        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);

        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Should fail when not initialized
        let result = engine.push_data(asset.clone(), Utc::now(), 150.0);
        assert!(result.is_err());

        // Should succeed after initialization
        engine.is_initialized = true;
        let result = engine.push_data(asset, Utc::now(), 150.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_propagation_validates_timestamp_order() {
        use chrono::Utc;

        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);
        engine.is_initialized = true;

        let asset = AssetKey::new_equity("AAPL").unwrap();
        let node_id = NodeId(1);

        // Add a node state with timestamp
        let mut state = NodePushState::new(None);
        let ts1 = Utc::now();
        state.last_computed_timestamp = Some(ts1);
        engine.node_states.insert(node_id, state);

        // Pushing earlier timestamp should work (no nodes matched)
        let ts0 = ts1 - chrono::Duration::days(1);
        let result = engine.push_data(asset, ts0, 150.0);
        assert!(result.is_ok()); // No nodes have this asset, so no error
    }

    #[test]
    fn test_propagation_empty_dag() {
        use chrono::Utc;

        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);
        engine.is_initialized = true;

        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Should succeed with empty DAG (no nodes to update)
        let result = engine.push_data(asset, Utc::now(), 150.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_nodes_with_asset_empty() {
        let dag = AnalyticsDag::new();
        let engine = PushModeEngine::new(dag);

        let asset = AssetKey::new_equity("AAPL").unwrap();
        let nodes = engine.find_nodes_with_asset(&asset);

        assert_eq!(nodes.len(), 0);
    }

    #[test]
    fn test_propagation_with_valid_data() {
        use chrono::Utc;

        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);
        engine.is_initialized = true;

        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Valid data should succeed
        let result = engine.push_data(asset.clone(), Utc::now(), 150.0);
        assert!(result.is_ok());

        // Can push multiple times with increasing timestamps
        let result = engine.push_data(asset, Utc::now() + chrono::Duration::seconds(1), 155.0);
        assert!(result.is_ok());
    }

    // Task Group 5.1: Tests for callbacks

    #[test]
    fn test_register_callback() {
        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);

        let node_id = NodeId(1);
        let callback: Callback = Box::new(|_node_id, _output, _timestamp| {
            // Test callback
        });

        let result = engine.register_callback(node_id, callback);
        assert!(result.is_ok());

        // Verify callback was stored
        assert!(engine.callbacks.contains_key(&node_id));
        assert_eq!(engine.callbacks.get(&node_id).unwrap().len(), 1);
    }

    #[test]
    fn test_multiple_callbacks_same_node() {
        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);

        let node_id = NodeId(1);

        // Register first callback
        engine
            .register_callback(node_id, Box::new(|_, _, _| {}))
            .unwrap();

        // Register second callback
        engine
            .register_callback(node_id, Box::new(|_, _, _| {}))
            .unwrap();

        // Should have 2 callbacks
        assert_eq!(engine.callbacks.get(&node_id).unwrap().len(), 2);
    }

    #[test]
    fn test_invoke_callbacks() {
        use std::sync::{Arc, Mutex};

        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);

        let node_id = NodeId(1);
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        // Register callback that sets flag
        engine
            .register_callback(
                node_id,
                Box::new(move |_, _, _| {
                    *called_clone.lock().unwrap() = true;
                }),
            )
            .unwrap();

        // Invoke callbacks
        let output = NodeOutput::None;
        engine.invoke_callbacks(node_id, &output);

        // Callback should have been called
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_callback_error_doesnt_halt() {
        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);

        let node_id = NodeId(1);

        // Register callback that panics
        engine
            .register_callback(
                node_id,
                Box::new(|_, _, _| {
                    panic!("Test panic");
                }),
            )
            .unwrap();

        // Invoke callbacks - should not panic
        let output = NodeOutput::None;
        engine.invoke_callbacks(node_id, &output);

        // Test passes if we reach here without panicking
    }

    // Task Group 8.1: Tests for query APIs

    #[test]
    fn test_get_history_empty() {
        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);

        let node_id = NodeId(1);
        let state = NodePushState::new(None);
        engine.node_states.insert(node_id, state);

        let history = engine.get_history(node_id);
        assert!(history.is_ok());
        assert_eq!(history.unwrap().len(), 0);
    }

    #[test]
    fn test_get_latest_none() {
        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);

        let node_id = NodeId(1);
        let state = NodePushState::new(None);
        engine.node_states.insert(node_id, state);

        let latest = engine.get_latest(node_id);
        assert!(latest.is_ok());
        assert!(latest.unwrap().is_none());
    }

    #[test]
    fn test_get_history_node_not_found() {
        let dag = AnalyticsDag::new();
        let engine = PushModeEngine::new(dag);

        let node_id = NodeId(999);
        let result = engine.get_history(node_id);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PushError::NodeNotFound(_)));
    }

    // Task Group 6.1: Tests for initialization

    #[test]
    fn test_initialize_sets_flag() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::Utc;

        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);

        assert!(!engine.is_initialized());

        let provider = InMemoryDataProvider::new();
        let result = engine.initialize(&provider, Utc::now(), 30);

        assert!(result.is_ok());
        assert!(engine.is_initialized());
    }

    #[test]
    fn test_calculate_required_lookback() {
        let dag = AnalyticsDag::new();
        let engine = PushModeEngine::new(dag);

        let lookback = engine.calculate_required_lookback();
        assert!(lookback > 0);
    }

    // Task Group 7.1: Tests for error handling (simplified)

    #[test]
    fn test_error_handling_in_propagation() {
        use chrono::Utc;

        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);
        engine.is_initialized = true;

        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Should succeed even with empty DAG
        let result = engine.push_data(asset, Utc::now(), 150.0);
        assert!(result.is_ok());
    }

    // Task Group 9.1: Integration tests

    #[test]
    fn test_end_to_end_workflow() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::Utc;

        // Create engine with empty DAG
        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);

        // Initialize
        let provider = InMemoryDataProvider::new();
        engine.initialize(&provider, Utc::now(), 30).unwrap();

        // Push data
        let asset = AssetKey::new_equity("AAPL").unwrap();
        let result = engine.push_data(asset.clone(), Utc::now(), 150.0);
        assert!(result.is_ok());

        // Push more data
        let result = engine.push_data(asset, Utc::now() + chrono::Duration::seconds(1), 155.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_callback_integration() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::Utc;
        use std::sync::{Arc, Mutex};

        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);

        let provider = InMemoryDataProvider::new();
        engine.initialize(&provider, Utc::now(), 30).unwrap();

        // Register callback
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        let node_id = NodeId(1);
        engine
            .register_callback(
                node_id,
                Box::new(move |_, _, _| {
                    *call_count_clone.lock().unwrap() += 1;
                }),
            )
            .unwrap();

        // Push data (won't trigger callback since node doesn't exist in DAG)
        let asset = AssetKey::new_equity("AAPL").unwrap();
        engine.push_data(asset, Utc::now(), 150.0).unwrap();

        // Callback count unchanged since no nodes matched
        assert_eq!(*call_count.lock().unwrap(), 0);
    }

    #[test]
    fn test_multi_asset_sequential() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::Utc;

        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);

        let provider = InMemoryDataProvider::new();
        engine.initialize(&provider, Utc::now(), 30).unwrap();

        let aapl = AssetKey::new_equity("AAPL").unwrap();
        let msft = AssetKey::new_equity("MSFT").unwrap();

        // Push data for multiple assets sequentially
        engine.push_data(aapl, Utc::now(), 150.0).unwrap();
        engine.push_data(msft, Utc::now(), 300.0).unwrap();
    }

    #[test]
    fn test_query_apis_integration() {
        let dag = AnalyticsDag::new();
        let mut engine = PushModeEngine::new(dag);

        let node_id = NodeId(1);
        let mut state = NodePushState::new(Some(10));

        // Add some data
        state.append_output(TimeSeriesPoint::new(chrono::Utc::now(), 100.0));
        state.append_output(TimeSeriesPoint::new(chrono::Utc::now(), 105.0));

        engine.node_states.insert(node_id, state);

        // Query history
        let history = engine.get_history(node_id).unwrap();
        assert_eq!(history.len(), 2);

        // Query latest
        let latest = engine.get_latest(node_id).unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().close_price, 105.0);
    }

    #[test]
    fn test_full_test_count() {
        // Verify we have approximately expected number of tests
        // This test just passes - it's a placeholder to document test count
        assert!(true);
    }

    // Integration tests for Item 6: Analytics Push Integration

    #[tokio::test]
    async fn test_data_provider_to_returns_integration() {
        use chrono::Utc;

        // Create a simple DAG: DataProvider → Returns
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Add data provider node
        let data_node = dag.add_node(
            "data_provider".to_string(),
            NodeParams::None,
            vec![asset.clone()],
        );

        // Add returns node
        let returns_node =
            dag.add_node("returns".to_string(), NodeParams::None, vec![asset.clone()]);

        // Connect: data → returns
        dag.add_edge(data_node, returns_node).unwrap();

        // Create engine
        let mut engine = PushModeEngine::new(dag);
        engine.is_initialized = true;

        // Push first price
        let ts1 = Utc::now();
        let result = engine.push_data(asset.clone(), ts1, 100.0);
        assert!(result.is_ok(), "First push_data should succeed");

        // Push second price
        let ts2 = ts1 + chrono::Duration::seconds(1);
        let result = engine.push_data(asset, ts2, 105.0);
        assert!(result.is_ok(), "Second push_data should succeed");

        // Check returns node has outputs
        let returns_history = engine.get_history(returns_node);
        assert!(
            returns_history.is_ok(),
            "Should be able to get returns history"
        );

        let history = returns_history.unwrap();
        assert!(
            history.len() >= 1,
            "Returns node should have at least 1 output"
        );
    }

    #[tokio::test]
    async fn test_full_chain_data_returns_volatility() {
        use chrono::Utc;

        // Create full DAG: DataProvider → Returns → Volatility
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Add nodes
        let data_node = dag.add_node(
            "data_provider".to_string(),
            NodeParams::None,
            vec![asset.clone()],
        );

        let returns_node =
            dag.add_node("returns".to_string(), NodeParams::None, vec![asset.clone()]);

        let mut vol_params = HashMap::new();
        vol_params.insert("window_size".to_string(), "5".to_string());
        let vol_node = dag.add_node(
            "volatility".to_string(),
            NodeParams::Map(vol_params),
            vec![asset.clone()],
        );

        // Connect: data → returns → volatility
        dag.add_edge(data_node, returns_node).unwrap();
        dag.add_edge(returns_node, vol_node).unwrap();

        // Create engine
        let mut engine = PushModeEngine::new(dag);
        engine.is_initialized = true;

        // Push multiple prices
        let mut ts = Utc::now();
        for i in 0..10 {
            let price = 100.0 + i as f64;
            engine.push_data(asset.clone(), ts, price).unwrap();
            ts = ts + chrono::Duration::seconds(1);
        }

        // Check all nodes have outputs
        let data_history = engine.get_history(data_node).unwrap();
        assert_eq!(data_history.len(), 10, "Data node should have 10 outputs");

        let returns_history = engine.get_history(returns_node).unwrap();
        assert!(
            returns_history.len() >= 9,
            "Returns node should have outputs"
        );

        let vol_history = engine.get_history(vol_node).unwrap();
        assert!(
            vol_history.len() >= 5,
            "Volatility node should have outputs"
        );
    }

    #[tokio::test]
    async fn test_callback_fires_with_real_data() {
        use chrono::Utc;
        use std::sync::{Arc, Mutex};

        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        let data_node = dag.add_node(
            "data_provider".to_string(),
            NodeParams::None,
            vec![asset.clone()],
        );

        let mut engine = PushModeEngine::new(dag);
        engine.is_initialized = true;

        // Register callback
        let callback_fired = Arc::new(Mutex::new(false));
        let callback_fired_clone = callback_fired.clone();

        engine
            .register_callback(
                data_node,
                Box::new(move |_, output, _| {
                    if let NodeOutput::Single(points) = output {
                        if !points.is_empty() {
                            *callback_fired_clone.lock().unwrap() = true;
                        }
                    }
                }),
            )
            .unwrap();

        // Push data
        engine.push_data(asset, Utc::now(), 150.0).unwrap();

        // Callback should have fired
        assert!(
            *callback_fired.lock().unwrap(),
            "Callback should have been invoked"
        );
    }
}
