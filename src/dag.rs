//! Entry point for the analytics DAG module.
//!
//! The heavy lifting lives under `core` with metadata captured in `types`.

pub mod core;
pub mod types;
pub mod visualization;

pub use core::{AnalyticsDag, DagError};
pub use types::{
    AnalyticType, Node, NodeId, NodeKey, NodeOutput, NodeParams, WindowKind, WindowSpec,
};
pub use visualization::DagVisualization;
