import { useMemo } from 'react';
import ReactFlow, {
  Controls,
  Background,
  MiniMap,
  Handle,
  Position,
  type Node,
  type Edge,
  type NodeTypes,
  SmoothStepEdge,
} from 'reactflow';
import 'reactflow/dist/style.css';
import { Box, Paper, Typography, Link, Chip, CircularProgress, Alert } from '@mui/material';
import type { DagVisualization as DagVisualizationType, VisualizationNode } from '../types';
import dagre from 'dagre';

// Custom node component that displays node information with links
function DagNode({ data }: { data: VisualizationNode }) {
  return (
    <Paper
      elevation={3}
      sx={{
        padding: 2,
        width: 220,
        minHeight: 180,
        backgroundColor: '#fff',
        border: '2px solid #1976d2',
        borderRadius: 2,
      }}
    >
      <Handle type="target" position={Position.Left} />
      
      <Typography variant="h6" sx={{ fontSize: '0.9rem', fontWeight: 'bold', mb: 1 }}>
        {data.node_type}
      </Typography>
      
      <Typography variant="caption" sx={{ display: 'block', color: 'text.secondary', mb: 1 }}>
        {data.analytic_type}
      </Typography>

      {data.assets.length > 0 && (
        <Box sx={{ mb: 1 }}>
          {data.assets.map((asset) => (
            <Chip
              key={asset}
              label={asset}
              size="small"
              sx={{ mr: 0.5, mb: 0.5 }}
            />
          ))}
        </Box>
      )}

      {Object.keys(data.params).length > 0 && (
        <Box sx={{ mb: 1 }}>
          {Object.entries(data.params).map(([key, value]) => (
            <Typography key={key} variant="caption" sx={{ display: 'block' }}>
              <strong>{key}:</strong> {value}
            </Typography>
          ))}
        </Box>
      )}

      {data.description && (
        <Typography variant="caption" sx={{ display: 'block', color: 'text.secondary', mb: 1 }}>
          {data.description}
        </Typography>
      )}

      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 0.5, mt: 1 }}>
        {data.data_url && (
          <Link
            href={data.data_url}
            target="_blank"
            rel="noopener noreferrer"
            sx={{ fontSize: '0.75rem' }}
          >
            📊 Query Data
          </Link>
        )}
        {data.code_url && (
          <Link
            href={data.code_url}
            target="_blank"
            rel="noopener noreferrer"
            sx={{ fontSize: '0.75rem' }}
          >
            💻 View Code
          </Link>
        )}
      </Box>

      <Handle type="source" position={Position.Right} />
    </Paper>
  );
}

// Node types for React Flow
const nodeTypes: NodeTypes = {
  dagNode: DagNode,
};

// Edge types for React Flow - use smoothstep for better routing
const edgeTypes = {
  smoothstep: SmoothStepEdge,
};

// Layout algorithm using dagre with improved spacing for edge routing
function getLayoutedElements(
  nodes: VisualizationNode[],
  edges: { source: number; target: number }[]
) {
  const dagreGraph = new dagre.graphlib.Graph();
  dagreGraph.setDefaultEdgeLabel(() => ({}));
  
  // Left to right layout with increased spacing to prevent edge-node crossings
  // Increased spacing gives edges more room to route around nodes
  const nodeWidth = 220;
  const nodeHeight = 200;
  // Add substantial padding to ensure edges have clear paths between nodes
  // This prevents edges from crossing through node boxes
  const horizontalPadding = 150; // Space for edges to route horizontally (increased for better clearance)
  const verticalPadding = 120;   // Space for edges to route vertically (increased for better clearance)
  
  dagreGraph.setGraph({ 
    rankdir: 'LR',  // Left to right
    nodesep: nodeWidth + horizontalPadding,   // Horizontal spacing between nodes (node width + padding)
    ranksep: nodeHeight + verticalPadding,    // Vertical spacing between ranks (node height + padding)
    marginx: 50,     // Horizontal margin
    marginy: 50,    // Vertical margin
    acyclicer: 'greedy', // Ensure acyclic graph
    ranker: 'tight-tree', // Better ranking algorithm for clearer layout
    // Increase edge separation to prevent overlaps
    edgesep: 50,    // Minimum separation between parallel edges
  });

  // Add nodes to dagre graph with actual dimensions
  nodes.forEach((node) => {
    dagreGraph.setNode(node.id.toString(), { 
      width: nodeWidth, 
      height: nodeHeight 
    });
  });

  // Add edges to dagre graph
  edges.forEach((edge) => {
    dagreGraph.setEdge(edge.source.toString(), edge.target.toString());
  });

  dagre.layout(dagreGraph);

  // Convert to React Flow format
  const layoutedNodes: Node[] = nodes.map((node) => {
    const nodeWithPosition = dagreGraph.node(node.id.toString());
    return {
      id: node.id.toString(),
      type: 'dagNode',
      position: {
        x: nodeWithPosition.x - nodeWidth / 2, // Center the node horizontally
        y: nodeWithPosition.y - nodeHeight / 2, // Center the node vertically
      },
      data: node,
    };
  });

  // Use smoothstep edges which route better and avoid crossing node centers
  // smoothstep edges create stepped paths that naturally avoid node boxes
  const layoutedEdges: Edge[] = edges.map((edge, index) => ({
    id: `e${edge.source}-${edge.target}-${index}`,
    source: edge.source.toString(),
    target: edge.target.toString(),
    type: 'smoothstep', // Stepped edges that route around obstacles better
    animated: true,
    style: { stroke: '#1976d2', strokeWidth: 2 },
    markerEnd: {
      type: 'arrowclosed',
      color: '#1976d2',
    },
  }));

  return { nodes: layoutedNodes, edges: layoutedEdges };
}

interface DagVisualizationProps {
  dag: DagVisualizationType | null;
  loading: boolean;
  error: string | null;
}

export function DagVisualization({ dag, loading, error }: DagVisualizationProps) {
  const { nodes, edges } = useMemo(() => {
    if (!dag) {
      return { nodes: [], edges: [] };
    }
    return getLayoutedElements(dag.nodes, dag.edges);
  }, [dag]);

  if (loading) {
    return (
      <Box display="flex" justifyContent="center" alignItems="center" minHeight={400}>
        <CircularProgress />
      </Box>
    );
  }

  if (error) {
    return (
      <Alert severity="error" sx={{ mt: 2 }}>
        {error}
      </Alert>
    );
  }

  if (!dag || dag.nodes.length === 0) {
    return (
      <Alert severity="info" sx={{ mt: 2 }}>
        Select an asset and analytic to visualize the DAG
      </Alert>
    );
  }

  return (
    <Box sx={{ width: '100%', height: '700px', mt: 2 }}>
      <Box sx={{ mb: 2 }}>
        <Typography variant="body2" color="text.secondary">
          DAG: {dag.metadata.node_count} nodes, {dag.metadata.edge_count} edges
        </Typography>
      </Box>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        fitView
        fitViewOptions={{ padding: 0.2, maxZoom: 1.5 }}
        attributionPosition="bottom-left"
        defaultEdgeOptions={{
          type: 'smoothstep',
          animated: true,
        }}
        connectionLineType="smoothstep"
        // Prevent edges from intersecting with nodes by using proper routing
        nodeExtent={undefined} // Allow nodes anywhere
      >
        <Controls />
        <MiniMap />
        <Background />
      </ReactFlow>
    </Box>
  );
}

