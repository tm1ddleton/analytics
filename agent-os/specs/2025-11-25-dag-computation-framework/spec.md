# Specification: DAG Computation Framework

## Goal
Build a DAG construction and execution engine that wires analytics dependencies explicitly using Rust DAG libraries, with cycle detection, topological sorting, and parallel execution support, designed as a foundational layer for push-mode analytics integration.

## User Stories
- As a quantitative trader, I want to wire analytics computations as a DAG so that dependencies are explicit and execution order is automatically determined
- As a system developer, I want a DAG framework that supports dynamic modifications and parallel execution so that analytics can be added and computed efficiently at runtime
- As a research analyst, I want a flexible DAG framework that supports parameterized nodes and multi-input analytics so that I can build complex analytical computations

## Specific Requirements

**DAG Library Selection**
- Evaluate Rust DAG libraries: petgraph, daggy, dagrs, dagx
- Prioritize libraries with better APIs specifically for DAG creation
- Select library that best supports dynamic DAG modifications and cycle detection
- Ensure chosen library integrates well with tokio for async/parallel execution

**DAG Construction**
- Support explicit wiring of analytics dependencies as a directed acyclic graph
- Nodes represent analytics computations (e.g., moving averages, correlations)
- Edges represent data flow dependencies between nodes
- Single DAG instance handles multiple assets (one DAG for all assets)
- Support parameterized nodes (e.g., "N-day moving average" where N is configurable)

**Node Input/Output Structure**
- Support multi-input nodes (e.g., correlation that needs two time series)
- Each node has single output (though output may be a collection of time series)
- Nodes can output collections of time series from their inputs
- Framework must handle data type compatibility between node outputs and inputs

**Cycle Detection**
- Implement cycle detection to prevent invalid DAG construction
- Detect cycles when adding new nodes with dependencies
- Return clear error messages when cycle is detected
- Prevent DAG from entering invalid state with circular dependencies

**Topological Sorting**
- Implement topological sorting to determine execution order
- Generate valid execution sequence respecting all dependencies
- Handle edge cases: empty DAG, single node, disconnected components
- Support querying execution order without executing

**Parallel Execution**
- Support parallel execution of independent nodes (nodes with no dependencies on each other)
- Use tokio for async runtime and concurrent computation
- Execute nodes in parallel when possible to optimize performance
- Ensure thread-safe execution when multiple nodes run concurrently

**Dynamic DAG Modifications**
- Support adding new nodes with dependencies on existing nodes at runtime
- Support removing nodes with no dependencies at runtime
- Validate DAG structure after each modification (cycle detection)
- Maintain execution order after modifications (recompute topological sort if needed)

**Push-Mode Integration API**
- Design API to be ready for push-mode integration (foundational layer for Push-Mode Analytics Engine)
- Support incremental updates when new data arrives
- Enable dependency propagation through the DAG
- Provide hooks or callbacks for push-mode engine to trigger node execution

**Generic and Flexible Framework**
- Keep framework generic and flexible (no specific patterns prioritized)
- Support various analytics computation patterns
- Allow custom node types and computation logic
- Do not hardcode specific analytics (handled by Basic Analytics Library)

**Integration with Existing Systems**
- Integrate with existing asset-centric data model (AssetKey, TimeSeriesPoint)
- Work with SQLite data storage via DataProvider trait
- Use serde for serialization if needed for API communication
- Follow existing Rust coding conventions and error handling patterns

## Visual Design
No visual assets provided.

## Existing Code to Leverage

**AssetKey and TimeSeriesPoint structures**
- Use existing AssetKey enum for identifying assets in DAG nodes
- Leverage TimeSeriesPoint struct for time series data flow between nodes
- Follow existing asset-centric architecture patterns

**DataProvider trait**
- Integrate with DataProvider trait for querying time series data
- Support both SqliteDataProvider and InMemoryDataProvider for testing
- Use existing DataProviderError types for error handling

**Tokio async runtime**
- Use tokio for async runtime and concurrent computation
- Follow existing async patterns used in YahooFinanceDownloader
- Leverage tokio's parallel execution capabilities

**Error handling patterns**
- Follow existing error handling patterns (e.g., DownloadError enum structure)
- Use Result types for operations that can fail
- Provide clear error messages for cycle detection and invalid operations

## Out of Scope
- Push-mode analytics implementation (next roadmap item - Push-Mode Analytics Engine)
- Pull-mode analytics (future roadmap item)
- Specific analytics computations (handled by Basic Analytics Library roadmap item)
- Node parameter modification without remove/re-add (must remove and re-add node to change parameters)
- Removing nodes that have dependencies (only nodes with no dependencies can be removed)
- Multiple independent outputs from a single node (single output only, though output may be a collection)
- DAG visualization or UI components (handled by React UI Dashboard roadmap item)
- Persistence of DAG structure to storage (DAG exists in memory only for POC)
- DAG serialization/deserialization (not required for POC phase)
- Distributed DAG execution across multiple nodes (handled by Distributed Architecture Foundation roadmap item)

