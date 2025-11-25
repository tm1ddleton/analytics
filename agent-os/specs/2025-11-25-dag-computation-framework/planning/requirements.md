# Spec Requirements: DAG Computation Framework

## Initial Description
DAG Computation Framework — Build DAG construction and execution engine using appropriate Rust libraries (e.g., petgraph) to wire analytics dependencies explicitly with cycle detection and topological sorting

## Requirements Discussion

### First Round Questions

**Q1:** I assume we'll use the `petgraph` crate for DAG construction and operations (as mentioned in the roadmap). Is that correct, or should we evaluate other Rust DAG libraries?
**Answer:** Evaluate daggy, dagrs, or dagx as alternatives to petgraph. DAG-based libraries might have a better API specifically for the creation of DAGs.

**Q2:** I'm thinking the DAG will represent analytics dependencies where nodes are analytics computations and edges represent data flow (e.g., "moving average depends on price data"). Should we support both single-input and multi-input analytics (e.g., a correlation that needs two time series)?
**Answer:** Definitely multi input, but a node might output a collection of timeseries (so in that sense would be single input).

**Q3:** I assume cycle detection should prevent invalid DAG construction at build time (returning an error if a cycle is detected). Should we also support dynamic DAG modification (adding/removing nodes/edges at runtime), or is the DAG static once constructed?
**Answer:** DAG should be dynamic.

**Q4:** I'm thinking topological sorting will be used to determine execution order. Should we support parallel execution of independent nodes (nodes with no dependencies on each other), or sequential execution only?
**Answer:** Yes, support parallel execution.

**Q5:** I assume each node in the DAG will represent a specific analytics computation (e.g., "20-day moving average of AAPL close prices"). Should nodes be parameterized (e.g., "N-day moving average" where N is configurable), or fixed computations?
**Answer:** Parameterized.

**Q6:** I'm thinking the DAG will need to track which assets each node operates on. Should a single DAG instance handle multiple assets (e.g., one DAG for all assets), or should each asset have its own DAG instance?
**Answer:** One DAG for all assets.

**Q7:** I assume the DAG framework will be a foundational layer that the Push-Mode Analytics Engine (next roadmap item) will build upon. Should we design the API to be ready for push-mode integration, or focus solely on DAG construction and execution for now?
**Answer:** Ready for push-mode integration.

**Q8:** Are there any specific analytics computation patterns or dependency structures we should prioritize supporting, or should we keep the framework generic and flexible?
**Answer:** Generic and flexible.

### Existing Code to Reference

No similar existing features identified for reference.

### Follow-up Questions

**Follow-up 1:** DAG library selection: You mentioned daggy, dagrs, or dagx as alternatives to petgraph. Should we evaluate all four (petgraph, daggy, dagrs, dagx) and choose the best fit, or do you have a preference? Any specific criteria (performance, API ergonomics, maintenance status)?
**Answer:** Nothing specific, but it appears that potentially the dag-based libraries might have a better API specifically for the creation of DAGs.

**Follow-up 2:** Node input/output structure: For multi-input nodes that output collections of time series, should each output time series in the collection correspond to a specific input (e.g., a correlation node takes two assets and outputs one correlation time series), or can a node produce multiple independent outputs from its inputs?
**Answer:** We should assume only one output but the output might be a collection.

**Follow-up 3:** Dynamic DAG operations: For runtime modifications (adding/removing nodes/edges), should we support: adding new nodes with dependencies on existing nodes, removing nodes (and handling dependent nodes), modifying node parameters without removing/re-adding, or all of the above?
**Answer:** Adding new nodes. Removing nodes with no dependencies.

## Visual Assets

### Files Provided:
No visual assets provided.

### Visual Insights:
No visual assets provided.

## Requirements Summary

### Functional Requirements
- Build DAG construction and execution engine for analytics dependencies
- Support explicit wiring of analytics dependencies as a directed acyclic graph
- Implement cycle detection to prevent invalid DAG construction
- Implement topological sorting to determine execution order
- Support parallel execution of independent nodes (nodes with no dependencies on each other)
- Support dynamic DAG modification:
  - Adding new nodes with dependencies on existing nodes
  - Removing nodes with no dependencies
- Support multi-input nodes (e.g., correlation that needs two time series)
- Support parameterized nodes (e.g., "N-day moving average" where N is configurable)
- Single DAG instance handles multiple assets (one DAG for all assets)
- Design API to be ready for push-mode integration (foundational layer for Push-Mode Analytics Engine)
- Keep framework generic and flexible (no specific patterns prioritized)

### Reusability Opportunities
- Evaluate existing Rust DAG libraries: petgraph, daggy, dagrs, dagx
- Focus on libraries with better APIs specifically for DAG creation
- No similar existing features in codebase to reference

### Scope Boundaries
**In Scope:**
- DAG construction and execution engine
- Cycle detection
- Topological sorting
- Parallel execution support
- Dynamic node addition (with dependencies on existing nodes)
- Dynamic node removal (nodes with no dependencies only)
- Multi-input node support
- Parameterized nodes
- Single DAG instance for all assets
- API design ready for push-mode integration

**Out of Scope:**
- Push-mode analytics implementation (next roadmap item)
- Pull-mode analytics (future roadmap item)
- Specific analytics computations (handled by Basic Analytics Library)
- Node parameter modification without remove/re-add
- Removing nodes that have dependencies
- Multiple independent outputs from a single node (single output only, though output may be a collection)

### Technical Considerations
- Evaluate Rust DAG libraries: petgraph, daggy, dagrs, dagx
- Prioritize libraries with better APIs for DAG creation
- Use tokio for async runtime and concurrent computation (from tech stack)
- Node output: single output that may be a collection of time series
- Framework should be generic and flexible to support various analytics patterns
- Must integrate with existing asset-centric data model (from Core Asset Data Model spec)
- Must work with SQLite data storage (from SQLite Data Storage spec)
- Foundation for Push-Mode Analytics Engine (next roadmap item)

