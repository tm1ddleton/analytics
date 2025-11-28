//! DAG Visualization Support
//!
//! This module provides functionality to serialize DAG structures for visualization
//! in the frontend, including node metadata, edges, and links to query data and code.

use crate::dag::types::{AnalyticType, Node, NodeId, NodeKey, NodeParams};
use crate::dag::AnalyticsDag;
use crate::time_series::DateRange;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Finds the line number of a struct definition in the registry file
/// The file is included at compile time via include_str!
fn find_definition_line(analytic_type: AnalyticType) -> Option<usize> {
    // Include the registry.rs file at compile time
    const REGISTRY_FILE: &str = include_str!("../analytics/registry.rs");
    
    // Search patterns for each definition
    let search_pattern = match analytic_type {
        AnalyticType::DataProvider => "struct DataProviderDefinition",
        AnalyticType::Returns => "struct ReturnsDefinition",
        AnalyticType::Volatility => "struct VolatilityDefinition",
        AnalyticType::Lag => "struct LagDefinition",
        _ => return None,
    };
    
    // Find the line number (1-indexed)
    for (line_num, line) in REGISTRY_FILE.lines().enumerate() {
        if line.trim().starts_with(search_pattern) {
            return Some(line_num + 1); // Convert to 1-indexed
        }
    }
    
    None
}

/// Represents a node in the DAG visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationNode {
    /// Node ID
    pub id: usize,
    /// Node type (e.g., "returns", "volatility", "data_provider")
    pub node_type: String,
    /// Analytic type enum
    pub analytic_type: String,
    /// Assets this node operates on
    pub assets: Vec<String>,
    /// Parameters for this node
    pub params: HashMap<String, String>,
    /// Position for visualization (optional, can be calculated by frontend)
    pub position: Option<NodePosition>,
    /// URL to query data for this node
    pub data_url: Option<String>,
    /// URL to view code for this node type
    pub code_url: Option<String>,
    /// Description of what this node does
    pub description: Option<String>,
}

/// Position of a node in the visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

/// Represents an edge in the DAG visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationEdge {
    /// Source node ID
    pub source: usize,
    /// Target node ID
    pub target: usize,
    /// Edge label (optional)
    pub label: Option<String>,
}

/// Complete DAG visualization structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagVisualization {
    /// All nodes in the DAG
    pub nodes: Vec<VisualizationNode>,
    /// All edges in the DAG
    pub edges: Vec<VisualizationEdge>,
    /// Metadata about the DAG
    pub metadata: DagMetadata,
}

/// Metadata about the DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagMetadata {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Base URL for API endpoints
    pub api_base_url: String,
    /// Base URL for code repository
    pub code_base_url: String,
}

impl AnalyticsDag {
    /// Serializes the DAG for visualization
    ///
    /// This is a generic function that takes the DAG and returns a structure
    /// suitable for visualization in the frontend, including links to query
    /// data and view code.
    ///
    /// # Arguments
    /// * `api_base_url` - Base URL for API endpoints (e.g., "http://localhost:3000")
    /// * `code_base_url` - Base URL for code repository (e.g., "https://github.com/user/repo")
    ///
    /// # Returns
    /// A `DagVisualization` structure containing nodes, edges, and metadata
    pub fn to_visualization(
        &self,
        api_base_url: &str,
        code_base_url: &str,
    ) -> DagVisualization {
        // Map from NodeKey to a unique visualization node ID
        let mut key_to_viz_id: HashMap<NodeKey, usize> = HashMap::new();
        let mut node_id_to_viz_id: HashMap<NodeId, usize> = HashMap::new();
        let mut nodes = Vec::new();
        let mut next_viz_id = 0;

        // First pass: collect unique nodes by their analytic type and assets
        // This merges duplicate nodes (e.g., multiple DataProvider nodes for the same asset)
        for node_id in self.node_ids() {
            if let Some(node) = self.get_node(node_id) {
                if let Some(key) = self.node_key(node_id) {
                    // Create a simplified key for deduplication: analytic type + assets
                    // This ensures nodes with same type and assets are merged
                    let dedup_key = NodeKey {
                        analytic: key.analytic,
                        assets: key.assets.clone(),
                        range: None, // Ignore range for deduplication
                        window: None, // Ignore window for deduplication
                        override_tag: None, // Ignore override_tag for deduplication
                        params: HashMap::new(), // Ignore params for deduplication
                    };
                    
                    // Check if we've already seen this simplified key
                    let viz_id = if let Some(&existing_id) = key_to_viz_id.get(&dedup_key) {
                        // Use existing visualization node ID
                        existing_id
                    } else {
                        // Create new visualization node
                        let new_viz_id = next_viz_id;
                        next_viz_id += 1;
                        key_to_viz_id.insert(dedup_key, new_viz_id);
                        new_viz_id
                    };
                    
                    node_id_to_viz_id.insert(node_id, viz_id);
                    
                    // Only create the visualization node if this is the first time we see this key
                    if viz_id == nodes.len() {
                        let analytic_type = format!("{:?}", key.analytic);
                        
                        // Extract and filter valid API parameters (exclude internal metadata)
                        let valid_params = match &node.params {
                            NodeParams::Map(map) => {
                                // Filter out internal metadata parameters
                                map.iter()
                                    .filter(|(k, _)| {
                                        // Only include actual API parameters
                                        matches!(k.as_str(), "window" | "lag" | "override")
                                    })
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                    .collect::<HashMap<_, _>>()
                            }
                            NodeParams::None => HashMap::new(),
                        };
                        
                        // Build data URL with query parameters
                        let data_url = if let Some(asset) = node.assets.first() {
                            let mut url = match key.analytic {
                                AnalyticType::DataProvider => {
                                    // DataProvider uses /assets/{asset}/data endpoint
                                    format!("{}/assets/{}/data", api_base_url, asset)
                                }
                                _ => {
                                    // Analytics use /analytics/{asset}/{type} endpoint
                                    format!(
                                        "{}/analytics/{}/{}",
                                        api_base_url,
                                        asset,
                                        key.analytic.to_string()
                                    )
                                }
                            };
                            
                            // Add query parameters to data URL
                            let mut query_params = Vec::new();
                            
                            // Add date range if available
                            if let Some(range) = &key.range {
                                query_params.push(format!("start={}", range.start));
                                query_params.push(format!("end={}", range.end));
                            }
                            
                            // Add valid parameters
                            for (k, v) in &valid_params {
                                query_params.push(format!("{}={}", k, v));
                            }
                            
                            if !query_params.is_empty() {
                                url.push_str(&format!("?{}", query_params.join("&")));
                            }
                            
                            Some(url)
                        } else {
                            None
                        };

                        // Build code URL with line number anchor to the specific definition
                        let code_url = {
                            let base_file = "src/analytics/registry.rs";
                            let branch = "initial";
                            // Ensure code_base_url doesn't have trailing slash
                            let base = code_base_url.trim_end_matches('/');
                            
                            // Find line number at runtime (file content included at compile time)
                            if let Some(line) = find_definition_line(key.analytic) {
                                let url = format!("{}/blob/{}/{}#L{}", base, branch, base_file, line);
                                Some(url)
                            } else {
                                None
                            }
                        };

                        // Get description from node type
                        let description = Some(format!("{:?} analytic", key.analytic));

                        nodes.push(VisualizationNode {
                            id: viz_id,
                            node_type: node.node_type.clone(),
                            analytic_type,
                            assets: node.assets.iter().map(|a| a.to_string()).collect(),
                            params: valid_params, // Only valid API parameters, no internal metadata
                            position: None, // Frontend can calculate layout
                            data_url,
                            code_url,
                            description,
                        });
                    }
                }
            }
        }

        // Second pass: collect edges, mapping node IDs to visualization IDs
        // Use a set to deduplicate edges
        let mut edge_set: HashMap<(usize, usize), ()> = HashMap::new();
        let mut edges = Vec::new();
        
        for node_id in self.node_ids() {
            if let Some(&source_viz_id) = node_id_to_viz_id.get(&node_id) {
                let children = self.get_children(node_id);
                for child_id in children {
                    if let Some(&target_viz_id) = node_id_to_viz_id.get(&child_id) {
                        // Only add edge if source and target are different (avoid self-loops)
                        if source_viz_id != target_viz_id {
                            let edge_key = (source_viz_id, target_viz_id);
                            if !edge_set.contains_key(&edge_key) {
                                edge_set.insert(edge_key, ());
                                edges.push(VisualizationEdge {
                                    source: source_viz_id,
                                    target: target_viz_id,
                                    label: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        let node_count = nodes.len();
        let edge_count = edges.len();
        
        DagVisualization {
            nodes,
            edges,
            metadata: DagMetadata {
                node_count,
                edge_count,
                api_base_url: api_base_url.to_string(),
                code_base_url: code_base_url.to_string(),
            },
        }
    }
}

