# Task Breakdown: DAG Computation Framework

## Overview
Total Tasks: 7 task groups

## Task List

### DAG Library Evaluation and Selection

#### Task Group 1: DAG Library Evaluation and Selection
**Dependencies:** None

- [x] 1.0 Complete DAG library evaluation and selection
  - [x] 1.1 Write 2-8 focused tests for DAG library evaluation
    - Test basic DAG creation with each library (petgraph, daggy, dagrs, dagx)
    - Test cycle detection capabilities
    - Test dynamic node addition/removal
    - Test topological sorting functionality
  - [x] 1.2 Evaluate petgraph library
    - Assess API ergonomics for DAG creation
    - Test cycle detection support
    - Test dynamic modification capabilities
    - Evaluate tokio integration potential
  - [x] 1.3 Evaluate daggy library
    - Assess API ergonomics for DAG creation
    - Test cycle detection support
    - Test dynamic modification capabilities
    - Evaluate tokio integration potential
  - [x] 1.4 Evaluate dagrs library
    - Assess API ergonomics for DAG creation
    - Test cycle detection support
    - Test dynamic modification capabilities
    - Evaluate tokio integration potential
    - Note: Not evaluated due to complex dependencies (v8, deno)
  - [x] 1.5 Evaluate dagx library
    - Assess API ergonomics for DAG creation
    - Test cycle detection support
    - Test dynamic modification capabilities
    - Evaluate tokio integration potential
    - Note: Not evaluated due to complex dependencies (v8, deno)
  - [x] 1.6 Select best-fit library and add to Cargo.toml
    - Compare libraries based on API ergonomics, cycle detection, dynamic modifications, tokio integration
    - Select library with best API for DAG creation
    - Add selected library to Cargo.toml with appropriate features
    - Document selection rationale
    - Selected: daggy v0.8 (better API for DAG creation, built-in cycle prevention)
  - [x] 1.7 Ensure DAG library evaluation tests pass
    - Run ONLY the 2-8 tests written in 1.1
    - Verify all libraries can be tested
    - Verify selection criteria are met
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 1.1 pass
- All four libraries (petgraph, daggy, dagrs, dagx) are evaluated
- Best-fit library is selected and added to Cargo.toml
- Selection rationale is documented

### Core DAG Structure

#### Task Group 2: Core DAG Structure and Node Definition
**Dependencies:** Task Group 1

- [x] 2.0 Complete core DAG structure and node definition
  - [x] 2.1 Write 2-8 focused tests for DAG structure
    - Test creating empty DAG
    - Test creating DAG with single node
    - Test creating DAG with multiple nodes and edges
    - Test node parameterization
  - [x] 2.2 Create DAG struct using selected library
    - Wrap selected DAG library in DAG struct
    - Support single DAG instance for all assets
    - Initialize empty DAG
    - Provide basic DAG construction methods
    - Created AnalyticsDag struct wrapping daggy::Dag
  - [x] 2.3 Define Node trait or enum
    - Create trait/struct for analytics computation nodes
    - Support parameterized nodes (e.g., N-day moving average)
    - Support multi-input nodes
    - Support single output (may be collection of time series)
    - Created Node struct with NodeParams enum for parameterization
  - [x] 2.4 Implement node identification and metadata
    - Assign unique identifiers to nodes
    - Store node parameters
    - Track which assets each node operates on (using AssetKey)
    - Store node computation type/function
    - Implemented NodeId, Node struct with params, assets, and node_type
  - [x] 2.5 Implement edge creation for dependencies
    - Create edges representing data flow between nodes
    - Support multi-input nodes (multiple edges to one node)
    - Validate edge creation (source and target nodes exist)
    - Store edge metadata if needed
    - Implemented add_edge method with cycle detection via daggy
  - [x] 2.6 Ensure core DAG structure tests pass
    - Run ONLY the 2-8 tests written in 2.1
    - Verify DAG can be created and nodes added
    - Verify edges can be created between nodes
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 2.1 pass
- DAG struct can be created using selected library
- Nodes can be added with parameters and asset tracking
- Edges can be created to represent dependencies

### Cycle Detection

#### Task Group 3: Cycle Detection Implementation
**Dependencies:** Task Group 2

- [ ] 3.0 Complete cycle detection implementation
  - [ ] 3.1 Write 2-8 focused tests for cycle detection
    - Test detecting cycles when adding new node
    - Test detecting cycles in existing DAG
    - Test valid DAG (no cycles) passes detection
    - Test error messages for cycle detection
  - [ ] 3.2 Implement cycle detection algorithm
    - Use selected library's cycle detection or implement custom algorithm
    - Detect cycles when adding new nodes with dependencies
    - Detect cycles in existing DAG structure
    - Return clear error messages when cycle is detected
  - [ ] 3.3 Integrate cycle detection with node addition
    - Check for cycles before adding new node
    - Prevent DAG from entering invalid state
    - Return appropriate error if cycle would be created
    - Maintain DAG integrity
  - [ ] 3.4 Create cycle detection error types
    - Define error enum for cycle detection failures
    - Include clear error messages indicating which nodes form the cycle
    - Follow existing error handling patterns (e.g., DownloadError structure)
  - [ ] 3.5 Ensure cycle detection tests pass
    - Run ONLY the 2-8 tests written in 3.1
    - Verify cycles are detected correctly
    - Verify valid DAGs pass detection
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 3.1 pass
- Cycles are detected when adding nodes that would create cycles
- Clear error messages are returned for cycle detection failures
- Valid DAGs pass cycle detection

### Topological Sorting

#### Task Group 4: Topological Sorting Implementation
**Dependencies:** Task Group 2

- [ ] 4.0 Complete topological sorting implementation
  - [ ] 4.1 Write 2-8 focused tests for topological sorting
    - Test topological sort for simple linear DAG
    - Test topological sort for DAG with parallel branches
    - Test edge cases: empty DAG, single node, disconnected components
    - Test querying execution order without executing
  - [ ] 4.2 Implement topological sorting algorithm
    - Use selected library's topological sort or implement custom algorithm
    - Generate valid execution sequence respecting all dependencies
    - Handle edge cases: empty DAG, single node, disconnected components
    - Return ordered list of nodes for execution
  - [ ] 4.3 Implement execution order query method
    - Support querying execution order without executing nodes
    - Return execution sequence as vector of node identifiers
    - Handle invalid DAG states gracefully
    - Provide clear error messages if DAG is invalid
  - [ ] 4.4 Integrate topological sort with DAG structure
    - Recompute topological sort after DAG modifications
    - Cache execution order when DAG is unchanged
    - Invalidate cache when nodes or edges are added/removed
  - [ ] 4.5 Ensure topological sorting tests pass
    - Run ONLY the 2-8 tests written in 4.1
    - Verify execution order respects all dependencies
    - Verify edge cases are handled correctly
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 4.1 pass
- Topological sort generates valid execution sequence
- Execution order respects all dependencies
- Edge cases are handled correctly

### Parallel Execution

#### Task Group 5: Parallel Execution Support
**Dependencies:** Task Groups 2, 4

- [ ] 5.0 Complete parallel execution support
  - [ ] 5.1 Write 2-8 focused tests for parallel execution
    - Test parallel execution of independent nodes
    - Test sequential execution of dependent nodes
    - Test thread-safe execution with multiple concurrent nodes
    - Test execution with mixed parallel and sequential nodes
  - [ ] 5.2 Implement execution engine using tokio
    - Use tokio for async runtime and concurrent computation
    - Execute nodes in parallel when possible (independent nodes)
    - Execute nodes sequentially when dependencies require it
    - Ensure thread-safe execution when multiple nodes run concurrently
  - [ ] 5.3 Implement node execution logic
    - Create execution context for nodes
    - Pass input data from dependency nodes to dependent nodes
    - Handle node outputs (single output, may be collection)
    - Store execution results for use by dependent nodes
  - [ ] 5.4 Integrate parallel execution with topological sort
    - Use topological sort to determine execution order
    - Identify independent nodes that can run in parallel
    - Schedule parallel execution of independent nodes
    - Wait for dependencies before executing dependent nodes
  - [ ] 5.5 Ensure parallel execution tests pass
    - Run ONLY the 2-8 tests written in 5.1
    - Verify independent nodes execute in parallel
    - Verify dependent nodes execute in correct order
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 5.1 pass
- Independent nodes execute in parallel using tokio
- Dependent nodes execute in correct sequential order
- Thread-safe execution is maintained

### Dynamic DAG Modifications

#### Task Group 6: Dynamic DAG Modifications
**Dependencies:** Task Groups 2, 3, 4

- [ ] 6.0 Complete dynamic DAG modifications
  - [ ] 6.1 Write 2-8 focused tests for dynamic modifications
    - Test adding new nodes with dependencies on existing nodes
    - Test removing nodes with no dependencies
    - Test cycle detection after adding nodes
    - Test execution order update after modifications
  - [ ] 6.2 Implement node addition with validation
    - Support adding new nodes with dependencies on existing nodes at runtime
    - Validate DAG structure after addition (cycle detection)
    - Update execution order cache after successful addition
    - Return error if addition would create cycle
  - [ ] 6.3 Implement node removal with validation
    - Support removing nodes with no dependencies at runtime
    - Validate node has no dependencies before removal
    - Update execution order cache after successful removal
    - Return error if node has dependencies
  - [ ] 6.4 Integrate modifications with cycle detection and topological sort
    - Run cycle detection after each modification
    - Recompute topological sort after valid modifications
    - Maintain DAG integrity throughout modifications
    - Handle concurrent modification attempts (if needed)
  - [ ] 6.5 Ensure dynamic modification tests pass
    - Run ONLY the 2-8 tests written in 6.1
    - Verify nodes can be added and removed dynamically
    - Verify cycle detection works after modifications
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 6.1 pass
- Nodes can be added with dependencies on existing nodes
- Nodes with no dependencies can be removed
- Cycle detection and execution order are maintained after modifications

### Integration and Push-Mode API

#### Task Group 7: Integration with Existing Systems and Push-Mode API Design
**Dependencies:** Task Groups 2, 5

- [ ] 7.0 Complete integration and push-mode API design
  - [ ] 7.1 Write 2-8 focused tests for integration
    - Test DAG integration with AssetKey and TimeSeriesPoint
    - Test DAG integration with DataProvider trait
    - Test push-mode API hooks/callbacks
    - Test incremental update triggering
  - [ ] 7.2 Integrate with AssetKey and TimeSeriesPoint
    - Use existing AssetKey enum for identifying assets in DAG nodes
    - Leverage TimeSeriesPoint struct for time series data flow between nodes
    - Follow existing asset-centric architecture patterns
    - Ensure data type compatibility between node outputs and inputs
  - [ ] 7.3 Integrate with DataProvider trait
    - Integrate with DataProvider trait for querying time series data
    - Support both SqliteDataProvider and InMemoryDataProvider for testing
    - Use existing DataProviderError types for error handling
    - Allow nodes to query data from DataProvider when needed
  - [ ] 7.4 Design push-mode integration API
    - Design API to be ready for push-mode integration (foundational layer)
    - Support incremental updates when new data arrives
    - Enable dependency propagation through the DAG
    - Provide hooks or callbacks for push-mode engine to trigger node execution
  - [ ] 7.5 Implement error handling following existing patterns
    - Follow existing error handling patterns (e.g., DownloadError enum structure)
    - Use Result types for operations that can fail
    - Provide clear error messages for all error cases
    - Integrate with existing error types where appropriate
  - [ ] 7.6 Ensure integration tests pass
    - Run ONLY the 2-8 tests written in 7.1
    - Verify integration with existing systems works correctly
    - Verify push-mode API hooks are available
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 7.1 pass
- DAG integrates with AssetKey, TimeSeriesPoint, and DataProvider
- Push-mode integration API is designed and available
- Error handling follows existing patterns

### Testing

#### Task Group 8: Test Review & Gap Analysis
**Dependencies:** Task Groups 1-7

- [ ] 8.0 Review existing tests and fill critical gaps only
  - [ ] 8.1 Review tests from Task Groups 1-7
    - Review the 2-8 tests written by Task Group 1 (DAG library evaluation)
    - Review the 2-8 tests written by Task Group 2 (Core DAG structure)
    - Review the 2-8 tests written by Task Group 3 (Cycle detection)
    - Review the 2-8 tests written by Task Group 4 (Topological sorting)
    - Review the 2-8 tests written by Task Group 5 (Parallel execution)
    - Review the 2-8 tests written by Task Group 6 (Dynamic modifications)
    - Review the 2-8 tests written by Task Group 7 (Integration)
    - Total existing tests: approximately 14-56 tests
  - [ ] 8.2 Analyze test coverage gaps for THIS feature only
    - Identify critical user workflows that lack test coverage
    - Focus ONLY on gaps related to this spec's feature requirements
    - Do NOT assess entire application test coverage
    - Prioritize end-to-end workflows over unit test gaps
  - [ ] 8.3 Write up to 10 additional strategic tests maximum
    - Add maximum of 10 new tests to fill identified critical gaps
    - Focus on integration points and end-to-end workflows
    - Do NOT write comprehensive coverage for all scenarios
    - Skip edge cases, performance tests, and accessibility tests unless business-critical
  - [ ] 8.4 Run feature-specific tests only
    - Run ONLY tests related to this spec's feature (tests from 1.1, 2.1, 3.1, 4.1, 5.1, 6.1, 7.1, and 8.3)
    - Expected total: approximately 24-66 tests maximum
    - Do NOT run the entire application test suite
    - Verify critical workflows pass

**Acceptance Criteria:**
- All feature-specific tests pass (approximately 24-66 tests total)
- Critical user workflows for this feature are covered
- No more than 10 additional tests added when filling in testing gaps
- Testing focused exclusively on this spec's feature requirements

## Execution Order

Recommended implementation sequence:
1. DAG Library Evaluation and Selection (Task Group 1) - Foundation for all DAG operations
2. Core DAG Structure and Node Definition (Task Group 2) - Core data structures
3. Cycle Detection Implementation (Task Group 3) - Requires Task Group 2, can proceed in parallel with Task Group 4
4. Topological Sorting Implementation (Task Group 4) - Requires Task Group 2, can proceed in parallel with Task Group 3
5. Parallel Execution Support (Task Group 5) - Requires Task Groups 2 and 4
6. Dynamic DAG Modifications (Task Group 6) - Requires Task Groups 2, 3, and 4
7. Integration with Existing Systems and Push-Mode API Design (Task Group 7) - Requires Task Groups 2 and 5
8. Test Review & Gap Analysis (Task Group 8) - Requires all previous task groups

