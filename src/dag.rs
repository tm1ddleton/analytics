//! DAG Computation Framework
//! 
//! This module provides a DAG (Directed Acyclic Graph) construction and execution engine
//! for wiring analytics dependencies explicitly with cycle detection, topological sorting,
//! and parallel execution support.

use crate::asset_key::AssetKey;
use crate::time_series::TimeSeriesPoint;
use daggy::{Dag, NodeIndex, EdgeIndex, WouldCycle, Walker, petgraph::Direction};
use serde::{Deserialize, Serialize};
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
}

impl std::fmt::Display for DagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DagError::CycleDetected(msg) => write!(f, "Cycle detected: {}", msg),
            DagError::NodeNotFound(msg) => write!(f, "Node not found: {}", msg),
            DagError::EdgeNotFound(msg) => write!(f, "Edge not found: {}", msg),
            DagError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            DagError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
        }
    }
}

impl std::error::Error for DagError {}

/// Node identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub usize);

/// Parameters for a node (generic, can be extended)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeParams {
    /// Simple key-value parameters
    Map(HashMap<String, String>),
    /// Empty parameters
    None,
}

/// Execution result for a node (can be collection of time series)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeOutput {
    /// Single time series result
    Single(Vec<TimeSeriesPoint>),
    /// Multiple time series results (collection)
    Collection(Vec<Vec<TimeSeriesPoint>>),
    /// Scalar value (e.g., correlation coefficient)
    Scalar(f64),
    /// No output (for sink nodes)
    None,
}

/// Node in the DAG representing an analytics computation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier for this node
    pub id: NodeId,
    /// Node type/name (e.g., "moving_average", "correlation")
    pub node_type: String,
    /// Parameters for this node (e.g., {"window": "20"} for 20-day moving average)
    pub params: NodeParams,
    /// Assets this node operates on
    pub assets: Vec<AssetKey>,
    /// Computation function identifier (for future use)
    pub computation_id: Option<String>,
}

impl Node {
    /// Creates a new node
    pub fn new(id: NodeId, node_type: String, params: NodeParams, assets: Vec<AssetKey>) -> Self {
        Node {
            id,
            node_type,
            params,
            assets,
            computation_id: None,
        }
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
}

impl AnalyticsDag {
    /// Creates a new empty DAG
    pub fn new() -> Self {
        AnalyticsDag {
            dag: Dag::new(),
            node_id_to_index: HashMap::new(),
            index_to_node_id: HashMap::new(),
            next_node_id: 0,
            cached_toposort: None,
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

    /// Adds an edge (dependency) between two nodes
    /// 
    /// # Arguments
    /// * `from` - Source node ID
    /// * `to` - Target node ID
    /// 
    /// # Returns
    /// Returns Ok(EdgeIndex) if successful, or Err(DagError) if cycle would be created
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> Result<EdgeIndex, DagError> {
        let from_index = self.node_id_to_index.get(&from)
            .ok_or_else(|| DagError::NodeNotFound(format!("Node {:?} not found", from)))?;
        let to_index = self.node_id_to_index.get(&to)
            .ok_or_else(|| DagError::NodeNotFound(format!("Node {:?} not found", to)))?;

        match self.dag.add_edge(*from_index, *to_index, ()) {
            Ok(edge_index) => {
                // Invalidate cache when DAG structure changes
                self.cached_toposort = None;
                Ok(edge_index)
            }
            Err(_would_cycle) => Err(DagError::CycleDetected(
                format!("Adding edge from {:?} to {:?} would create a cycle", from, to)
            )),
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
                "Topological sort failed - DAG may contain cycles".to_string()
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
            for neighbor_idx in self.dag.graph().neighbors_directed(node_idx, Direction::Outgoing) {
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

            for neighbor_idx in self.dag.graph().neighbors_directed(node_index, Direction::Outgoing) {
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
                "Topological sort failed - DAG may contain cycles".to_string()
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
                let parents: Vec<NodeId> = self.dag
                    .parents(*node_index)
                    .iter(&self.dag)
                    .filter_map(|(_, parent_idx)| self.index_to_node_id.get(&parent_idx).copied())
                    .collect();
                
                let all_deps_ready = {
                    let results_guard = results.read().await;
                    parents.iter().all(|parent_id| results_guard.contains_key(parent_id))
                };
                
                if all_deps_ready {
                    current_level.push(node_id);
                } else {
                    next_remaining.push(node_id);
                }
            }
            
            if current_level.is_empty() {
                return Err(DagError::ExecutionError(
                    "No nodes ready to execute - possible circular dependency".to_string()
                ));
            }
            
            // Execute all nodes in current level in parallel
            let mut tasks = Vec::new();
            
            for &node_id in &current_level {
                let node = self.get_node(node_id).unwrap().clone();
                let node_index = self.node_id_to_index.get(&node_id).unwrap();
                
                // Collect inputs from parent nodes
                let parents: Vec<NodeId> = self.dag
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
        let node3_id = dag.add_node(
            "correlation".to_string(),
            NodeParams::None,
            vec![asset_b],
        );
        
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
        
        let node_id = dag.add_node(
            "moving_average".to_string(),
            params.clone(),
            vec![asset],
        );
        
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
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
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
        assert!(max_conc >= 2, "Expected at least 2 concurrent executions, got {}", max_conc);
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
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        let mut dag = AnalyticsDag::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();
        
        // Create multiple independent nodes
        let mut nodes = Vec::new();
        for i in 0..5 {
            let node_id = dag.add_node(
                format!("Node{}", i),
                NodeParams::None,
                vec![asset.clone()],
            );
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
        let a_pos = final_order.iter().position(|(id, _)| *id == node_a).unwrap();
        let b_pos = final_order.iter().position(|(id, _)| *id == node_b).unwrap();
        let c_pos = final_order.iter().position(|(id, _)| *id == node_c).unwrap();
        let d_pos = final_order.iter().position(|(id, _)| *id == node_d).unwrap();
        
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
        
        let compute_fn = |_node: Node, _inputs: Vec<NodeOutput>| async {
            Ok(NodeOutput::None)
        };
        
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
}
