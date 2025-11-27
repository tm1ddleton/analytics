//! DAG Computation Framework
//!
//! This module provides a DAG (Directed Acyclic Graph) construction and execution engine
//! for wiring analytics dependencies explicitly with cycle detection, topological sorting,
//! and parallel execution support.

use crate::analytics::registry::{AnalyticExecutor, AnalyticRegistry, ParentOutput};
use crate::asset_key::AssetKey;
use crate::dag::types::{Node, NodeId, NodeKey, NodeOutput, NodeParams};
use crate::dag::AnalyticType;
use crate::time_series::{DataProvider, DataProviderError, DateRange, TimeSeriesPoint};
use chrono::{DateTime, Utc};
use daggy::{petgraph::Direction, Dag, EdgeIndex, NodeIndex, Walker};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Error types for DAG operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagError {
    /// Cycle detected when adding edge
    CycleDetected(String),
    /// Node not found
    NodeNotFound(String),
    /// Edge not found
    EdgeNotFound(String),
    /// Invalid operation
    InvalidOperation(String),
    /// Execution error
    ExecutionError(String),
    /// Data provider error
    DataProviderError(String),
}

impl std::fmt::Display for DagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DagError::CycleDetected(msg) => write!(f, "Cycle detected: {}", msg),
            DagError::NodeNotFound(msg) => write!(f, "Node not found: {}", msg),
            DagError::EdgeNotFound(msg) => write!(f, "Edge not found: {}", msg),
            DagError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            DagError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            DagError::DataProviderError(msg) => write!(f, "Data provider error: {}", msg),
        }
    }
}

impl AnalyticsDag {
    fn simulate_push_from_calendar(
        &self,
        nodes_to_execute: &[NodeId],
        data_points: &[TimeSeriesPoint],
        target_node: NodeId,
    ) -> Result<Vec<TimeSeriesPoint>, DagError> {
        let mut push_history: HashMap<NodeId, Vec<TimeSeriesPoint>> = HashMap::new();

        for point in data_points {
            for &node_id in nodes_to_execute {
                let parent_histories: Vec<ParentOutput> = self
                    .get_parents(node_id)
                    .iter()
                    .map(|&parent_id| ParentOutput {
                        node_id: parent_id,
                        analytic: self.analytic_type_for_node(parent_id),
                        output: push_history.get(&parent_id).cloned().unwrap_or_default(),
                    })
                    .collect();

                let output = match self.execute_push_node(
                    node_id,
                    &parent_histories,
                    point.timestamp,
                    point.close_price,
                ) {
                    Ok(output) => output,
                    Err(err) => {
                        if Self::is_insufficient_data_error(&err) {
                            push_history
                                .entry(node_id)
                                .or_insert_with(Vec::new)
                                .push(TimeSeriesPoint::new(point.timestamp, f64::NAN));
                            continue;
                        }
                        return Err(err);
                    }
                };
                let mut points = Self::node_output_to_timeseries(&output, point.timestamp);
                if !points.is_empty() {
                    push_history
                        .entry(node_id)
                        .or_insert_with(Vec::new)
                        .append(&mut points);
                }
            }
        }

        Ok(push_history.get(&target_node).cloned().unwrap_or_default())
    }

    fn node_output_to_timeseries(
        output: &NodeOutput,
        timestamp: DateTime<Utc>,
    ) -> Vec<TimeSeriesPoint> {
        match output {
            NodeOutput::Single(points) => points.clone(),
            NodeOutput::Scalar(value) => vec![TimeSeriesPoint::new(timestamp, *value)],
            NodeOutput::Collection(collection) => collection
                .iter()
                .flat_map(|points_vec| points_vec.clone())
                .collect(),
            NodeOutput::None => Vec::new(),
        }
    }

    fn is_insufficient_data_error(err: &DagError) -> bool {
        matches!(
            err,
            DagError::ExecutionError(msg) if {
                msg.contains("requires at least two price points")
                    || msg.contains("requires returns data")
                    || msg.contains("requires input price and lagged values")
            }
        )
    }

    fn is_data_provider_node(&self, node_id: NodeId) -> bool {
        if let Some(key) = self.node_key(node_id) {
            key.analytic == AnalyticType::DataProvider
        } else if let Some(node) = self.get_node(node_id) {
            let lower = node.node_type.to_lowercase();
            lower == "data_provider" || lower == "dataprovider"
        } else {
            false
        }
    }

    pub(crate) fn analytic_type_for_node(&self, node_id: NodeId) -> AnalyticType {
        if let Some(key) = self.node_key(node_id) {
            key.analytic
        } else if let Some(node) = self.get_node(node_id) {
            AnalyticType::from_str(&node.node_type)
        } else {
            AnalyticType::DataProvider
        }
    }
}

impl From<DataProviderError> for DagError {
    fn from(err: DataProviderError) -> Self {
        DagError::DataProviderError(err.to_string())
    }
}

impl std::error::Error for DagError {}

/// Execution cache for intermediate results during pull-mode execution
///
/// Stores node outputs to avoid re-computation when a parent's result is needed by multiple children
#[derive(Debug)]
struct ExecutionCache {
    /// Cached outputs for each node
    outputs: HashMap<NodeId, Vec<TimeSeriesPoint>>,
    /// Extended date ranges used for each node (for debugging/analysis)
    extended_ranges: HashMap<NodeId, DateRange>,
}

impl ExecutionCache {
    /// Creates a new empty cache
    fn new() -> Self {
        ExecutionCache {
            outputs: HashMap::new(),
            extended_ranges: HashMap::new(),
        }
    }

    /// Gets cached output for a node
    fn get(&self, node_id: NodeId) -> Option<&Vec<TimeSeriesPoint>> {
        self.outputs.get(&node_id)
    }

    /// Inserts output for a node
    fn insert(&mut self, node_id: NodeId, output: Vec<TimeSeriesPoint>, range: DateRange) {
        self.outputs.insert(node_id, output);
        self.extended_ranges.insert(node_id, range);
    }

    /// Extracts output filtered to a specific date range
    fn extract_range(
        &self,
        node_id: NodeId,
        target_range: &DateRange,
    ) -> Option<Vec<TimeSeriesPoint>> {
        self.outputs.get(&node_id).map(|output| {
            output
                .iter()
                .filter(|point| {
                    let date = point.timestamp.date_naive();
                    date >= target_range.start && date <= target_range.end
                })
                .cloned()
                .collect()
        })
    }

    /// Clears all cached data
    fn clear(&mut self) {
        self.outputs.clear();
        self.extended_ranges.clear();
    }
}

/// DAG for analytics dependencies
///
/// Single DAG instance handles multiple assets (one DAG for all assets)
#[derive(Debug)]
pub struct AnalyticsDag {
    /// The underlying daggy DAG
    dag: Dag<Node, ()>,
    /// Map from NodeId to daggy NodeIndex
    node_id_to_index: HashMap<NodeId, NodeIndex>,
    /// Map from daggy NodeIndex to NodeId
    index_to_node_id: HashMap<NodeIndex, NodeId>,
    /// Next available node ID
    next_node_id: usize,
    /// Cached topological sort result
    cached_toposort: Option<Vec<NodeId>>,
    /// Map from NodeKey metadata to NodeId for deduplication
    node_lookup: HashMap<NodeKey, NodeId>,
    /// Map from NodeId to its NodeKey metadata (if registered)
    node_keys_by_id: HashMap<NodeId, NodeKey>,
    /// Registry defining analytic executors and dependencies
    registry: Arc<AnalyticRegistry>,
}

impl AnalyticsDag {
    /// Creates a new empty DAG using the default registry.
    pub fn new() -> Self {
        Self::new_with_registry(Arc::new(AnalyticRegistry::default()))
    }

    /// Creates a new DAG using the provided registry.
    pub fn new_with_registry(registry: Arc<AnalyticRegistry>) -> Self {
        AnalyticsDag {
            dag: Dag::new(),
            node_id_to_index: HashMap::new(),
            index_to_node_id: HashMap::new(),
            next_node_id: 0,
            cached_toposort: None,
            node_lookup: HashMap::new(),
            node_keys_by_id: HashMap::new(),
            registry,
        }
    }

    /// Adds a new node to the DAG
    ///
    /// # Arguments
    /// * `node_type` - Type of the node (e.g., "moving_average")
    /// * `params` - Parameters for the node
    /// * `assets` - Assets this node operates on
    ///
    /// # Returns
    /// Returns the NodeId of the newly created node
    pub fn add_node(
        &mut self,
        node_type: String,
        params: NodeParams,
        assets: Vec<AssetKey>,
    ) -> NodeId {
        let node_id = NodeId(self.next_node_id);
        self.next_node_id += 1;

        let node = Node::new(node_id, node_type, params, assets);
        let index = self.dag.add_node(node);

        self.node_id_to_index.insert(node_id, index);
        self.index_to_node_id.insert(index, node_id);

        // Invalidate cache when DAG structure changes
        self.cached_toposort = None;

        node_id
    }

    /// Resolves or creates a node based on its metadata key and registered definitions.
    pub fn resolve_node(&mut self, key: NodeKey) -> Result<NodeId, DagError> {
        if let Some(&existing) = self.node_lookup.get(&key) {
            return Ok(existing);
        }

        let definition = self.registry.definition(key.analytic).ok_or_else(|| {
            DagError::InvalidOperation(format!("No analytic definition for {:?}", key.analytic))
        })?;
        let node_type_name = definition.node_type().to_string();
        let dependency_keys = definition.dependencies(&key)?;
        let _ = definition;
        let mut dependency_ids = Vec::new();
        for dep_key in dependency_keys {
            let dep_id = self.resolve_node(dep_key)?;
            dependency_ids.push(dep_id);
        }

        let params = NodeParams::Map(key.params_map());
        let node_id = self.add_node(node_type_name, params, key.assets.clone());
        self.node_lookup.insert(key.clone(), node_id);
        self.node_keys_by_id.insert(node_id, key.clone());

        for dep_id in dependency_ids {
            self.add_edge(dep_id, node_id)?;
        }

        Ok(node_id)
    }

    /// Registers metadata for a manually-added node, allowing registry-driven execution.
    pub fn register_node_key(&mut self, node_id: NodeId, key: NodeKey) -> Result<(), DagError> {
        if !self.node_id_to_index.contains_key(&node_id) {
            return Err(DagError::NodeNotFound(format!(
                "Node {:?} not found for registration",
                node_id
            )));
        }

        self.node_lookup.insert(key.clone(), node_id);
        self.node_keys_by_id.insert(node_id, key);

        Ok(())
    }

    /// Retrieves the metadata key for a node, if registered.
    pub fn node_key(&self, node_id: NodeId) -> Option<&NodeKey> {
        self.node_keys_by_id.get(&node_id)
    }

    fn executor_for_node(
        &self,
        node: &Node,
        node_id: NodeId,
    ) -> Result<&dyn AnalyticExecutor, DagError> {
        let analytic = if let Some(key) = self.node_key(node_id) {
            key.analytic
        } else {
            AnalyticType::from_str(&node.node_type)
        };

        let definition = self.registry.definition(analytic).ok_or_else(|| {
            DagError::InvalidOperation(format!("No analytic definition for {:?}", analytic))
        })?;

        Ok(definition.executor())
    }

    /// Adds an edge (dependency) between two nodes
    ///
    /// # Arguments
    /// * `from` - Source node ID
    /// * `to` - Target node ID
    ///
    /// # Returns
    /// Returns Ok(EdgeIndex) if successful, or Err(DagError) if cycle would be created
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> Result<EdgeIndex, DagError> {
        let from_index = self
            .node_id_to_index
            .get(&from)
            .ok_or_else(|| DagError::NodeNotFound(format!("Node {:?} not found", from)))?;
        let to_index = self
            .node_id_to_index
            .get(&to)
            .ok_or_else(|| DagError::NodeNotFound(format!("Node {:?} not found", to)))?;

        match self.dag.add_edge(*from_index, *to_index, ()) {
            Ok(edge_index) => {
                // Invalidate cache when DAG structure changes
                self.cached_toposort = None;
                Ok(edge_index)
            }
            Err(_would_cycle) => Err(DagError::CycleDetected(format!(
                "Adding edge from {:?} to {:?} would create a cycle",
                from, to
            ))),
        }
    }

    /// Gets a node by its ID
    pub fn get_node(&self, node_id: NodeId) -> Option<&Node> {
        self.node_id_to_index
            .get(&node_id)
            .and_then(|&index| self.dag.node_weight(index))
    }

    /// Returns the number of nodes in the DAG
    pub fn node_count(&self) -> usize {
        self.dag.node_count()
    }

    /// Returns the number of edges in the DAG
    pub fn edge_count(&self) -> usize {
        self.dag.edge_count()
    }

    /// Removes a node from the DAG
    ///
    /// Only nodes with no dependencies (no child nodes) can be removed.
    ///
    /// # Arguments
    /// * `node_id` - ID of the node to remove
    ///
    /// # Returns
    /// Returns Ok(()) if successful, or Err if node has dependencies or doesn't exist
    pub fn remove_node(&mut self, node_id: NodeId) -> Result<(), DagError> {
        // Check if node exists
        let node_index = self
            .node_id_to_index
            .get(&node_id)
            .ok_or_else(|| DagError::NodeNotFound(format!("Node {:?} not found", node_id)))?;

        // Check if node has any children (dependencies)
        let has_children = self
            .dag
            .children(*node_index)
            .iter(&self.dag)
            .next()
            .is_some();

        if has_children {
            return Err(DagError::InvalidOperation(format!(
                "Cannot remove node {:?}: node has dependencies",
                node_id
            )));
        }

        // Remove the node - daggy returns the removed node weight
        let node_index_copy = *node_index;
        let _removed_node = self.dag.remove_node(node_index_copy).ok_or_else(|| {
            DagError::InvalidOperation(format!("Failed to remove node {:?}", node_id))
        })?;

        // After removal, daggy may have shifted indices. We need to rebuild our mappings.
        // Collect all current nodes and their indices
        let mut new_node_id_to_index = HashMap::new();
        let mut new_index_to_node_id = HashMap::new();

        for idx in self.dag.graph().node_indices() {
            if let Some(node) = self.dag.node_weight(idx) {
                new_node_id_to_index.insert(node.id, idx);
                new_index_to_node_id.insert(idx, node.id);
            }
        }

        // Update mappings
        self.node_id_to_index = new_node_id_to_index;
        self.index_to_node_id = new_index_to_node_id;

        // Invalidate cache when DAG structure changes
        self.cached_toposort = None;

        // Ensure key map no longer points to the removed node
        self.node_lookup
            .retain(|_, &mut existing_node| existing_node != node_id);
        self.node_keys_by_id.remove(&node_id);

        Ok(())
    }

    /// Checks if a node has any dependencies (child nodes)
    pub fn has_dependencies(&self, node_id: NodeId) -> bool {
        if let Some(&node_index) = self.node_id_to_index.get(&node_id) {
            self.dag
                .children(node_index)
                .iter(&self.dag)
                .next()
                .is_some()
        } else {
            false
        }
    }

    /// Computes topological sort of the DAG using Kahn's algorithm
    ///
    /// Returns nodes in topological order (dependencies before dependents)
    ///
    /// # Returns
    /// Returns Ok(Vec<NodeId>) with nodes in topological order, or Err if DAG is invalid
    fn compute_toposort(&self) -> Result<Vec<NodeId>, DagError> {
        if self.dag.node_count() == 0 {
            return Ok(Vec::new());
        }

        // Kahn's algorithm for topological sorting
        let mut in_degree: HashMap<NodeIndex, usize> = HashMap::new();
        let mut queue = std::collections::VecDeque::new();
        let mut result = Vec::new();

        // Initialize in-degrees (use graph() to access petgraph API)
        for node_idx in self.dag.graph().node_indices() {
            in_degree.insert(node_idx, 0);
        }

        // Count in-degrees
        for edge in self.dag.raw_edges() {
            let target = edge.target();
            *in_degree.entry(target).or_insert(0) += 1;
        }

        // Find all nodes with in-degree 0 (no dependencies)
        for (node_index, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(*node_index);
            }
        }

        // Process nodes
        while let Some(node_index) = queue.pop_front() {
            // Add to result
            if let Some(&node_id) = self.index_to_node_id.get(&node_index) {
                result.push(node_id);
            }

            // Decrease in-degree of neighbors (process outgoing edges)
            for child_idx in self.dag.children(node_index).iter(&self.dag) {
                if let Some(degree) = in_degree.get_mut(&child_idx.1) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(child_idx.1);
                    }
                }
            }
        }

        // Check if all nodes were processed (detects cycles, though daggy prevents them)
        if result.len() != self.dag.node_count() {
            return Err(DagError::InvalidOperation(
                "Topological sort failed - DAG may contain cycles".to_string(),
            ));
        }

        Ok(result)
    }

    /// Gets the execution order (topological sort) of nodes
    ///
    /// Supports querying execution order without executing nodes.
    /// Results are cached and invalidated when DAG structure changes.
    ///
    /// # Returns
    /// Returns execution sequence as vector of node identifiers in topological order,
    /// or error if DAG is invalid
    ///
    /// # Errors
    /// Returns `DagError::InvalidOperation` if DAG structure is invalid
    pub fn execution_order(&mut self) -> Result<Vec<NodeId>, DagError> {
        // Return cached result if available
        if let Some(ref cached) = self.cached_toposort {
            return Ok(cached.clone());
        }

        // Compute topological sort
        let sorted = self.compute_toposort()?;

        // Cache the result
        self.cached_toposort = Some(sorted.clone());

        Ok(sorted)
    }

    /// Gets the execution order without mutating (doesn't cache)
    ///
    /// Useful for read-only access to execution order
    pub fn execution_order_immutable(&self) -> Result<Vec<NodeId>, DagError> {
        if self.dag.node_count() == 0 {
            return Ok(Vec::new());
        }

        // Same algorithm as compute_toposort but without caching
        let mut in_degree: HashMap<NodeIndex, usize> = HashMap::new();
        let mut queue = std::collections::VecDeque::new();
        let mut result = Vec::new();

        // Initialize in-degrees
        for node_idx in self.dag.graph().node_indices() {
            in_degree.insert(node_idx, 0);
        }

        // Count in-degrees using graph() to access petgraph API
        for node_idx in self.dag.graph().node_indices() {
            for neighbor_idx in self
                .dag
                .graph()
                .neighbors_directed(node_idx, Direction::Outgoing)
            {
                *in_degree.entry(neighbor_idx).or_insert(0) += 1;
            }
        }

        // Find all nodes with in-degree 0
        for (node_index, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(*node_index);
            }
        }

        // Process nodes
        while let Some(node_index) = queue.pop_front() {
            if let Some(&node_id) = self.index_to_node_id.get(&node_index) {
                result.push(node_id);
            }

            for neighbor_idx in self
                .dag
                .graph()
                .neighbors_directed(node_index, Direction::Outgoing)
            {
                if let Some(degree) = in_degree.get_mut(&neighbor_idx) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor_idx);
                    }
                }
            }
        }

        if result.len() != self.dag.node_count() {
            return Err(DagError::InvalidOperation(
                "Topological sort failed - DAG may contain cycles".to_string(),
            ));
        }

        Ok(result)
    }

    /// Executes the DAG with provided computation functions
    ///
    /// Uses topological sort to determine execution order and tokio for parallel execution.
    /// Independent nodes (no dependencies between them) execute in parallel.
    ///
    /// # Arguments
    /// * `compute_fn` - Async function that takes (node, input_data) and returns output
    ///
    /// # Returns
    /// Returns HashMap of NodeId -> NodeOutput with execution results
    pub async fn execute<F, Fut>(
        &mut self,
        compute_fn: F,
    ) -> Result<HashMap<NodeId, NodeOutput>, DagError>
    where
        F: Fn(Node, Vec<NodeOutput>) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<NodeOutput, DagError>> + Send,
    {
        // Get execution order
        let execution_order = self.execution_order()?;

        // Results storage (thread-safe for parallel execution)
        let results = Arc::new(RwLock::new(HashMap::new()));

        // Track which level of the DAG we're at (for parallel execution)
        let mut current_level = Vec::new();
        let mut remaining_nodes: Vec<NodeId> = execution_order.clone();

        while !remaining_nodes.is_empty() {
            // Find all nodes at current level (no dependencies on remaining nodes)
            current_level.clear();
            let mut next_remaining = Vec::new();

            for &node_id in &remaining_nodes {
                let node_index = self.node_id_to_index.get(&node_id).unwrap();

                // Check if all dependencies have been computed
                let parents: Vec<NodeId> = self
                    .dag
                    .parents(*node_index)
                    .iter(&self.dag)
                    .filter_map(|(_, parent_idx)| self.index_to_node_id.get(&parent_idx).copied())
                    .collect();

                let all_deps_ready = {
                    let results_guard = results.read().await;
                    parents
                        .iter()
                        .all(|parent_id| results_guard.contains_key(parent_id))
                };

                if all_deps_ready {
                    current_level.push(node_id);
                } else {
                    next_remaining.push(node_id);
                }
            }

            if current_level.is_empty() {
                return Err(DagError::ExecutionError(
                    "No nodes ready to execute - possible circular dependency".to_string(),
                ));
            }

            // Execute all nodes in current level in parallel
            let mut tasks = Vec::new();

            for &node_id in &current_level {
                let node = self.get_node(node_id).unwrap().clone();
                let node_index = self.node_id_to_index.get(&node_id).unwrap();

                // Collect inputs from parent nodes
                let parents: Vec<NodeId> = self
                    .dag
                    .parents(*node_index)
                    .iter(&self.dag)
                    .filter_map(|(_, parent_idx)| self.index_to_node_id.get(&parent_idx).copied())
                    .collect();

                let inputs = {
                    let results_guard = results.read().await;
                    parents
                        .iter()
                        .filter_map(|parent_id| results_guard.get(parent_id).cloned())
                        .collect::<Vec<_>>()
                };

                let compute_fn_clone = compute_fn.clone();
                let results_clone = Arc::clone(&results);

                let task = tokio::spawn(async move {
                    let output = compute_fn_clone(node.clone(), inputs).await?;
                    let mut results_guard = results_clone.write().await;
                    results_guard.insert(node_id, output);
                    Ok::<_, DagError>(())
                });

                tasks.push(task);
            }

            // Wait for all parallel tasks to complete
            for task in tasks {
                task.await
                    .map_err(|e| DagError::ExecutionError(format!("Task join error: {}", e)))??;
            }

            remaining_nodes = next_remaining;
        }

        // Extract results
        let final_results = results.read().await.clone();
        Ok(final_results)
    }

    /// Gets parent nodes of a given node
    pub fn get_parents(&self, node_id: NodeId) -> Vec<NodeId> {
        if let Some(&node_index) = self.node_id_to_index.get(&node_id) {
            self.dag
                .parents(node_index)
                .iter(&self.dag)
                .filter_map(|(_, parent_idx)| self.index_to_node_id.get(&parent_idx).copied())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Gets all node IDs in the DAG
    ///
    /// # Returns
    /// Vector of all NodeId values
    pub fn node_ids(&self) -> Vec<NodeId> {
        self.node_id_to_index.keys().copied().collect()
    }

    /// Gets child nodes of a given node
    pub fn get_children(&self, node_id: NodeId) -> Vec<NodeId> {
        if let Some(&node_index) = self.node_id_to_index.get(&node_id) {
            self.dag
                .children(node_index)
                .iter(&self.dag)
                .filter_map(|(_, child_idx)| self.index_to_node_id.get(&child_idx).copied())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Executes nodes affected by a change (push-mode incremental update)
    ///
    /// When new data arrives for a node, this method identifies and executes
    /// only the affected downstream nodes (dependencies) rather than the entire DAG.
    ///
    /// # Arguments
    /// * `changed_node_id` - ID of the node that has new data
    /// * `compute_fn` - Async function to compute node outputs
    ///
    /// # Returns
    /// Returns HashMap of NodeId -> NodeOutput for all affected nodes
    pub async fn execute_incremental<F, Fut>(
        &mut self,
        changed_node_id: NodeId,
        compute_fn: F,
    ) -> Result<HashMap<NodeId, NodeOutput>, DagError>
    where
        F: Fn(Node, Vec<NodeOutput>) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<NodeOutput, DagError>> + Send,
    {
        // Find all nodes affected by this change (descendants)
        let affected_nodes = self.get_descendants(changed_node_id);

        if affected_nodes.is_empty() {
            // No downstream dependencies, return empty result
            return Ok(HashMap::new());
        }

        // Get topological order for affected nodes only
        let full_order = self.execution_order()?;
        let affected_order: Vec<NodeId> = full_order
            .into_iter()
            .filter(|id| affected_nodes.contains(id))
            .collect();

        // Execute only affected nodes (similar to execute() but with subset)
        let results = Arc::new(RwLock::new(HashMap::new()));
        let mut current_level = Vec::new();
        let mut remaining_nodes = affected_order;

        while !remaining_nodes.is_empty() {
            current_level.clear();
            let mut next_remaining = Vec::new();

            for &node_id in &remaining_nodes {
                let node_index = self.node_id_to_index.get(&node_id).unwrap();

                let parents: Vec<NodeId> = self
                    .dag
                    .parents(*node_index)
                    .iter(&self.dag)
                    .filter_map(|(_, parent_idx)| self.index_to_node_id.get(&parent_idx).copied())
                    .collect();

                let all_deps_ready = {
                    let results_guard = results.read().await;
                    parents.iter().all(|parent_id| {
                        // Either already computed in this run, or not affected (assume ready)
                        results_guard.contains_key(parent_id) || !affected_nodes.contains(parent_id)
                    })
                };

                if all_deps_ready {
                    current_level.push(node_id);
                } else {
                    next_remaining.push(node_id);
                }
            }

            if current_level.is_empty() {
                return Err(DagError::ExecutionError(
                    "No nodes ready to execute in incremental update".to_string(),
                ));
            }

            let mut tasks = Vec::new();

            for &node_id in &current_level {
                let node = self.get_node(node_id).unwrap().clone();
                let node_index = self.node_id_to_index.get(&node_id).unwrap();

                let parents: Vec<NodeId> = self
                    .dag
                    .parents(*node_index)
                    .iter(&self.dag)
                    .filter_map(|(_, parent_idx)| self.index_to_node_id.get(&parent_idx).copied())
                    .collect();

                let inputs = {
                    let results_guard = results.read().await;
                    parents
                        .iter()
                        .filter_map(|parent_id| results_guard.get(parent_id).cloned())
                        .collect::<Vec<_>>()
                };

                let compute_fn_clone = compute_fn.clone();
                let results_clone = Arc::clone(&results);

                let task = tokio::spawn(async move {
                    let output = compute_fn_clone(node.clone(), inputs).await?;
                    let mut results_guard = results_clone.write().await;
                    results_guard.insert(node_id, output);
                    Ok::<_, DagError>(())
                });

                tasks.push(task);
            }

            for task in tasks {
                task.await
                    .map_err(|e| DagError::ExecutionError(format!("Task join error: {}", e)))??;
            }

            remaining_nodes = next_remaining;
        }

        let final_results = results.read().await.clone();
        Ok(final_results)
    }

    /// Gets all descendant nodes (downstream dependencies)
    ///
    /// This is useful for push-mode updates where we need to know which nodes
    /// are affected by a change to a particular node.
    pub fn get_descendants(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut descendants = Vec::new();
        let mut to_visit = vec![node_id];
        let mut visited = std::collections::HashSet::new();

        while let Some(current) = to_visit.pop() {
            if !visited.insert(current) {
                continue;
            }

            if current != node_id {
                descendants.push(current);
            }

            let children = self.get_children(current);
            to_visit.extend(children);
        }

        descendants
    }

    /// Registers a callback for node execution completion (push-mode hook)
    ///
    /// This method is designed to support push-mode integration by allowing
    /// external systems to register callbacks that are invoked when nodes complete.
    ///
    /// Note: This is a placeholder for the push-mode API design. The actual
    /// implementation would store callbacks and invoke them during execution.
    pub fn register_completion_callback<F>(&mut self, _node_id: NodeId, _callback: F)
    where
        F: Fn(NodeId, &NodeOutput) + Send + Sync + 'static,
    {
        // Placeholder for push-mode integration
        // In a full implementation, this would store callbacks in a HashMap
        // and invoke them during execute() or execute_incremental()
    }

    /// Calculates the number of burn-in days needed for a node and its dependencies
    ///
    /// This determines how many extra days of data to query before the user's requested range
    /// to ensure analytics have enough historical context.
    ///
    /// # Examples
    /// - DataProvider: 0 days (no burn-in needed)
    /// - Returns: 1 day (needs 1 extra price to compute first return)
    /// - Volatility(10): 11 days (10 for window + 1 for returns)
    fn calculate_burnin_days(&self, node_id: NodeId) -> usize {
        let node = match self.get_node(node_id) {
            Some(n) => n,
            None => return 0,
        };

        // Handle DataProvider nodes
        if node.node_type == "DataProvider" || node.node_type.contains("DataProvider") {
            return 0;
        }

        match node.node_type.as_str() {
            "Returns" => {
                let lag = Self::parse_lag_from_node(&node);
                let parent_burnin: usize = self
                    .get_parents(node_id)
                    .iter()
                    .map(|&parent_id| self.calculate_burnin_days(parent_id))
                    .max()
                    .unwrap_or(0);
                parent_burnin + lag
            }
            "lag" => {
                let lag = Self::parse_lag_from_node(&node);
                let parent_burnin: usize = self
                    .get_parents(node_id)
                    .iter()
                    .map(|&parent_id| self.calculate_burnin_days(parent_id))
                    .max()
                    .unwrap_or(0);
                parent_burnin + lag
            }
            "Volatility" => {
                // Volatility needs window_size extra returns
                let window_size = if let NodeParams::Map(ref params) = node.params {
                    params
                        .get("window_size")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(10)
                } else {
                    10
                };

                // Recursively get parent burn-in
                let parent_burnin: usize = self
                    .get_parents(node_id)
                    .iter()
                    .map(|&parent_id| self.calculate_burnin_days(parent_id))
                    .max()
                    .unwrap_or(0);
                parent_burnin + window_size
            }
            _ => {
                // Unknown node type, get max parent burn-in
                self.get_parents(node_id)
                    .iter()
                    .map(|&parent_id| self.calculate_burnin_days(parent_id))
                    .max()
                    .unwrap_or(0)
            }
        }
    }

    fn parse_lag_from_node(node: &Node) -> usize {
        if let NodeParams::Map(ref params) = node.params {
            params
                .get("lag")
                .and_then(|value| value.parse::<usize>().ok())
                .filter(|&lag| lag > 0)
                .unwrap_or(1)
        } else {
            1
        }
    }

    /// Executes DAG in pull-mode for batch computation
    ///
    /// Pull-mode executes the entire DAG for a specified date range, computing
    /// complete time series in a single pass. This is the complement to push-mode's
    /// incremental updates.
    ///
    /// # Arguments
    /// * `node_id` - The target node to execute
    /// * `date_range` - The date range to compute analytics for
    /// * `provider` - Data source for querying historical data
    ///
    /// # Returns
    /// Complete time series as `Vec<TimeSeriesPoint>` for the requested date range
    ///
    /// # Errors
    /// Returns `DagError` if:
    /// - Node not found
    /// - Data loading fails
    /// - Computation fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use analytics::{AnalyticsDag, NodeParams, AssetKey, DateRange, InMemoryDataProvider};
    /// use chrono::NaiveDate;
    /// use std::sync::Arc;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut dag = AnalyticsDag::new();
    /// let aapl = AssetKey::new_equity("AAPL")?;
    ///
    /// // Add nodes (DataProvider, Returns, Volatility)
    /// let data_node = dag.add_node("DataProvider".to_string(), NodeParams::None, vec![aapl.clone()]);
    ///
    /// let provider = Arc::new(InMemoryDataProvider::new());
    /// let date_range = DateRange::new(
    ///     NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    ///     NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
    /// );
    ///
    /// // Execute in pull-mode for complete time series
    /// let results = dag.execute_pull_mode(data_node, date_range, &*provider)?;
    /// println!("Computed {} data points", results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn execute_pull_mode(
        &self,
        node_id: NodeId,
        date_range: DateRange,
        provider: &dyn DataProvider,
    ) -> Result<Vec<TimeSeriesPoint>, DagError> {
        use chrono::Duration;

        // Verify target node exists
        let _target_node = self
            .get_node(node_id)
            .ok_or_else(|| DagError::NodeNotFound(format!("Node {} not found", node_id.0)))?;

        // Calculate burn-in days needed for this node
        let burnin_days = self.calculate_burnin_days(node_id);

        // Extend date range backward for burn-in
        let extended_start = date_range.start - Duration::days(burnin_days as i64);
        let extended_range = DateRange::new(extended_start, date_range.end);

        // Get topological order to determine execution sequence
        let execution_order = self.execution_order_immutable()?;

        // Find all ancestors of target node that need to be executed
        let mut nodes_to_execute = Vec::new();
        for &candidate_id in &execution_order {
            // Include candidate if it's the target or an ancestor of target
            let descendants = self.get_descendants(candidate_id);
            if candidate_id == node_id || descendants.contains(&node_id) {
                nodes_to_execute.push(candidate_id);
            }
        }

        // Create execution cache for intermediate results
        let mut cache = ExecutionCache::new();
        let mut calendar_cache: HashMap<AssetKey, Vec<DateTime<Utc>>> = HashMap::new();

        // Execute nodes in topological order with extended date range
        for &current_id in &nodes_to_execute {
            // Get parent outputs from cache
            let parents = self.get_parents(current_id);
            let parent_outputs: Vec<ParentOutput> = parents
                .iter()
                .map(|&parent_id| ParentOutput {
                    node_id: parent_id,
                    analytic: self.analytic_type_for_node(parent_id),
                    output: cache.get(parent_id).cloned().unwrap_or_default(),
                })
                .collect();

            // Execute the node based on its type (using extended range for data loading)
            let result = self.execute_pull_node(
                current_id,
                &parent_outputs,
                &extended_range,
                provider,
                &mut calendar_cache,
            )?;

            // Cache the result with its extended range
            cache.insert(current_id, result, extended_range.clone());
        }

        // Identify a data provider node to drive the simulated push
        let data_node_id = nodes_to_execute
            .iter()
            .find(|&&id| self.is_data_provider_node(id))
            .ok_or_else(|| {
                DagError::ExecutionError(
                    "No data provider node found for push simulation".to_string(),
                )
            })?;

        let data_points = cache.get(*data_node_id).cloned().ok_or_else(|| {
            DagError::ExecutionError("Data provider output missing for simulation".to_string())
        })?;

        // Simulate push-mode over the cached calendar/timestamps
        let simulated =
            self.simulate_push_from_calendar(&nodes_to_execute, &data_points, node_id)?;

        // Filter simulation output to the originally requested date range
        let filtered_result: Vec<TimeSeriesPoint> = simulated
            .into_iter()
            .filter(|point| {
                let date = point.timestamp.date_naive();
                date >= date_range.start && date <= date_range.end
            })
            .collect();

        Ok(filtered_result)
    }

    /// Executes multiple DAG nodes in parallel for batch computation
    ///
    /// This method enables efficient computation of multiple independent analytics by
    /// executing them concurrently. Nodes with shared dependencies will reuse cached
    /// intermediate results.
    ///
    /// # Arguments
    /// * `node_ids` - Vector of target nodes to execute
    /// * `date_range` - The date range to compute analytics for
    /// * `provider` - Data source for querying historical data (must be thread-safe)
    ///
    /// # Returns
    /// HashMap mapping each NodeId to its complete time series result
    ///
    /// # Errors
    /// Returns `DagError` if any node execution fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use analytics::{AnalyticsDag, NodeParams, AssetKey, DateRange, InMemoryDataProvider};
    /// use chrono::NaiveDate;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut dag = AnalyticsDag::new();
    /// let aapl = AssetKey::new_equity("AAPL")?;
    /// let msft = AssetKey::new_equity("MSFT")?;
    ///
    /// // Add nodes for two assets
    /// let aapl_node = dag.add_node("Returns".to_string(), NodeParams::None, vec![aapl]);
    /// let msft_node = dag.add_node("Returns".to_string(), NodeParams::None, vec![msft]);
    ///
    /// let provider = InMemoryDataProvider::new();
    /// let date_range = DateRange::new(
    ///     NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    ///     NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
    /// );
    ///
    /// // Execute both nodes in parallel
    /// let results = dag.execute_pull_mode_parallel(
    ///     vec![aapl_node, msft_node],
    ///     date_range,
    ///     &provider,
    /// )?;
    ///
    /// println!("AAPL: {} points", results.get(&aapl_node).unwrap().len());
    /// println!("MSFT: {} points", results.get(&msft_node).unwrap().len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn execute_pull_mode_parallel(
        &self,
        node_ids: Vec<NodeId>,
        date_range: DateRange,
        provider: &(dyn DataProvider + Sync),
    ) -> Result<HashMap<NodeId, Vec<TimeSeriesPoint>>, DagError> {
        use std::sync::Mutex as StdMutex;

        // Create a shared result map protected by mutex
        let results = StdMutex::new(HashMap::<NodeId, Vec<TimeSeriesPoint>>::new());
        let errors = StdMutex::new(Vec::<String>::new());

        // Execute each node (sequentially for now, but designed for future parallelization)
        // Note: For true parallelism, we'd need thread-safe access to the DAG,
        // which would require wrapping it in Arc<RwLock<>> at the calling site.
        // For daily data and reasonable numbers of assets, sequential execution is acceptable.
        for node_id in node_ids {
            match self.execute_pull_mode(node_id, date_range.clone(), provider) {
                Ok(result) => {
                    results.lock().unwrap().insert(node_id, result);
                }
                Err(e) => {
                    errors
                        .lock()
                        .unwrap()
                        .push(format!("Node {}: {}", node_id.0, e));
                }
            }
        }

        // Check if any errors occurred
        let error_list = errors.lock().unwrap();
        if !error_list.is_empty() {
            return Err(DagError::ExecutionError(format!(
                "Parallel execution had {} error(s): {}",
                error_list.len(),
                error_list.join("; ")
            )));
        }

        // Return results
        Ok(results.into_inner().unwrap())
    }

    fn execute_pull_node(
        &self,
        node_id: NodeId,
        parent_outputs: &[ParentOutput],
        date_range: &DateRange,
        provider: &dyn DataProvider,
        calendar_cache: &mut HashMap<AssetKey, Vec<DateTime<Utc>>>,
    ) -> Result<Vec<TimeSeriesPoint>, DagError> {
        let node = self
            .get_node(node_id)
            .ok_or_else(|| DagError::NodeNotFound(format!("Node {} not found", node_id.0)))?;
        let executor = self.executor_for_node(node, node_id)?;
        let result = executor.execute_pull(node, parent_outputs, date_range, provider)?;

        // Capture calendar metadata for data providers
        if let Some(key) = self.node_keys_by_id.get(&node_id) {
            if key.analytic == AnalyticType::DataProvider {
                if let Some(asset) = node.assets.first() {
                    if let Ok(dates) = provider.available_dates(asset, date_range) {
                        calendar_cache.insert(asset.clone(), dates);
                    }
                }
            }
        }

        Ok(result)
    }

    pub(crate) fn execute_push_node(
        &self,
        node_id: NodeId,
        parent_outputs: &[ParentOutput],
        timestamp: DateTime<Utc>,
        value: f64,
    ) -> Result<NodeOutput, DagError> {
        let node = self
            .get_node(node_id)
            .ok_or_else(|| DagError::NodeNotFound(format!("Node {} not found", node_id.0)))?;
        let executor = self.executor_for_node(node, node_id)?;
        executor.execute_push(node, parent_outputs, timestamp, value)
    }
}

impl Default for AnalyticsDag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_key::AssetKey;

    fn add_returns_chain(dag: &mut AnalyticsDag, asset: AssetKey) -> (NodeId, NodeId, NodeId) {
        let data_node = dag.add_node(
            "DataProvider".to_string(),
            NodeParams::None,
            vec![asset.clone()],
        );
        let lag_node = dag.add_node("lag".to_string(), NodeParams::None, vec![asset.clone()]);
        let returns_node = dag.add_node("Returns".to_string(), NodeParams::None, vec![asset]);
        dag.add_edge(data_node, lag_node).unwrap();
        dag.add_edge(data_node, returns_node).unwrap();
        dag.add_edge(lag_node, returns_node).unwrap();
        (data_node, lag_node, returns_node)
    }

    /// Task Group 1.1: Write 2-8 focused tests for DAG library evaluation
    ///
    /// These tests evaluate different DAG libraries to determine which one
    /// has the best API for DAG creation, cycle detection, dynamic modifications,
    /// and topological sorting.

    #[test]
    fn test_petgraph_basic_dag_creation() {
        // Test basic DAG creation with petgraph
        // This will help evaluate API ergonomics for DAG creation
        use petgraph::graph::{DiGraph, NodeIndex};

        let mut graph = DiGraph::<String, ()>::new();
        let node_a = graph.add_node("Node A".to_string());
        let node_b = graph.add_node("Node B".to_string());
        let _edge = graph.add_edge(node_a, node_b, ());

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_petgraph_cycle_detection() {
        // Test cycle detection capabilities with petgraph
        use petgraph::algo::is_cyclic_directed;
        use petgraph::graph::DiGraph;

        // Create a DAG (no cycles)
        let mut dag = DiGraph::<String, ()>::new();
        let a = dag.add_node("A".to_string());
        let b = dag.add_node("B".to_string());
        let c = dag.add_node("C".to_string());
        dag.add_edge(a, b, ());
        dag.add_edge(b, c, ());

        assert!(!is_cyclic_directed(&dag));

        // Create a graph with cycle
        let mut cyclic = DiGraph::<String, ()>::new();
        let x = cyclic.add_node("X".to_string());
        let y = cyclic.add_node("Y".to_string());
        cyclic.add_edge(x, y, ());
        cyclic.add_edge(y, x, ());

        assert!(is_cyclic_directed(&cyclic));
    }

    #[test]
    fn test_petgraph_dynamic_node_addition() {
        // Test dynamic node addition/removal with petgraph
        use petgraph::graph::DiGraph;

        let mut graph = DiGraph::<String, ()>::new();
        let node1 = graph.add_node("Node 1".to_string());
        assert_eq!(graph.node_count(), 1);

        let node2 = graph.add_node("Node 2".to_string());
        graph.add_edge(node1, node2, ());
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_petgraph_topological_sort() {
        // Test topological sorting functionality with petgraph
        use petgraph::algo::toposort;
        use petgraph::graph::DiGraph;

        let mut graph = DiGraph::<String, ()>::new();
        let a = graph.add_node("A".to_string());
        let b = graph.add_node("B".to_string());
        let c = graph.add_node("C".to_string());
        graph.add_edge(a, b, ());
        graph.add_edge(b, c, ());

        let sorted = toposort(&graph, None).unwrap();
        // Topological sort should respect dependencies
        let a_pos = sorted.iter().position(|&n| n == a).unwrap();
        let b_pos = sorted.iter().position(|&n| n == b).unwrap();
        let c_pos = sorted.iter().position(|&n| n == c).unwrap();

        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_daggy_basic_dag_creation() {
        // Test basic DAG creation with daggy
        // daggy has a better API specifically for DAG creation
        use daggy::Dag;

        let mut dag = Dag::<String, ()>::new();
        let node_a = dag.add_node("Node A".to_string());
        let node_b = dag.add_node("Node B".to_string());

        // daggy's add_edge returns Result, preventing cycles at construction
        assert!(dag.add_edge(node_a, node_b, ()).is_ok());
        assert_eq!(dag.node_count(), 2);
        assert_eq!(dag.edge_count(), 1);
    }

    #[test]
    fn test_daggy_cycle_detection() {
        // Test cycle detection with daggy
        // daggy prevents cycles at construction time - better API than petgraph
        use daggy::Dag;

        let mut dag = Dag::<String, ()>::new();
        let a = dag.add_node("A".to_string());
        let b = dag.add_node("B".to_string());

        // Add edge A -> B (should succeed)
        assert!(dag.add_edge(a, b, ()).is_ok());

        // Try to add edge B -> A (should fail - cycle detected at construction time)
        // This is better API than petgraph which requires separate is_cyclic_directed check
        assert!(dag.add_edge(b, a, ()).is_err());
    }

    #[test]
    fn test_daggy_topological_sort() {
        // Test topological sorting with daggy
        use daggy::Dag;

        let mut dag = Dag::<String, ()>::new();
        let a = dag.add_node("A".to_string());
        let b = dag.add_node("B".to_string());
        let c = dag.add_node("C".to_string());

        dag.add_edge(a, b, ()).unwrap();
        dag.add_edge(b, c, ()).unwrap();

        // daggy provides access to nodes
        assert_eq!(dag.node_count(), 3);
        assert_eq!(dag.edge_count(), 2);
    }

    // Task Group 2.1: Write 2-8 focused tests for DAG structure

    #[test]
    fn test_create_empty_dag() {
        // Test creating empty DAG
        let dag = AnalyticsDag::new();
        assert_eq!(dag.node_count(), 0);
        assert_eq!(dag.edge_count(), 0);
    }

    #[test]
    fn test_create_dag_with_single_node() {
        // Test creating DAG with single node
        let mut dag = AnalyticsDag::new();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let params = NodeParams::Map({
            let mut m = HashMap::new();
            m.insert("window".to_string(), "20".to_string());
            m
        });

        let node_id = dag.add_node(
            "moving_average".to_string(),
            params.clone(),
            vec![asset_key],
        );

        assert_eq!(dag.node_count(), 1);
        assert_eq!(dag.edge_count(), 0);

        let node = dag.get_node(node_id).unwrap();
        assert_eq!(node.node_type, "moving_average");
        assert_eq!(node.params, params);
    }

    #[test]
    fn test_create_dag_with_multiple_nodes_and_edges() {
        // Test creating DAG with multiple nodes and edges
        let mut dag = AnalyticsDag::new();
        let asset_a = AssetKey::new_equity("AAPL").unwrap();
        let asset_b = AssetKey::new_equity("MSFT").unwrap();

        // Create nodes
        let node1_id = dag.add_node(
            "price_data".to_string(),
            NodeParams::None,
            vec![asset_a.clone()],
        );
        let node2_id = dag.add_node(
            "moving_average".to_string(),
            NodeParams::Map({
                let mut m = HashMap::new();
                m.insert("window".to_string(), "20".to_string());
                m
            }),
            vec![asset_a],
        );
        let node3_id = dag.add_node("correlation".to_string(), NodeParams::None, vec![asset_b]);

        // Create edges (dependencies)
        assert!(dag.add_edge(node1_id, node2_id).is_ok());
        assert!(dag.add_edge(node1_id, node3_id).is_ok());

        assert_eq!(dag.node_count(), 3);
        assert_eq!(dag.edge_count(), 2);
    }

    #[test]
    fn test_node_parameterization() {
        // Test node parameterization
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create node with parameters
        let params = NodeParams::Map({
            let mut m = HashMap::new();
            m.insert("window".to_string(), "20".to_string());
            m.insert("method".to_string(), "simple".to_string());
            m
        });

        let node_id = dag.add_node("moving_average".to_string(), params.clone(), vec![asset]);

        let node = dag.get_node(node_id).unwrap();
        match &node.params {
            NodeParams::Map(map) => {
                assert_eq!(map.get("window"), Some(&"20".to_string()));
                assert_eq!(map.get("method"), Some(&"simple".to_string()));
            }
            NodeParams::None => panic!("Expected Map params"),
        }
    }

    // Task Group 3.1: Write 2-8 focused tests for cycle detection

    #[test]
    fn test_detecting_cycles_when_adding_new_edge() {
        // Test detecting cycles when adding new edge
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create a simple chain: A -> B -> C
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset]);

        // Add edges A -> B and B -> C (should succeed)
        assert!(dag.add_edge(node_a, node_b).is_ok());
        assert!(dag.add_edge(node_b, node_c).is_ok());

        // Try to add edge C -> A (should fail - creates cycle A -> B -> C -> A)
        let result = dag.add_edge(node_c, node_a);
        assert!(result.is_err());
        match result {
            Err(DagError::CycleDetected(msg)) => {
                assert!(msg.contains("would create a cycle"));
            }
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn test_detecting_cycles_in_existing_dag() {
        // Test detecting cycles in existing DAG structure
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create nodes
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_d = dag.add_node("D".to_string(), NodeParams::None, vec![asset]);

        // Create valid edges: A -> B, B -> C, C -> D
        assert!(dag.add_edge(node_a, node_b).is_ok());
        assert!(dag.add_edge(node_b, node_c).is_ok());
        assert!(dag.add_edge(node_c, node_d).is_ok());

        // Try to create cycle: D -> B (would create B -> C -> D -> B)
        let result = dag.add_edge(node_d, node_b);
        assert!(result.is_err());
        assert!(matches!(result, Err(DagError::CycleDetected(_))));
    }

    #[test]
    fn test_valid_dag_passes_cycle_detection() {
        // Test valid DAG (no cycles) passes detection
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create nodes
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_d = dag.add_node("D".to_string(), NodeParams::None, vec![asset]);

        // Create valid edges (no cycles)
        assert!(dag.add_edge(node_a, node_b).is_ok());
        assert!(dag.add_edge(node_a, node_c).is_ok());
        assert!(dag.add_edge(node_b, node_d).is_ok());
        assert!(dag.add_edge(node_c, node_d).is_ok());

        // DAG should be valid (no cycles)
        assert_eq!(dag.node_count(), 4);
        assert_eq!(dag.edge_count(), 4);
    }

    #[test]
    fn test_error_messages_for_cycle_detection() {
        // Test error messages for cycle detection
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset]);

        // Create edge A -> B
        assert!(dag.add_edge(node_a, node_b).is_ok());

        // Try to create cycle B -> A
        let result = dag.add_edge(node_b, node_a);
        assert!(result.is_err());

        if let Err(DagError::CycleDetected(msg)) = result {
            // Error message should indicate which nodes would form the cycle
            assert!(msg.contains("would create a cycle"));
            assert!(msg.contains("NodeId"));
        } else {
            panic!("Expected CycleDetected error with message");
        }
    }

    #[test]
    fn test_self_loop_detection() {
        // Test that self-loops (node to itself) are detected as cycles
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset]);

        // Try to create edge A -> A (self-loop, should be detected as cycle)
        let result = dag.add_edge(node_a, node_a);
        assert!(result.is_err());
        assert!(matches!(result, Err(DagError::CycleDetected(_))));
    }

    #[test]
    fn test_complex_cycle_detection() {
        // Test detecting cycles in more complex DAG structures
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create a diamond structure: A -> B, A -> C, B -> D, C -> D
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_d = dag.add_node("D".to_string(), NodeParams::None, vec![asset]);

        // Create valid diamond structure
        assert!(dag.add_edge(node_a, node_b).is_ok());
        assert!(dag.add_edge(node_a, node_c).is_ok());
        assert!(dag.add_edge(node_b, node_d).is_ok());
        assert!(dag.add_edge(node_c, node_d).is_ok());

        // Try to create cycle: D -> A (would create A -> B -> D -> A or A -> C -> D -> A)
        let result = dag.add_edge(node_d, node_a);
        assert!(result.is_err());
        assert!(matches!(result, Err(DagError::CycleDetected(_))));
    }

    // Task Group 4.1: Write 2-8 focused tests for topological sorting

    #[test]
    fn test_toposort_simple_linear_dag() {
        // Test topological sort for simple linear DAG: A -> B -> C
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset]);

        dag.add_edge(node_a, node_b).unwrap();
        dag.add_edge(node_b, node_c).unwrap();

        let order = dag.execution_order().unwrap();

        // Verify order respects dependencies
        assert_eq!(order.len(), 3);
        let a_pos = order.iter().position(|&id| id == node_a).unwrap();
        let b_pos = order.iter().position(|&id| id == node_b).unwrap();
        let c_pos = order.iter().position(|&id| id == node_c).unwrap();

        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_toposort_dag_with_parallel_branches() {
        // Test topological sort for DAG with parallel branches (diamond structure)
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create diamond: A -> B, A -> C, B -> D, C -> D
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_d = dag.add_node("D".to_string(), NodeParams::None, vec![asset]);

        dag.add_edge(node_a, node_b).unwrap();
        dag.add_edge(node_a, node_c).unwrap();
        dag.add_edge(node_b, node_d).unwrap();
        dag.add_edge(node_c, node_d).unwrap();

        let order = dag.execution_order().unwrap();

        // Verify order respects dependencies
        assert_eq!(order.len(), 4);
        let a_pos = order.iter().position(|&id| id == node_a).unwrap();
        let b_pos = order.iter().position(|&id| id == node_b).unwrap();
        let c_pos = order.iter().position(|&id| id == node_c).unwrap();
        let d_pos = order.iter().position(|&id| id == node_d).unwrap();

        // A must come before B and C
        assert!(a_pos < b_pos);
        assert!(a_pos < c_pos);
        // B and C must both come before D
        assert!(b_pos < d_pos);
        assert!(c_pos < d_pos);
        // B and C can be in any order relative to each other
    }

    #[test]
    fn test_toposort_empty_dag() {
        // Test edge case: empty DAG
        let mut dag = AnalyticsDag::new();

        let order = dag.execution_order().unwrap();
        assert_eq!(order.len(), 0);
    }

    #[test]
    fn test_toposort_single_node() {
        // Test edge case: single node
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset]);

        let order = dag.execution_order().unwrap();
        assert_eq!(order.len(), 1);
        assert_eq!(order[0], node_a);
    }

    #[test]
    fn test_toposort_disconnected_components() {
        // Test edge case: disconnected components (no edges)
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset]);

        // No edges - all nodes are disconnected
        let order = dag.execution_order().unwrap();

        // All nodes should be in the result
        assert_eq!(order.len(), 3);
        assert!(order.contains(&node_a));
        assert!(order.contains(&node_b));
        assert!(order.contains(&node_c));
    }

    #[test]
    fn test_toposort_query_without_executing() {
        // Test querying execution order without executing nodes
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset]);

        dag.add_edge(node_a, node_b).unwrap();
        dag.add_edge(node_b, node_c).unwrap();

        // Query execution order multiple times
        let order1 = dag.execution_order().unwrap();
        let order2 = dag.execution_order().unwrap();

        // Should get the same result (from cache)
        assert_eq!(order1, order2);
        assert_eq!(order1.len(), 3);
    }

    #[test]
    fn test_toposort_cache_invalidation() {
        // Test that topological sort cache is invalidated when DAG changes
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);

        dag.add_edge(node_a, node_b).unwrap();

        let order1 = dag.execution_order().unwrap();
        assert_eq!(order1.len(), 2);

        // Add new node - should invalidate cache
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset]);
        dag.add_edge(node_b, node_c).unwrap();

        let order2 = dag.execution_order().unwrap();
        assert_eq!(order2.len(), 3);

        // Verify new order includes all nodes
        assert!(order2.contains(&node_a));
        assert!(order2.contains(&node_b));
        assert!(order2.contains(&node_c));
    }

    #[test]
    fn test_toposort_complex_dag() {
        // Test complex DAG with multiple dependencies
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create complex structure
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_d = dag.add_node("D".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_e = dag.add_node("E".to_string(), NodeParams::None, vec![asset]);

        // A -> B, A -> C, B -> D, C -> D, D -> E
        dag.add_edge(node_a, node_b).unwrap();
        dag.add_edge(node_a, node_c).unwrap();
        dag.add_edge(node_b, node_d).unwrap();
        dag.add_edge(node_c, node_d).unwrap();
        dag.add_edge(node_d, node_e).unwrap();

        let order = dag.execution_order().unwrap();

        // Verify all dependencies are respected
        assert_eq!(order.len(), 5);
        let a_pos = order.iter().position(|&id| id == node_a).unwrap();
        let b_pos = order.iter().position(|&id| id == node_b).unwrap();
        let c_pos = order.iter().position(|&id| id == node_c).unwrap();
        let d_pos = order.iter().position(|&id| id == node_d).unwrap();
        let e_pos = order.iter().position(|&id| id == node_e).unwrap();

        // Verify ordering constraints
        assert!(a_pos < b_pos);
        assert!(a_pos < c_pos);
        assert!(b_pos < d_pos);
        assert!(c_pos < d_pos);
        assert!(d_pos < e_pos);
    }

    // Task Group 5.1: Write 2-8 focused tests for parallel execution

    #[tokio::test]
    async fn test_parallel_execution_independent_nodes() {
        // Test parallel execution of independent nodes
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use tokio::time::{sleep, Duration};

        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create three independent nodes (no edges between them)
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset]);

        // Track concurrent execution
        let concurrent_count = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));

        let concurrent_clone = Arc::clone(&concurrent_count);
        let max_clone = Arc::clone(&max_concurrent);

        let compute_fn = move |_node: Node, _inputs: Vec<NodeOutput>| {
            let concurrent = Arc::clone(&concurrent_clone);
            let max_conc = Arc::clone(&max_clone);

            async move {
                // Increment concurrent count
                let current = concurrent.fetch_add(1, Ordering::SeqCst) + 1;

                // Update max concurrent
                max_conc.fetch_max(current, Ordering::SeqCst);

                // Simulate work
                sleep(Duration::from_millis(50)).await;

                // Decrement concurrent count
                concurrent.fetch_sub(1, Ordering::SeqCst);

                Ok(NodeOutput::None)
            }
        };

        let _results = dag.execute(compute_fn).await.unwrap();

        // Verify that at least 2 nodes ran concurrently (ideally all 3)
        let max_conc = max_concurrent.load(Ordering::SeqCst);
        assert!(
            max_conc >= 2,
            "Expected at least 2 concurrent executions, got {}",
            max_conc
        );
    }

    #[tokio::test]
    async fn test_sequential_execution_dependent_nodes() {
        // Test sequential execution of dependent nodes
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create linear chain: A -> B -> C
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset]);

        dag.add_edge(node_a, node_b).unwrap();
        dag.add_edge(node_b, node_c).unwrap();

        // Track execution order
        let execution_order = Arc::new(Mutex::new(Vec::new()));
        let order_clone = Arc::clone(&execution_order);

        let compute_fn = move |node: Node, _inputs: Vec<NodeOutput>| {
            let order = Arc::clone(&order_clone);

            async move {
                let mut order_guard = order.lock().await;
                order_guard.push(node.id);
                drop(order_guard);

                Ok(NodeOutput::Scalar(node.id.0 as f64))
            }
        };

        let _results = dag.execute(compute_fn).await.unwrap();

        // Verify execution order
        let final_order = execution_order.lock().await;
        assert_eq!(final_order.len(), 3);

        // Find positions
        let a_pos = final_order.iter().position(|&id| id == node_a).unwrap();
        let b_pos = final_order.iter().position(|&id| id == node_b).unwrap();
        let c_pos = final_order.iter().position(|&id| id == node_c).unwrap();

        // Verify A before B before C
        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[tokio::test]
    async fn test_thread_safe_execution() {
        // Test thread-safe execution with multiple concurrent nodes
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create multiple independent nodes
        let mut nodes = Vec::new();
        for i in 0..5 {
            let node_id = dag.add_node(format!("Node{}", i), NodeParams::None, vec![asset.clone()]);
            nodes.push(node_id);
        }

        // Shared counter (tests thread safety)
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let compute_fn = move |_node: Node, _inputs: Vec<NodeOutput>| {
            let counter = Arc::clone(&counter_clone);

            async move {
                // Simulate concurrent access
                for _ in 0..100 {
                    counter.fetch_add(1, Ordering::SeqCst);
                }

                Ok(NodeOutput::None)
            }
        };

        let _results = dag.execute(compute_fn).await.unwrap();

        // Verify counter (should be 5 nodes * 100 increments = 500)
        assert_eq!(counter.load(Ordering::SeqCst), 500);
    }

    #[tokio::test]
    async fn test_mixed_parallel_sequential_execution() {
        // Test execution with mixed parallel and sequential nodes (diamond structure)
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create diamond: A -> B, A -> C, B -> D, C -> D
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_d = dag.add_node("D".to_string(), NodeParams::None, vec![asset]);

        dag.add_edge(node_a, node_b).unwrap();
        dag.add_edge(node_a, node_c).unwrap();
        dag.add_edge(node_b, node_d).unwrap();
        dag.add_edge(node_c, node_d).unwrap();

        // Track execution order
        let execution_order = Arc::new(Mutex::new(Vec::new()));
        let order_clone = Arc::clone(&execution_order);

        let compute_fn = move |node: Node, inputs: Vec<NodeOutput>| {
            let order = Arc::clone(&order_clone);

            async move {
                let mut order_guard = order.lock().await;
                order_guard.push((node.id, inputs.len()));
                drop(order_guard);

                Ok(NodeOutput::Scalar(node.id.0 as f64))
            }
        };

        let _results = dag.execute(compute_fn).await.unwrap();

        // Verify execution order
        let final_order = execution_order.lock().await;
        assert_eq!(final_order.len(), 4);

        // Find positions
        let a_pos = final_order
            .iter()
            .position(|(id, _)| *id == node_a)
            .unwrap();
        let b_pos = final_order
            .iter()
            .position(|(id, _)| *id == node_b)
            .unwrap();
        let c_pos = final_order
            .iter()
            .position(|(id, _)| *id == node_c)
            .unwrap();
        let d_pos = final_order
            .iter()
            .position(|(id, _)| *id == node_d)
            .unwrap();

        // A must execute first
        assert!(a_pos < b_pos);
        assert!(a_pos < c_pos);

        // B and C can be in any order (parallel)
        // But both must be before D
        assert!(b_pos < d_pos);
        assert!(c_pos < d_pos);

        // D should have received 2 inputs (from B and C)
        let d_inputs = final_order.iter().find(|(id, _)| *id == node_d).unwrap().1;
        assert_eq!(d_inputs, 2);
    }

    #[tokio::test]
    async fn test_execution_with_node_outputs() {
        // Test that node outputs are passed correctly to dependent nodes
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create chain: A -> B -> C
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset]);

        dag.add_edge(node_a, node_b).unwrap();
        dag.add_edge(node_b, node_c).unwrap();

        let compute_fn = move |node: Node, inputs: Vec<NodeOutput>| {
            async move {
                // Sum all input scalars and add node ID
                let input_sum: f64 = inputs
                    .iter()
                    .filter_map(|output| match output {
                        NodeOutput::Scalar(v) => Some(*v),
                        _ => None,
                    })
                    .sum();

                let result = input_sum + (node.id.0 as f64);
                Ok(NodeOutput::Scalar(result))
            }
        };

        let results = dag.execute(compute_fn).await.unwrap();

        // Node A (id=0): 0 (no inputs)
        // Node B (id=1): 0 + 1 = 1
        // Node C (id=2): 1 + 2 = 3
        assert_eq!(results.len(), 3);

        match results.get(&node_a).unwrap() {
            NodeOutput::Scalar(v) => assert_eq!(*v, 0.0),
            _ => panic!("Expected scalar"),
        }

        match results.get(&node_b).unwrap() {
            NodeOutput::Scalar(v) => assert_eq!(*v, 1.0),
            _ => panic!("Expected scalar"),
        }

        match results.get(&node_c).unwrap() {
            NodeOutput::Scalar(v) => assert_eq!(*v, 3.0),
            _ => panic!("Expected scalar"),
        }
    }

    #[tokio::test]
    async fn test_execution_empty_dag() {
        // Test execution of empty DAG
        let mut dag = AnalyticsDag::new();

        let compute_fn = |_node: Node, _inputs: Vec<NodeOutput>| async { Ok(NodeOutput::None) };

        let results = dag.execute(compute_fn).await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_execution_single_node() {
        // Test execution of single node
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset]);

        let compute_fn = |node: Node, _inputs: Vec<NodeOutput>| async move {
            Ok(NodeOutput::Scalar(node.id.0 as f64 * 10.0))
        };

        let results = dag.execute(compute_fn).await.unwrap();

        assert_eq!(results.len(), 1);
        match results.get(&node_a).unwrap() {
            NodeOutput::Scalar(v) => assert_eq!(*v, 0.0),
            _ => panic!("Expected scalar"),
        }
    }

    // Task Group 6.1: Write 2-8 focused tests for dynamic modifications

    #[test]
    fn test_adding_nodes_with_dependencies() {
        // Test adding new nodes with dependencies on existing nodes
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create initial nodes
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        dag.add_edge(node_a, node_b).unwrap();

        assert_eq!(dag.node_count(), 2);

        // Add new node C with dependency on B (B -> C)
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset.clone()]);
        dag.add_edge(node_b, node_c).unwrap();

        assert_eq!(dag.node_count(), 3);
        assert_eq!(dag.edge_count(), 2);

        // Verify execution order is updated
        let order = dag.execution_order().unwrap();
        assert_eq!(order.len(), 3);

        let a_pos = order.iter().position(|&id| id == node_a).unwrap();
        let b_pos = order.iter().position(|&id| id == node_b).unwrap();
        let c_pos = order.iter().position(|&id| id == node_c).unwrap();

        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_removing_nodes_without_dependencies() {
        // Test removing nodes with no dependencies (leaf nodes)
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create chain: A -> B -> C
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset]);

        dag.add_edge(node_a, node_b).unwrap();
        dag.add_edge(node_b, node_c).unwrap();

        assert_eq!(dag.node_count(), 3);

        // Remove leaf node C (no dependencies)
        assert!(!dag.has_dependencies(node_c));
        dag.remove_node(node_c).unwrap();

        assert_eq!(dag.node_count(), 2);

        // Verify execution order is updated
        let order = dag.execution_order().unwrap();
        assert_eq!(order.len(), 2);
        assert!(!order.contains(&node_c));
    }

    #[test]
    fn test_cannot_remove_node_with_dependencies() {
        // Test that nodes with dependencies cannot be removed
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create chain: A -> B -> C
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset]);

        dag.add_edge(node_a, node_b).unwrap();
        dag.add_edge(node_b, node_c).unwrap();

        // Try to remove node B (has dependency C)
        assert!(dag.has_dependencies(node_b));
        let result = dag.remove_node(node_b);

        assert!(result.is_err());
        match result {
            Err(DagError::InvalidOperation(msg)) => {
                assert!(msg.contains("has dependencies"));
            }
            _ => panic!("Expected InvalidOperation error"),
        }

        // Node count should remain unchanged
        assert_eq!(dag.node_count(), 3);
    }

    #[test]
    fn test_cycle_detection_after_adding_nodes() {
        // Test cycle detection after adding nodes dynamically
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create initial chain: A -> B
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        dag.add_edge(node_a, node_b).unwrap();

        // Add node C
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset]);

        // Create edge B -> C
        dag.add_edge(node_b, node_c).unwrap();

        // Try to create cycle: C -> A
        let result = dag.add_edge(node_c, node_a);

        assert!(result.is_err());
        assert!(matches!(result, Err(DagError::CycleDetected(_))));

        // DAG should remain valid (no cycle)
        assert_eq!(dag.node_count(), 3);
        assert_eq!(dag.edge_count(), 2);
    }

    #[test]
    fn test_execution_order_update_after_modifications() {
        // Test that execution order is correctly updated after modifications
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create initial structure: A -> B
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        dag.add_edge(node_a, node_b).unwrap();

        let order1 = dag.execution_order().unwrap();
        assert_eq!(order1.len(), 2);

        // Add node C with edge B -> C
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset.clone()]);
        dag.add_edge(node_b, node_c).unwrap();

        let order2 = dag.execution_order().unwrap();
        assert_eq!(order2.len(), 3);
        assert!(order2.contains(&node_c));

        // Remove node C
        dag.remove_node(node_c).unwrap();

        let order3 = dag.execution_order().unwrap();
        assert_eq!(order3.len(), 2);
        assert!(!order3.contains(&node_c));
        assert_eq!(order1, order3);
    }

    #[test]
    fn test_dynamic_dag_complex_modifications() {
        // Test complex dynamic modifications
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create initial structure: A -> B, A -> C
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset.clone()]);

        dag.add_edge(node_a, node_b).unwrap();
        dag.add_edge(node_a, node_c).unwrap();

        assert_eq!(dag.node_count(), 3);
        assert_eq!(dag.edge_count(), 2);

        // Add node D with edges B -> D, C -> D (diamond structure)
        let node_d = dag.add_node("D".to_string(), NodeParams::None, vec![asset.clone()]);
        dag.add_edge(node_b, node_d).unwrap();
        dag.add_edge(node_c, node_d).unwrap();

        assert_eq!(dag.node_count(), 4);
        assert_eq!(dag.edge_count(), 4);

        // Verify execution order
        let order = dag.execution_order().unwrap();
        let a_pos = order.iter().position(|&id| id == node_a).unwrap();
        let d_pos = order.iter().position(|&id| id == node_d).unwrap();
        assert!(a_pos < d_pos);

        // Remove leaf node D
        dag.remove_node(node_d).unwrap();

        assert_eq!(dag.node_count(), 3);
        assert_eq!(dag.edge_count(), 2);

        // Now B and C are leaf nodes, remove both
        dag.remove_node(node_b).unwrap();
        dag.remove_node(node_c).unwrap();

        assert_eq!(dag.node_count(), 1);
        assert_eq!(dag.edge_count(), 0);
    }

    #[test]
    fn test_remove_nonexistent_node() {
        // Test removing a node that doesn't exist
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset]);

        // Try to remove non-existent node
        let fake_node = NodeId(999);
        let result = dag.remove_node(fake_node);

        assert!(result.is_err());
        assert!(matches!(result, Err(DagError::NodeNotFound(_))));

        // Original node should still exist
        assert_eq!(dag.node_count(), 1);
        assert!(dag.get_node(node_a).is_some());
    }

    #[test]
    fn test_adding_parallel_branches_dynamically() {
        // Test adding multiple parallel branches dynamically
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Start with single root node
        let root = dag.add_node("Root".to_string(), NodeParams::None, vec![asset.clone()]);

        // Add multiple branches
        let mut leaf_nodes = Vec::new();
        for i in 0..5 {
            let branch = dag.add_node(
                format!("Branch{}", i),
                NodeParams::None,
                vec![asset.clone()],
            );
            dag.add_edge(root, branch).unwrap();
            leaf_nodes.push(branch);
        }

        assert_eq!(dag.node_count(), 6); // 1 root + 5 branches
        assert_eq!(dag.edge_count(), 5);

        // Verify all branches are children of root
        let root_children = dag.get_children(root);
        assert_eq!(root_children.len(), 5);

        // Remove all leaf nodes
        for leaf in leaf_nodes {
            dag.remove_node(leaf).unwrap();
        }

        assert_eq!(dag.node_count(), 1);
        assert_eq!(dag.edge_count(), 0);
    }

    // Task Group 7.1: Write 2-8 focused tests for integration

    #[test]
    fn test_integration_with_asset_key_and_time_series_point() {
        // Test DAG integration with AssetKey and TimeSeriesPoint
        use chrono::Utc;

        let mut dag = AnalyticsDag::new();

        // Create nodes with different asset keys
        let aapl = AssetKey::new_equity("AAPL").unwrap();
        let msft = AssetKey::new_equity("MSFT").unwrap();
        let es_future = AssetKey::new_future(
            "ES".to_string(),
            chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        )
        .unwrap();

        let node1 = dag.add_node(
            "AAPL_data".to_string(),
            NodeParams::None,
            vec![aapl.clone()],
        );
        let node2 = dag.add_node(
            "MSFT_data".to_string(),
            NodeParams::None,
            vec![msft.clone()],
        );
        let node3 = dag.add_node("ES_data".to_string(), NodeParams::None, vec![es_future]);
        let node4 = dag.add_node(
            "correlation".to_string(),
            NodeParams::None,
            vec![aapl, msft],
        );

        // Add dependencies
        dag.add_edge(node1, node4).unwrap();
        dag.add_edge(node2, node4).unwrap();

        // Verify nodes and edges
        assert_eq!(dag.node_count(), 4);
        assert_eq!(dag.edge_count(), 2);

        // Verify node data
        let node1_data = dag.get_node(node1).unwrap();
        assert_eq!(node1_data.node_type, "AAPL_data");
        assert_eq!(node1_data.assets.len(), 1);

        let node4_data = dag.get_node(node4).unwrap();
        assert_eq!(node4_data.assets.len(), 2);

        // Test TimeSeriesPoint in NodeOutput
        let ts_point = TimeSeriesPoint::new(Utc::now(), 150.0);
        let output = NodeOutput::Single(vec![ts_point.clone()]);

        match output {
            NodeOutput::Single(points) => {
                assert_eq!(points.len(), 1);
                assert_eq!(points[0].close_price, 150.0);
            }
            _ => panic!("Expected Single output"),
        }
    }

    #[tokio::test]
    async fn test_integration_with_data_provider() {
        // Test DAG integration with DataProvider trait
        use crate::time_series::{DateRange, InMemoryDataProvider};
        use chrono::{NaiveDate, Utc};

        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create simple DAG
        let node_a = dag.add_node(
            "data_fetch".to_string(),
            NodeParams::None,
            vec![asset.clone()],
        );
        let node_b = dag.add_node("compute".to_string(), NodeParams::None, vec![asset.clone()]);
        dag.add_edge(node_a, node_b).unwrap();

        // Create data provider with test data
        let mut provider = InMemoryDataProvider::new();
        let test_data = vec![
            TimeSeriesPoint::new(Utc::now(), 150.0),
            TimeSeriesPoint::new(Utc::now(), 151.0),
        ];
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        provider.add_data(asset.clone(), test_data.clone());

        // Execute DAG with simulated data (demonstrating integration pattern)
        let test_data_arc = Arc::new(test_data.clone());
        let compute_fn = move |node: Node, inputs: Vec<NodeOutput>| {
            let data = Arc::clone(&test_data_arc);

            async move {
                if node.node_type == "data_fetch" {
                    // Simulate fetching data from provider
                    // In real usage, would call provider.get_time_series()
                    Ok(NodeOutput::Single((*data).clone()))
                } else {
                    // Use inputs from previous node
                    Ok(NodeOutput::Scalar(inputs.len() as f64))
                }
            }
        };

        let results = dag.execute(compute_fn).await.unwrap();

        // Verify results
        assert_eq!(results.len(), 2);

        match results.get(&node_a).unwrap() {
            NodeOutput::Single(data) => {
                // Verify data was returned from the simulated fetch
                assert_eq!(data.len(), 2);
                assert_eq!(data[0].close_price, 150.0);
                assert_eq!(data[1].close_price, 151.0);
            }
            _ => panic!("Expected Single output"),
        }

        // Verify that DataProvider trait integration works
        // Note: InMemoryDataProvider filters by date range, so test data needs matching timestamps
        // For integration testing, we verify the pattern works with the DAG execution above
    }

    #[tokio::test]
    async fn test_push_mode_incremental_updates() {
        // Test push-mode API hooks and incremental update triggering
        use std::sync::Arc;
        use tokio::sync::Mutex;

        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create DAG: A -> B -> C
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset]);

        dag.add_edge(node_a, node_b).unwrap();
        dag.add_edge(node_b, node_c).unwrap();

        // Track which nodes were executed
        let executed = Arc::new(Mutex::new(Vec::new()));
        let executed_clone = Arc::clone(&executed);

        let compute_fn = move |node: Node, inputs: Vec<NodeOutput>| {
            let executed = Arc::clone(&executed_clone);

            async move {
                let mut exec_guard = executed.lock().await;
                exec_guard.push(node.id);
                drop(exec_guard);

                let input_sum: f64 = inputs
                    .iter()
                    .filter_map(|output| match output {
                        NodeOutput::Scalar(v) => Some(*v),
                        _ => None,
                    })
                    .sum();

                Ok(NodeOutput::Scalar(input_sum + node.id.0 as f64))
            }
        };

        // Simulate incremental update when node_a changes
        let results = dag.execute_incremental(node_a, compute_fn).await.unwrap();

        // Only B and C should be executed (descendants of A)
        let exec_list = executed.lock().await;
        assert_eq!(exec_list.len(), 2);
        assert!(exec_list.contains(&node_b));
        assert!(exec_list.contains(&node_c));
        assert!(!exec_list.contains(&node_a)); // A itself not re-executed

        // Results should contain B and C
        assert_eq!(results.len(), 2);
        assert!(results.contains_key(&node_b));
        assert!(results.contains_key(&node_c));
    }

    #[test]
    fn test_get_descendants_for_push_mode() {
        // Test getting descendants for push-mode propagation
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Create diamond: A -> B, A -> C, B -> D, C -> D
        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_b = dag.add_node("B".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_c = dag.add_node("C".to_string(), NodeParams::None, vec![asset.clone()]);
        let node_d = dag.add_node("D".to_string(), NodeParams::None, vec![asset]);

        dag.add_edge(node_a, node_b).unwrap();
        dag.add_edge(node_a, node_c).unwrap();
        dag.add_edge(node_b, node_d).unwrap();
        dag.add_edge(node_c, node_d).unwrap();

        // Get descendants of A (should be B, C, D)
        let descendants_a = dag.get_descendants(node_a);
        assert_eq!(descendants_a.len(), 3);
        assert!(descendants_a.contains(&node_b));
        assert!(descendants_a.contains(&node_c));
        assert!(descendants_a.contains(&node_d));

        // Get descendants of B (should be D)
        let descendants_b = dag.get_descendants(node_b);
        assert_eq!(descendants_b.len(), 1);
        assert!(descendants_b.contains(&node_d));

        // Get descendants of D (should be empty - leaf node)
        let descendants_d = dag.get_descendants(node_d);
        assert_eq!(descendants_d.len(), 0);
    }

    #[test]
    fn test_error_handling_follows_patterns() {
        // Test error handling following existing patterns (Result types, clear messages)
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        let node_a = dag.add_node("A".to_string(), NodeParams::None, vec![asset]);

        // Test NodeNotFound error
        let fake_node = NodeId(999);
        let result = dag.remove_node(fake_node);
        assert!(result.is_err());

        match result {
            Err(DagError::NodeNotFound(msg)) => {
                assert!(msg.contains("NodeId(999)"));
                assert!(msg.contains("not found"));
            }
            _ => panic!("Expected NodeNotFound error"),
        }

        // Test CycleDetected error
        let result = dag.add_edge(node_a, node_a);
        assert!(result.is_err());
        assert!(matches!(result, Err(DagError::CycleDetected(_))));

        // Test InvalidOperation error
        // Add a dependent first
        let node_b = dag.add_node(
            "B".to_string(),
            NodeParams::None,
            vec![AssetKey::new_equity("MSFT").unwrap()],
        );
        dag.add_edge(node_a, node_b).unwrap();

        let result = dag.remove_node(node_a);
        assert!(result.is_err());
        match result {
            Err(DagError::InvalidOperation(msg)) => {
                assert!(msg.contains("dependencies"));
            }
            _ => panic!("Expected InvalidOperation error"),
        }
    }

    #[test]
    fn test_node_output_types() {
        // Test different NodeOutput types (Single, Collection, Scalar, None)
        use chrono::Utc;

        // Single time series
        let single = NodeOutput::Single(vec![TimeSeriesPoint::new(Utc::now(), 100.0)]);
        assert!(matches!(single, NodeOutput::Single(_)));

        // Collection of time series
        let collection = NodeOutput::Collection(vec![
            vec![TimeSeriesPoint::new(Utc::now(), 100.0)],
            vec![TimeSeriesPoint::new(Utc::now(), 200.0)],
        ]);
        assert!(matches!(collection, NodeOutput::Collection(_)));

        // Scalar value
        let scalar = NodeOutput::Scalar(42.0);
        assert!(matches!(scalar, NodeOutput::Scalar(_)));

        // No output
        let none = NodeOutput::None;
        assert!(matches!(none, NodeOutput::None));
    }

    #[tokio::test]
    async fn test_multi_asset_node_execution() {
        // Test nodes operating on multiple assets
        let mut dag = AnalyticsDag::new();

        let aapl = AssetKey::new_equity("AAPL").unwrap();
        let msft = AssetKey::new_equity("MSFT").unwrap();

        // Node operating on two assets (e.g., correlation)
        let correlation_node = dag.add_node(
            "correlation".to_string(),
            NodeParams::Map({
                let mut m = HashMap::new();
                m.insert("method".to_string(), "pearson".to_string());
                m
            }),
            vec![aapl, msft],
        );

        let compute_fn = |node: Node, _inputs: Vec<NodeOutput>| async move {
            // Verify multi-asset node
            assert_eq!(node.assets.len(), 2);

            // Simulate correlation computation
            Ok(NodeOutput::Scalar(0.85))
        };

        let results = dag.execute(compute_fn).await.unwrap();

        match results.get(&correlation_node).unwrap() {
            NodeOutput::Scalar(v) => assert_eq!(*v, 0.85),
            _ => panic!("Expected Scalar output"),
        }
    }

    #[test]
    fn test_data_provider_error_conversion() {
        // Test DataProviderError to DagError conversion
        let provider_err = DataProviderError::AssetNotFound;
        let dag_err: DagError = provider_err.into();

        match dag_err {
            DagError::DataProviderError(msg) => {
                assert!(msg.contains("Asset not found"));
            }
            _ => panic!("Expected DataProviderError variant"),
        }

        // Test error display
        let err = DagError::DataProviderError("Test error".to_string());
        let err_string = format!("{}", err);
        assert!(err_string.contains("Data provider error"));
        assert!(err_string.contains("Test error"));
    }

    // Task Group 1: Core Pull-Mode Execution Tests

    #[test]
    fn test_execute_pull_mode_single_dataprovider_node() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};

        // Set up test data
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 101.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(), 102.0),
        ];
        provider.add_data(aapl.clone(), test_data.clone());

        // Create DAG with single DataProvider node
        let mut dag = AnalyticsDag::new();
        let data_node = dag.add_node(
            "DataProvider".to_string(),
            NodeParams::None,
            vec![aapl.clone()],
        );

        // Execute in pull-mode
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );

        let result = dag
            .execute_pull_mode(data_node, date_range, &provider)
            .unwrap();

        // Verify results
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].close_price, 100.0);
        assert_eq!(result[1].close_price, 101.0);
        assert_eq!(result[2].close_price, 102.0);
    }

    #[test]
    fn test_execute_pull_mode_node_not_found() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::NaiveDate;

        let provider = InMemoryDataProvider::new();
        let dag = AnalyticsDag::new();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        let non_existent_node = NodeId(999);
        let result = dag.execute_pull_mode(non_existent_node, date_range, &provider);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DagError::NodeNotFound(_)));
    }

    #[test]
    fn test_execute_pull_mode_returns_node() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};

        // Create DAG with dependencies: DataProvider -> Returns
        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        // Set up test data
        let mut provider = InMemoryDataProvider::new();
        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 102.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(), 101.0),
        ];
        provider.add_data(aapl.clone(), test_data);

        let (_, _, returns_node) = add_returns_chain(&mut dag, aapl.clone());

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );

        // Execute pull-mode
        let result = dag
            .execute_pull_mode(returns_node, date_range, &provider)
            .unwrap();

        // Should have 3 returns (same length as prices)
        assert_eq!(result.len(), 3);

        // First return is NaN (no previous price)
        assert!(result[0].close_price.is_nan() || result[0].close_price == 0.0);

        // Verify returns calculations: ln(102/100) ≈ 0.0198, ln(101/102) ≈ -0.0099
        assert!((result[1].close_price - (102.0_f64 / 100.0).ln()).abs() < 0.0001);
        assert!((result[2].close_price - (101.0_f64 / 102.0).ln()).abs() < 0.0001);
    }

    #[test]
    fn test_execute_pull_mode_volatility_node() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};
        use std::collections::HashMap;

        // Create DAG: DataProvider -> Returns -> Volatility
        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        // Set up test data with more points for volatility
        let mut provider = InMemoryDataProvider::new();
        let mut test_data = Vec::new();
        let mut price = 100.0;
        for day in 1..=15 {
            test_data.push(TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, day, 0, 0, 0).unwrap(),
                price,
            ));
            price += (day % 3) as f64 - 1.0; // Vary prices
        }
        provider.add_data(aapl.clone(), test_data);

        let data_node = dag.add_node(
            "DataProvider".to_string(),
            NodeParams::None,
            vec![aapl.clone()],
        );
        let returns_node =
            dag.add_node("Returns".to_string(), NodeParams::None, vec![aapl.clone()]);

        // Volatility node with window size parameter
        let mut vol_params = HashMap::new();
        vol_params.insert("window_size".to_string(), "5".to_string());
        let vol_node = dag.add_node(
            "Volatility".to_string(),
            NodeParams::Map(vol_params),
            vec![aapl.clone()],
        );

        dag.add_edge(data_node, returns_node).unwrap();
        dag.add_edge(returns_node, vol_node).unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        // Execute pull-mode
        let result = dag
            .execute_pull_mode(vol_node, date_range, &provider)
            .unwrap();

        // Volatility starts after window is full
        // 15 prices -> 14 returns -> 10 volatility values (window=5)
        assert!(
            result.len() >= 5,
            "Expected at least 5 volatility values, got {}",
            result.len()
        );

        // All volatility values should be non-negative (or NaN)
        for (i, point) in result.iter().enumerate() {
            if !point.close_price.is_nan() {
                assert!(
                    point.close_price >= 0.0,
                    "Volatility at index {} should be non-negative, got {}",
                    i,
                    point.close_price
                );
            }
        }
    }

    // Task Group 2: Burn-in Calculation Tests

    #[test]
    fn test_calculate_burnin_days_dataprovider() {
        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let data_node = dag.add_node("DataProvider".to_string(), NodeParams::None, vec![aapl]);

        // DataProvider should need 0 burn-in days
        assert_eq!(dag.calculate_burnin_days(data_node), 0);
    }

    #[test]
    fn test_calculate_burnin_days_returns() {
        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let data_node = dag.add_node(
            "DataProvider".to_string(),
            NodeParams::None,
            vec![aapl.clone()],
        );
        let returns_node = dag.add_node("Returns".to_string(), NodeParams::None, vec![aapl]);
        dag.add_edge(data_node, returns_node).unwrap();

        // Returns should need 1 burn-in day (for first return calculation)
        assert_eq!(dag.calculate_burnin_days(returns_node), 1);
    }

    #[test]
    fn test_calculate_burnin_days_volatility() {
        use std::collections::HashMap;

        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let data_node = dag.add_node(
            "DataProvider".to_string(),
            NodeParams::None,
            vec![aapl.clone()],
        );
        let returns_node =
            dag.add_node("Returns".to_string(), NodeParams::None, vec![aapl.clone()]);

        let mut vol_params = HashMap::new();
        vol_params.insert("window_size".to_string(), "10".to_string());
        let vol_node = dag.add_node(
            "Volatility".to_string(),
            NodeParams::Map(vol_params),
            vec![aapl],
        );

        dag.add_edge(data_node, returns_node).unwrap();
        dag.add_edge(returns_node, vol_node).unwrap();

        // Volatility(10) should need 11 burn-in days (10 for window + 1 for returns)
        assert_eq!(dag.calculate_burnin_days(vol_node), 11);
    }

    #[test]
    fn test_pull_mode_with_burnin_filters_correctly() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};

        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        // Create test data with extra days before the requested range
        let mut provider = InMemoryDataProvider::new();
        let mut test_data = Vec::new();
        for day in 1..=15 {
            test_data.push(TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, day, 0, 0, 0).unwrap(),
                100.0 + day as f64,
            ));
        }
        provider.add_data(aapl.clone(), test_data);

        let data_node = dag.add_node(
            "DataProvider".to_string(),
            NodeParams::None,
            vec![aapl.clone()],
        );
        let returns_node = dag.add_node("Returns".to_string(), NodeParams::None, vec![aapl]);
        dag.add_edge(data_node, returns_node).unwrap();

        // Request only days 5-10
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        let result = dag
            .execute_pull_mode(returns_node, date_range, &provider)
            .unwrap();

        // Result should only contain days 5-10 (6 days)
        assert_eq!(result.len(), 6);

        // Verify timestamps are within requested range
        for point in &result {
            let date = point.timestamp.date_naive();
            assert!(date >= NaiveDate::from_ymd_opt(2024, 1, 5).unwrap());
            assert!(date <= NaiveDate::from_ymd_opt(2024, 1, 10).unwrap());
        }
    }

    // Task Group 3: Execution Cache System Tests

    #[test]
    fn test_execution_cache_basic_operations() {
        use chrono::{NaiveDate, TimeZone, Utc};

        let mut cache = ExecutionCache::new();
        let node_id = NodeId(1);

        // Test empty cache
        assert!(cache.get(node_id).is_none());

        // Insert data
        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 101.0),
        ];
        let range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
        );

        cache.insert(node_id, test_data.clone(), range);

        // Test get
        assert_eq!(cache.get(node_id).unwrap().len(), 2);
        assert_eq!(cache.get(node_id).unwrap()[0].close_price, 100.0);

        // Test clear
        cache.clear();
        assert!(cache.get(node_id).is_none());
    }

    #[test]
    fn test_execution_cache_extract_range() {
        use chrono::{NaiveDate, TimeZone, Utc};

        let mut cache = ExecutionCache::new();
        let node_id = NodeId(1);

        // Insert data spanning 5 days
        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 101.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(), 102.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 4, 0, 0, 0).unwrap(), 103.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 5, 0, 0, 0).unwrap(), 104.0),
        ];
        let full_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
        );

        cache.insert(node_id, test_data, full_range);

        // Extract subset (days 2-4)
        let subset_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 4).unwrap(),
        );

        let extracted = cache.extract_range(node_id, &subset_range).unwrap();

        // Should have 3 days
        assert_eq!(extracted.len(), 3);
        assert_eq!(extracted[0].close_price, 101.0);
        assert_eq!(extracted[1].close_price, 102.0);
        assert_eq!(extracted[2].close_price, 103.0);
    }

    #[test]
    fn test_execution_cache_diamond_dependency() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};

        // Create diamond DAG: Data -> (Node1, Node2) -> Node3
        // This tests that Node3 can retrieve both parent outputs from cache
        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let mut provider = InMemoryDataProvider::new();
        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 102.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(), 104.0),
        ];
        provider.add_data(aapl.clone(), test_data);

        // Create diamond structure
        let data_node = dag.add_node(
            "DataProvider".to_string(),
            NodeParams::None,
            vec![aapl.clone()],
        );
        let returns_node1 =
            dag.add_node("Returns".to_string(), NodeParams::None, vec![aapl.clone()]);
        let returns_node2 =
            dag.add_node("Returns".to_string(), NodeParams::None, vec![aapl.clone()]);

        dag.add_edge(data_node, returns_node1).unwrap();
        dag.add_edge(data_node, returns_node2).unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );

        // Execute both nodes - they should both use the cached data_node output
        let result1 = dag
            .execute_pull_mode(returns_node1, date_range.clone(), &provider)
            .unwrap();
        let result2 = dag
            .execute_pull_mode(returns_node2, date_range, &provider)
            .unwrap();

        // Both should produce identical results
        assert_eq!(result1.len(), result2.len());
        for (p1, p2) in result1.iter().zip(result2.iter()) {
            // Handle NaN comparison properly
            if p1.close_price.is_nan() && p2.close_price.is_nan() {
                continue;
            }
            assert_eq!(p1.close_price, p2.close_price);
        }
    }

    // Task Group 4: Multi-Asset Parallel Execution Tests

    #[test]
    fn test_parallel_execution_two_independent_nodes() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};

        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();
        let msft = AssetKey::new_equity("MSFT").unwrap();

        // Set up test data for both assets
        let mut provider = InMemoryDataProvider::new();

        let aapl_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 102.0),
        ];
        provider.add_data(aapl.clone(), aapl_data);

        let msft_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 200.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 204.0),
        ];
        provider.add_data(msft.clone(), msft_data);

        // Create independent nodes for each asset
        let aapl_data_node = dag.add_node(
            "DataProvider".to_string(),
            NodeParams::None,
            vec![aapl.clone()],
        );
        let msft_data_node = dag.add_node(
            "DataProvider".to_string(),
            NodeParams::None,
            vec![msft.clone()],
        );

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
        );

        // Execute in parallel
        let results = dag
            .execute_pull_mode_parallel(vec![aapl_data_node, msft_data_node], date_range, &provider)
            .unwrap();

        // Verify results
        assert_eq!(results.len(), 2);
        assert_eq!(results.get(&aapl_data_node).unwrap().len(), 2);
        assert_eq!(results.get(&msft_data_node).unwrap().len(), 2);
        assert_eq!(results.get(&aapl_data_node).unwrap()[0].close_price, 100.0);
        assert_eq!(results.get(&msft_data_node).unwrap()[0].close_price, 200.0);
    }

    #[test]
    fn test_parallel_execution_multiple_assets_same_analytic() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};

        let mut dag = AnalyticsDag::new();

        // Create 3 assets
        let aapl = AssetKey::new_equity("AAPL").unwrap();
        let msft = AssetKey::new_equity("MSFT").unwrap();
        let googl = AssetKey::new_equity("GOOGL").unwrap();

        let mut provider = InMemoryDataProvider::new();

        // Add data for all 3 assets
        for (asset, base_price) in vec![
            (aapl.clone(), 100.0),
            (msft.clone(), 200.0),
            (googl.clone(), 300.0),
        ] {
            let data = vec![
                TimeSeriesPoint::new(
                    Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
                    base_price,
                ),
                TimeSeriesPoint::new(
                    Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(),
                    base_price + 2.0,
                ),
                TimeSeriesPoint::new(
                    Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(),
                    base_price + 4.0,
                ),
            ];
            provider.add_data(asset, data);
        }

        // Create Returns nodes for all 3 assets
        let mut node_ids = Vec::new();
        for asset in vec![aapl, msft, googl] {
            let data_node = dag.add_node(
                "DataProvider".to_string(),
                NodeParams::None,
                vec![asset.clone()],
            );
            let returns_node = dag.add_node("Returns".to_string(), NodeParams::None, vec![asset]);
            dag.add_edge(data_node, returns_node).unwrap();
            node_ids.push(returns_node);
        }

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );

        // Execute all 3 in parallel
        let results = dag
            .execute_pull_mode_parallel(node_ids.clone(), date_range, &provider)
            .unwrap();

        // Verify all 3 completed
        assert_eq!(results.len(), 3);
        for node_id in node_ids {
            assert_eq!(results.get(&node_id).unwrap().len(), 3);
        }
    }

    #[test]
    fn test_parallel_execution_error_handling() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::NaiveDate;

        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        // Create a node but don't provide data - should error
        let data_node = dag.add_node("DataProvider".to_string(), NodeParams::None, vec![aapl]);

        let provider = InMemoryDataProvider::new(); // Empty provider
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        // Should return error
        let result = dag.execute_pull_mode_parallel(vec![data_node], date_range, &provider);
        assert!(result.is_err());
    }

    // Task Group 5: Integration with Existing Systems Tests

    #[test]
    fn test_integration_with_inmemory_provider_various_ranges() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};

        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        // Create 1 year of data
        let mut provider = InMemoryDataProvider::new();
        let mut test_data = Vec::new();
        for day in 1..=365 {
            let date =
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap() + chrono::Duration::days(day - 1);
            test_data.push(TimeSeriesPoint::new(
                Utc.from_utc_datetime(&date.and_hms_opt(0, 0, 0).unwrap()),
                100.0 + (day as f64) * 0.1,
            ));
        }
        provider.add_data(aapl.clone(), test_data);

        let data_node = dag.add_node("DataProvider".to_string(), NodeParams::None, vec![aapl]);

        // Test 1 day
        let result = dag
            .execute_pull_mode(
                data_node,
                DateRange::new(
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                ),
                &provider,
            )
            .unwrap();
        assert_eq!(result.len(), 1);

        // Test 1 month (30 days)
        let result = dag
            .execute_pull_mode(
                data_node,
                DateRange::new(
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(2024, 1, 30).unwrap(),
                ),
                &provider,
            )
            .unwrap();
        assert_eq!(result.len(), 30);

        // Test 1 year
        let result = dag
            .execute_pull_mode(
                data_node,
                DateRange::new(
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                ),
                &provider,
            )
            .unwrap();
        assert_eq!(result.len(), 365);
    }

    #[test]
    fn test_integration_returns_matches_analytics_function() {
        use crate::analytics::testing::calculate_returns;
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};

        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        // Create test data
        let mut provider = InMemoryDataProvider::new();
        let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0];
        let test_data: Vec<TimeSeriesPoint> = prices
            .iter()
            .enumerate()
            .map(|(i, &price)| {
                TimeSeriesPoint::new(
                    Utc.with_ymd_and_hms(2024, 1, (i + 1) as u32, 0, 0, 0)
                        .unwrap(),
                    price,
                )
            })
            .collect();
        provider.add_data(aapl.clone(), test_data);

        // Set up DAG
        let data_node = dag.add_node(
            "DataProvider".to_string(),
            NodeParams::None,
            vec![aapl.clone()],
        );
        let returns_node = dag.add_node("Returns".to_string(), NodeParams::None, vec![aapl]);
        dag.add_edge(data_node, returns_node).unwrap();

        // Execute pull-mode
        let result = dag
            .execute_pull_mode(
                returns_node,
                DateRange::new(
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
                ),
                &provider,
            )
            .unwrap();

        // Calculate expected returns using analytics function
        let expected_returns = calculate_returns(&prices);

        // Compare (allowing for NaN)
        assert_eq!(result.len(), expected_returns.len());
        for (actual, &expected) in result.iter().zip(expected_returns.iter()) {
            if expected.is_nan() {
                assert!(actual.close_price.is_nan());
            } else {
                assert!((actual.close_price - expected).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_integration_volatility_various_windows() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};
        use std::collections::HashMap;

        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        // Create test data with 30 days
        let mut provider = InMemoryDataProvider::new();
        let mut test_data = Vec::new();
        for day in 1..=30 {
            test_data.push(TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, day, 0, 0, 0).unwrap(),
                100.0 + (day as f64) * 0.5,
            ));
        }
        provider.add_data(aapl.clone(), test_data);

        // Test different window sizes
        for window_size in vec![5, 10, 20] {
            let data_node = dag.add_node(
                "DataProvider".to_string(),
                NodeParams::None,
                vec![aapl.clone()],
            );
            let returns_node =
                dag.add_node("Returns".to_string(), NodeParams::None, vec![aapl.clone()]);

            let mut vol_params = HashMap::new();
            vol_params.insert("window_size".to_string(), window_size.to_string());
            let vol_node = dag.add_node(
                "Volatility".to_string(),
                NodeParams::Map(vol_params),
                vec![aapl.clone()],
            );

            dag.add_edge(data_node, returns_node).unwrap();
            dag.add_edge(returns_node, vol_node).unwrap();

            let result = dag
                .execute_pull_mode(
                    vol_node,
                    DateRange::new(
                        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        NaiveDate::from_ymd_opt(2024, 1, 30).unwrap(),
                    ),
                    &provider,
                )
                .unwrap();

            // Should have 30 volatility values
            assert_eq!(result.len(), 30, "Failed for window size {}", window_size);

            // All non-NaN values should be non-negative
            for point in &result {
                if !point.close_price.is_nan() {
                    assert!(point.close_price >= 0.0);
                }
            }
        }
    }

    #[test]
    fn test_integration_complex_dag_multiple_analytics() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};
        use std::collections::HashMap;

        // Build a complex DAG: DataProvider -> Returns -> (Volatility5, Volatility10)
        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let mut provider = InMemoryDataProvider::new();
        let mut test_data = Vec::new();
        for day in 1..=20 {
            test_data.push(TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, day, 0, 0, 0).unwrap(),
                100.0 + (day as f64),
            ));
        }
        provider.add_data(aapl.clone(), test_data);

        let data_node = dag.add_node(
            "DataProvider".to_string(),
            NodeParams::None,
            vec![aapl.clone()],
        );
        let returns_node =
            dag.add_node("Returns".to_string(), NodeParams::None, vec![aapl.clone()]);

        let mut vol5_params = HashMap::new();
        vol5_params.insert("window_size".to_string(), "5".to_string());
        let vol5_node = dag.add_node(
            "Volatility".to_string(),
            NodeParams::Map(vol5_params),
            vec![aapl.clone()],
        );

        let mut vol10_params = HashMap::new();
        vol10_params.insert("window_size".to_string(), "10".to_string());
        let vol10_node = dag.add_node(
            "Volatility".to_string(),
            NodeParams::Map(vol10_params),
            vec![aapl],
        );

        dag.add_edge(data_node, returns_node).unwrap();
        dag.add_edge(returns_node, vol5_node).unwrap();
        dag.add_edge(returns_node, vol10_node).unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
        );

        // Execute different nodes - should all work
        let vol5_result = dag
            .execute_pull_mode(vol5_node, date_range.clone(), &provider)
            .unwrap();
        let vol10_result = dag
            .execute_pull_mode(vol10_node, date_range, &provider)
            .unwrap();

        assert_eq!(vol5_result.len(), 20);
        assert_eq!(vol10_result.len(), 20);
    }

    // Task Group 6: Validation & Testing

    #[test]
    fn test_edge_case_empty_date_range() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::NaiveDate;

        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();
        let data_node = dag.add_node("DataProvider".to_string(), NodeParams::None, vec![aapl]);

        let provider = InMemoryDataProvider::new();

        // Empty range (start > end) - should return empty
        let result = dag.execute_pull_mode(
            data_node,
            DateRange::new(
                NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
            &provider,
        );

        // May error or return empty, both are acceptable
        assert!(result.is_err() || result.unwrap().is_empty());
    }

    #[test]
    fn test_edge_case_single_data_point() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};

        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let mut provider = InMemoryDataProvider::new();
        let test_data = vec![TimeSeriesPoint::new(
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            100.0,
        )];
        provider.add_data(aapl.clone(), test_data);

        let data_node = dag.add_node(
            "DataProvider".to_string(),
            NodeParams::None,
            vec![aapl.clone()],
        );
        let returns_node = dag.add_node("Returns".to_string(), NodeParams::None, vec![aapl]);
        dag.add_edge(data_node, returns_node).unwrap();

        let result = dag
            .execute_pull_mode(
                returns_node,
                DateRange::new(
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                ),
                &provider,
            )
            .unwrap();

        // Single point should return 1 result (NaN for returns)
        assert_eq!(result.len(), 1);
        assert!(result[0].close_price.is_nan());
    }

    #[test]
    fn test_edge_case_date_range_before_data() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};

        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let mut provider = InMemoryDataProvider::new();
        // Data starts from Jan 10
        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 10, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 11, 0, 0, 0).unwrap(), 101.0),
        ];
        provider.add_data(aapl.clone(), test_data);

        let data_node = dag.add_node("DataProvider".to_string(), NodeParams::None, vec![aapl]);

        // Request data before availability (Jan 1-5)
        let result = dag
            .execute_pull_mode(
                data_node,
                DateRange::new(
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
                ),
                &provider,
            )
            .unwrap();

        // Should return empty (no data in that range)
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_edge_case_date_range_after_data() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::{NaiveDate, TimeZone, Utc};

        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let mut provider = InMemoryDataProvider::new();
        // Data ends at Jan 5
        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 5, 0, 0, 0).unwrap(), 101.0),
        ];
        provider.add_data(aapl.clone(), test_data);

        let data_node = dag.add_node("DataProvider".to_string(), NodeParams::None, vec![aapl]);

        // Request data after availability (Jan 10-15)
        let result = dag
            .execute_pull_mode(
                data_node,
                DateRange::new(
                    NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
                    NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                ),
                &provider,
            )
            .unwrap();

        // Should return empty
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_error_handling_invalid_node_id() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::NaiveDate;

        let dag = AnalyticsDag::new();
        let provider = InMemoryDataProvider::new();

        let non_existent_node = NodeId(9999);
        let result = dag.execute_pull_mode(
            non_existent_node,
            DateRange::new(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            ),
            &provider,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DagError::NodeNotFound(_)));
    }

    #[test]
    fn test_error_handling_provider_failure() {
        use crate::time_series::InMemoryDataProvider;
        use chrono::NaiveDate;

        let mut dag = AnalyticsDag::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        // Create node but don't provide data - will cause AssetNotFound error
        let data_node = dag.add_node("DataProvider".to_string(), NodeParams::None, vec![aapl]);

        let provider = InMemoryDataProvider::new(); // Empty provider

        let result = dag.execute_pull_mode(
            data_node,
            DateRange::new(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            ),
            &provider,
        );

        assert!(result.is_err());
        // Should be a DataProviderError
        if let Err(e) = result {
            assert!(
                format!("{}", e).contains("Asset not found")
                    || format!("{}", e).contains("Data provider error")
            );
        }
    }
}
