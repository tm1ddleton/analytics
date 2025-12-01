# DAG Module Architecture

This module implements a Directed Acyclic Graph (DAG) computation framework for wiring analytics dependencies explicitly with cycle detection, topological sorting, and parallel execution support.

## Philosophy

The DAG module provides a graph-based execution engine that:

- **Explicit Dependencies**: Analytics dependencies are explicitly wired as edges in the graph
- **Automatic Resolution**: Node dependencies are automatically resolved and created via `AnalyticRegistry`
- **Cycle Detection**: Prevents circular dependencies through cycle detection
- **Topological Execution**: Executes nodes in dependency order using topological sorting
- **Deduplication**: Reuses nodes with identical metadata (same analytic type, assets, parameters)
- **Dual Execution Modes**: Supports both pull-mode (batch) and push-mode (incremental) execution
- **Caching**: Caches intermediate results to avoid redundant computation

## Core Components

### 1. Types (`types.rs`)

**Data structures that define nodes, edges, and metadata.**

Types provide the foundational data structures for the DAG:

- **Node**: Represents an analytics computation with ID, type, parameters, and assets
- **NodeKey**: Metadata key for node deduplication and identification
- **NodeOutput**: Execution result (time series, scalar, or collection)
- **AnalyticType**: Enum identifying the type of analytic (Returns, Volatility, etc.)
- **WindowSpec**: Windowing strategy (fixed or exponential) for lookback calculations

**Key Characteristics:**
- `NodeKey` uses hash-based deduplication (same key = same node)
- `NodeOutput` supports multiple output formats (single series, collection, scalar)
- `AnalyticType` maps to registered analytics in `AnalyticRegistry`
- `WindowSpec` defines burn-in requirements for windowed calculations

```rust
// Example: NodeKey for deduplication
let key = NodeKey {
    analytic: AnalyticType::Returns,
    assets: vec![asset_key],
    range: Some(date_range),
    window: None,
    override_tag: None,
    params: HashMap::from([("lag".to_string(), "1".to_string())]),
};

// Same key will resolve to same node ID
let node_id1 = dag.resolve_node(key.clone())?;
let node_id2 = dag.resolve_node(key)?;
assert_eq!(node_id1, node_id2); // Deduplication works
```

### 2. Core (`core.rs`)

**DAG construction, execution, and management.**

Core provides the main `AnalyticsDag` struct that:

- **Constructs the graph**: Adds nodes and edges with cycle detection
- **Resolves dependencies**: Automatically creates parent nodes via `AnalyticRegistry`
- **Executes computations**: Runs analytics in topological order
- **Manages state**: Tracks node metadata, execution cache, and topological order

**Key Methods:**
- `resolve_node(key)` - Resolves or creates node with automatic dependency resolution
- `execute_pull_mode(node_id, range, provider)` - Batch execution for date range
- `execute_pull_mode_parallel(node_id, range, provider)` - Parallel batch execution
- `add_edge(parent, child)` - Adds dependency edge with cycle detection
- `execution_order()` - Returns topological sort for execution

**Key Characteristics:**
- Uses `daggy` library for graph structure
- Maintains bidirectional maps between `NodeId` and `daggy::NodeIndex`
- Caches topological sort until DAG structure changes
- Supports both sequential and parallel execution
- Handles burn-in date range extension automatically

```rust
// Example: Creating and executing a DAG
let mut dag = AnalyticsDag::new();

// Resolve a node - dependencies are automatically created
let returns_key = NodeKey {
    analytic: AnalyticType::Returns,
    assets: vec![asset],
    range: Some(date_range),
    params: HashMap::from([("lag".to_string(), "1".to_string())]),
    // ... other fields
};

let returns_node = dag.resolve_node(returns_key)?;
// DAG now contains: DataProvider -> Lag -> Returns

// Execute in pull-mode
let results = dag.execute_pull_mode(returns_node, date_range, provider)?;
```

### 3. Visualization (`visualization.rs`)

**Serialization for frontend visualization.**

Visualization provides structures and methods to serialize the DAG for display:

- **DagVisualization**: Complete structure with nodes, edges, and metadata
- **VisualizationNode**: Node representation with URLs for data and code
- **VisualizationEdge**: Edge representation for graph visualization
- **to_visualization()**: Serializes DAG with API and code URLs

**Key Characteristics:**
- Deduplicates nodes by analytic type and assets (ignores range/params for visualization)
- Generates API URLs for querying node data
- Generates code URLs with line numbers to definition in registry
- Provides metadata for frontend layout calculation

```rust
// Example: Serializing DAG for visualization
let viz = dag.to_visualization(
    "http://localhost:3000",  // API base URL
    "https://github.com/user/repo"  // Code base URL
);

// Returns structure with:
// - nodes: Vec<VisualizationNode> with data_url and code_url
// - edges: Vec<VisualizationEdge> for graph connections
// - metadata: Node/edge counts and base URLs
```

## Key Concepts

### Node Resolution and Deduplication

Nodes are identified by `NodeKey` which includes:
- `analytic`: The type of analytic (Returns, Volatility, etc.)
- `assets`: Assets this node operates on
- `range`: Date range (optional, for pull-mode queries)
- `window`: Window specification (optional)
- `params`: Additional parameters (e.g., lag, lambda)
- `override_tag`: Override identifier (optional)

Nodes with identical `NodeKey` values are deduplicated - `resolve_node()` returns the same `NodeId` for the same key.

### Automatic Dependency Resolution

When resolving a node, the DAG:

1. Looks up the analytic definition in `AnalyticRegistry`
2. Calls `definition.dependencies(key)` to get required parent nodes
3. Recursively resolves each parent node
4. Creates edges from parents to the current node
5. Returns the node ID (or existing ID if deduplicated)

This means you only need to resolve the final node - all dependencies are created automatically.

### Execution Modes

**Pull-Mode (Batch Execution):**
- Executes complete time series for a date range
- Extends date range backward for burn-in automatically
- Uses topological sort to execute dependencies first
- Caches intermediate results to avoid recomputation
- Supports parallel execution for independent branches

**Push-Mode (Incremental Execution):**
- Executes one data point at a time
- Maintains state across incremental updates
- Simulates push-mode from calendar data for pull-mode queries
- Used internally by pull-mode for nodes that require incremental state

### Burn-in Calculation

Some analytics require historical data before producing valid output:
- **Windowed analytics**: Need N days of data for N-day window
- **Lag analytics**: Need lag+1 days for lag calculation
- **Returns**: Need 1 extra day for first return calculation

The DAG automatically:
- Calculates burn-in days for each node based on its dependencies
- Extends date ranges backward when executing pull-mode
- Handles insufficient data gracefully (returns NaN for early points)

## Data Flow

```
┌─────────────────────────────────────────────────────────┐
│              User Query (resolve_node)                   │
│                                                          │
│  User provides NodeKey for desired analytic              │
│                                                          │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│            Dependency Resolution                        │
│                                                          │
│  1. Look up AnalyticDefinition in registry              │
│  2. Get dependencies() → parent NodeKeys                │
│  3. Recursively resolve each parent                     │
│  4. Create edges: parent → current                      │
│  5. Return NodeId (deduplicated if exists)               │
│                                                          │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│            DAG Structure                                │
│                                                          │
│  DataProvider ──┐                                       │
│                 ├──> Lag ──> Returns ──> Volatility      │
│  DataProvider ──┘                                       │
│                                                          │
│  Nodes deduplicated by NodeKey                          │
│  Edges represent data dependencies                      │
│                                                          │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│            Execution (pull-mode)                        │
│                                                          │
│  1. Calculate burn-in days                              │
│  2. Extend date range backward                          │
│  3. Topological sort → execution order                   │
│  4. For each node in order:                             │
│     - Get parent outputs from cache                     │
│     - Execute node's executor                           │
│     - Cache result for children                         │
│  5. Return filtered results for target range             │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Example: Computing Volatility

**1. Resolve the target node:**
```rust
let mut dag = AnalyticsDag::new();

let volatility_key = NodeKey {
    analytic: AnalyticType::Volatility,
    assets: vec![asset],
    range: Some(date_range),
    params: HashMap::from([("window_size".to_string(), "20".to_string())]),
    window: Some(WindowSpec::fixed(20)),
    // ... other fields
};

let volatility_node = dag.resolve_node(volatility_key)?;
// DAG now contains: DataProvider -> Returns -> Volatility
```

**2. Execute in pull-mode:**
```rust
let results = dag.execute_pull_mode(
    volatility_node,
    date_range,
    provider
)?;

// Results are automatically:
// - Extended backward for burn-in (20 days for window + 1 for returns)
// - Executed in order: DataProvider → Returns → Volatility
// - Cached to avoid recomputation
// - Filtered to requested date range
```

**3. Visualize the DAG:**
```rust
let viz = dag.to_visualization(api_url, code_url);
// Returns structure ready for frontend graph visualization
```

## Benefits of This Architecture

1. **Explicit Dependencies**: Dependencies are visible in the graph structure
2. **Automatic Resolution**: No manual node creation - just resolve the target
3. **Deduplication**: Same computation = same node (efficient caching)
4. **Cycle Prevention**: Cycle detection prevents invalid dependency graphs
5. **Topological Execution**: Dependencies executed in correct order automatically
6. **Flexible Execution**: Supports both batch and incremental execution modes
7. **Parallelization**: Independent branches can execute in parallel
8. **Burn-in Handling**: Automatic date range extension for windowed analytics

## File Organization

```
src/dag/
├── types.rs         # Node, NodeKey, NodeOutput, AnalyticType, WindowSpec
├── core.rs          # AnalyticsDag, execution, dependency resolution
├── visualization.rs # DAG serialization for frontend
└── README.md        # This file
```

## Integration with Analytics Module

The DAG module integrates with the analytics module through:

- **AnalyticRegistry**: Provides definitions for dependency resolution
- **AnalyticExecutor**: Executes analytics via `execute_pull()` and `execute_push()`
- **AnalyticDefinition**: Defines dependencies and execution strategy

When resolving a node:
1. DAG looks up `AnalyticDefinition` in registry
2. Definition provides `dependencies()` → parent `NodeKey`s
3. Definition provides `executor()` → execution strategy
4. DAG creates parent nodes and wires executor

This separation allows:
- Analytics module to focus on computation logic
- DAG module to focus on graph structure and execution
- Clear boundaries between concerns

## Error Handling

The DAG uses `DagError` enum for error reporting:

- `CycleDetected`: Circular dependency detected when adding edge
- `NodeNotFound`: Referenced node doesn't exist
- `EdgeNotFound`: Referenced edge doesn't exist
- `InvalidOperation`: Invalid operation (e.g., missing definition)
- `ExecutionError`: Error during node execution
- `DataProviderError`: Error from data provider

All operations return `Result<T, DagError>` for error handling.

## Thread Safety

- `AnalyticsDag` is not thread-safe (requires `&mut` for structure changes)
- Execution methods use `&self` (immutable) for read-only execution
- Parallel execution uses `Arc<RwLock<...>>` internally for shared state
- Registry is `Arc<AnalyticRegistry>` for shared access

For concurrent access, wrap `AnalyticsDag` in `Arc<RwLock<...>>` or use separate DAG instances.

