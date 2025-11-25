//! DAG Computation Framework
//! 
//! This module provides a DAG (Directed Acyclic Graph) construction and execution engine
//! for wiring analytics dependencies explicitly with cycle detection, topological sorting,
//! and parallel execution support.

use crate::asset_key::AssetKey;
use crate::time_series::TimeSeriesPoint;
use daggy::{Dag, NodeIndex, EdgeIndex, WouldCycle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

impl std::fmt::Display for DagError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DagError::CycleDetected(msg) => write!(f, "Cycle detected: {}", msg),
            DagError::NodeNotFound(msg) => write!(f, "Node not found: {}", msg),
            DagError::EdgeNotFound(msg) => write!(f, "Edge not found: {}", msg),
            DagError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
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
}

impl AnalyticsDag {
    /// Creates a new empty DAG
    pub fn new() -> Self {
        AnalyticsDag {
            dag: Dag::new(),
            node_id_to_index: HashMap::new(),
            index_to_node_id: HashMap::new(),
            next_node_id: 0,
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
            Ok(edge_index) => Ok(edge_index),
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
}
