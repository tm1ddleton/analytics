# Tasks: REST API Server with Server-Sent Events

**Spec:** `agent-os/specs/2025-11-25-rest-api-server/spec.md`  
**Date:** 2025-11-25  
**Status:** Ready for Implementation

---

## Overview

This tasks list breaks down the REST API Server implementation into 10 task groups, ordered by dependencies and implementation complexity. Each group builds on previous work, allowing for incremental development and testing.

**Total Estimated Time:** ~12-14 hours

---

## Task Group 1: Project Setup and Dependencies

**Goal:** Set up axum server project structure and add required dependencies

**Dependencies:** None

**Acceptance Criteria:**
- Cargo.toml updated with axum dependencies
- Basic server structure created
- Can compile and run minimal server
- Health check endpoint responds

---

### Tasks

#### Task 1.1: Add dependencies to Cargo.toml
- [x] Add `axum = "0.7"` - Web framework
- [x] Add `tokio = { version = "1", features = ["full"] }` - Already present, verify features
- [x] Add `tower-http = { version = "0.5", features = ["trace", "cors"] }` - Middleware
- [x] Add `serde = { version = "1", features = ["derive"] }` - Already present
- [x] Add `serde_json = "1"` - JSON serialization
- [x] Add `uuid = { version = "1", features = ["v4", "serde"] }` - Session IDs
- [x] Add `tracing = "0.1"` - Logging
- [x] Add `tracing-subscriber = "0.3"` - Log formatting

**Estimated Time:** 15 minutes

---

#### Task 1.2: Create server module structure
- [x] Create `src/server/mod.rs`
- [x] Create `src/server/routes.rs` for route definitions
- [x] Create `src/server/handlers.rs` for endpoint handlers
- [x] Create `src/server/state.rs` for shared application state
- [x] Create `src/server/error.rs` for error types
- [x] Export server module from `src/lib.rs`

**Estimated Time:** 20 minutes

---

#### Task 1.3: Define application state structure
- [x] Define `AppState` struct in `state.rs`
- [x] Field: `data_provider: Arc<SqliteDataProvider>`
- [x] Field: `sessions: Arc<RwLock<HashMap<Uuid, ReplaySession>>>`
- [x] Field: `broadcasters: Arc<RwLock<HashMap<Uuid, Sender<Event>>>>`
- [x] Implement `AppState::new()` constructor
- [x] Add doc comments

**Estimated Time:** 20 minutes

---

#### Task 1.4: Create minimal server
- [x] Define `ServerConfig` struct with host, port, database_path
- [x] Implement `run_server()` function that takes config
- [x] Set up axum Router with basic routes
- [x] Add CORS middleware (allow all for POC)
- [x] Add tracing middleware
- [x] Add `GET /health` endpoint that returns `{"status": "ok"}`
- [x] Test server starts and health check responds

**Estimated Time:** 30 minutes

---

## Task Group 2: Error Handling and Response Types

**Goal:** Implement comprehensive error handling and JSON response structures

**Dependencies:** Task Group 1

**Acceptance Criteria:**
- All error types defined with proper HTTP status codes
- Error responses follow JSON format from spec
- Helper functions for common responses
- Error conversion traits implemented

---

### Tasks

#### Task 2.1: Define API error types
- [x] Create `ApiError` enum in `error.rs`
- [x] Variant: `AssetNotFound(String)`
- [x] Variant: `InvalidParameter(String)`
- [x] Variant: `InvalidDateRange(String)`
- [x] Variant: `ComputationFailed(String)`
- [x] Variant: `SessionNotFound(Uuid)`
- [x] Variant: `SessionLimitReached`
- [x] Variant: `InternalError(String)`
- [x] Implement `Display` trait

**Estimated Time:** 20 minutes

---

#### Task 2.2: Implement error to HTTP status mapping
- [x] Implement `IntoResponse` for `ApiError`
- [x] Map `AssetNotFound` → 404
- [x] Map `InvalidParameter` → 400
- [x] Map `InvalidDateRange` → 400
- [x] Map `SessionNotFound` → 404
- [x] Map `SessionLimitReached` → 503
- [x] Map `ComputationFailed` → 500
- [x] Map `InternalError` → 500

**Estimated Time:** 20 minutes

---

#### Task 2.3: Implement error JSON serialization
- [x] Define `ErrorResponse` struct
- [x] Fields: `error: String`, `message: String`, `details: Option<Value>`
- [x] Implement `From<ApiError>` for `ErrorResponse`
- [x] Serialize to JSON in `IntoResponse` implementation
- [x] Write tests for error formatting

**Estimated Time:** 25 minutes

---

#### Task 2.4: Convert from existing error types
- [x] Implement `From<DagError>` for `ApiError`
- [x] Implement `From<DataProviderError>` for `ApiError`
- [x] Implement `From<serde_json::Error>` for `ApiError`
- [x] Implement `From<chrono::ParseError>` for `ApiError`
- [x] Test error conversions

**Estimated Time:** 20 minutes

---

## Task Group 3: Static Information Endpoints

**Goal:** Implement endpoints that return static or simple database queries

**Dependencies:** Task Groups 1, 2

**Acceptance Criteria:**
- GET /assets returns list of assets
- GET /dag/nodes returns available analytics
- Responses match spec format
- Error handling works correctly

---

### Tasks

#### Task 3.1: Implement GET /assets endpoint
- [x] Create `list_assets` handler in `handlers.rs`
- [x] Query all assets from `SqliteDataProvider`
- [x] Map to response format with asset details
- [x] Include `data_available_from` and `data_available_to` dates
- [x] Return JSON array of asset objects
- [x] Handle database errors
- [x] Add route to router

**Estimated Time:** 35 minutes

---

#### Task 3.2: Implement GET /dag/nodes endpoint
- [x] Create `list_analytics` handler
- [x] Return hardcoded list of available analytics
- [x] Include: "returns" with no parameters
- [x] Include: "volatility" with window parameter (default 10)
- [x] Add descriptions and burn-in information
- [x] Format as JSON per spec
- [x] Add route to router

**Estimated Time:** 25 minutes

---

#### Task 3.3: Test static endpoints
- [x] Write integration test for `/assets`
- [x] Write integration test for `/dag/nodes`
- [x] Test error handling (database failure for /assets)
- [x] Test response format matches spec
- [x] Manual test with curl

**Test Count:** 4 tests

**Estimated Time:** 30 minutes

---

## Task Group 4: Asset Data Query Endpoint

**Goal:** Implement endpoint for querying raw historical price data

**Dependencies:** Task Groups 1, 2, 3

**Acceptance Criteria:**
- GET /assets/{asset}/data returns price data
- Query parameters parsed correctly
- Date range filtering works
- Returns 404 for unknown assets

---

### Tasks

#### Task 4.1: Define request/response types
- [x] Create `DataQueryParams` struct with `start` and `end` dates
- [x] Implement `FromRequest` for parsing query parameters
- [x] Create `AssetDataResponse` struct
- [x] Fields: `asset`, `start_date`, `end_date`, `data: Vec<DataPoint>`
- [x] Define `DataPoint` struct with `timestamp` and `close`
- [x] Implement serialization

**Estimated Time:** 25 minutes

---

#### Task 4.2: Implement GET /assets/{asset}/data handler
- [x] Create `get_asset_data` handler
- [x] Extract asset from path parameter
- [x] Parse query parameters (start, end dates)
- [x] Validate date range (start <= end)
- [x] Create `DateRange` from parameters
- [x] Query `SqliteDataProvider` for asset data
- [x] Convert to response format
- [x] Handle asset not found
- [x] Handle invalid dates
- [x] Add route to router

**Estimated Time:** 40 minutes

---

#### Task 4.3: Test asset data endpoint
- [x] Test successful query with valid date range
- [x] Test with different date ranges (1 day, 1 month, 1 year)
- [x] Test 404 for non-existent asset
- [x] Test 400 for invalid date format
- [x] Test 400 for invalid date range (end < start)
- [x] Test edge cases (no data in range)

**Test Count:** 6 tests

**Estimated Time:** 35 minutes

---

## Task Group 5: Pull-Mode Analytics Endpoints

**Goal:** Implement single and batch analytics query endpoints

**Dependencies:** Task Groups 1, 2, 3, 4

**Acceptance Criteria:**
- GET /analytics/{asset}/{type} executes pull-mode queries
- POST /analytics/batch handles multiple assets
- Dynamic parameter parsing works
- DAG built correctly based on analytic type

---

### Tasks

#### Task 5.1: Implement DAG builder function
- [x] Create `build_analytics_dag()` helper function
- [x] Parameters: asset, analytic_type, params map, returns (DAG, NodeId)
- [x] Handle "returns" case: DataProvider → Returns
- [x] Handle "volatility" case: DataProvider → Returns → Volatility
- [x] Extract window parameter for volatility (default 10)
- [x] Return DagError if unknown analytic type
- [x] Add doc comments with examples

**Estimated Time:** 35 minutes

---

#### Task 5.2: Define analytics request/response types
- [x] Create `AnalyticsQueryParams` struct
- [x] Fields: `start`, `end` (required), `window` (optional)
- [x] Create `AnalyticsResponse` struct
- [x] Fields: `asset`, `analytic`, `parameters`, `start_date`, `end_date`, `data`
- [x] Define `AnalyticDataPoint` with `timestamp` and `value` (nullable)
- [x] Implement serialization (NaN → null in JSON)

**Estimated Time:** 25 minutes

---

#### Task 5.3: Implement GET /analytics/{asset}/{type} handler
- [x] Create `get_analytics` handler
- [x] Extract path parameters: asset, analytic type
- [x] Parse query parameters
- [x] Validate parameters (window > 0 if provided)
- [x] Build DAG using helper function
- [x] Create DateRange from start/end
- [x] Call `dag.execute_pull_mode()`
- [x] Convert TimeSeriesPoint to response format
- [x] Handle NaN values (convert to null)
- [x] Return JSON response
- [x] Handle all error cases
- [x] Add route to router

**Estimated Time:** 45 minutes

---

#### Task 5.4: Implement POST /analytics/batch handler
- [x] Create `BatchQueryRequest` struct
- [x] Field: `queries: Vec<BatchQuery>`
- [x] Define `BatchQuery` with asset, analytic, parameters, dates
- [x] Create `BatchQueryResponse` struct
- [x] Fields: `results: Vec<AnalyticsResponse>`, `errors: Vec<BatchError>`
- [x] Create `batch_analytics` handler
- [x] Parse request body
- [x] Collect node IDs for all queries
- [x] Call `dag.execute_pull_mode_parallel()` if possible
- [x] OR execute each query sequentially for now
- [x] Collect successful results and errors
- [x] Return partial success if some fail
- [x] Add route to router

**Estimated Time:** 50 minutes

---

#### Task 5.5: Test analytics endpoints
- [x] Test GET /analytics with returns
- [x] Test GET /analytics with volatility (default window)
- [x] Test GET /analytics with custom volatility window
- [x] Test POST /analytics/batch with multiple assets
- [x] Test 404 for unknown asset
- [x] Test 400 for invalid parameters
- [x] Test 400 for invalid analytic type
- [x] Test response format (NaN → null)
- [x] Test burn-in handled correctly

**Test Count:** 9 tests

**Estimated Time:** 50 minutes

---

## Task Group 6: Replay Session Management

**Goal:** Implement session creation, status, and deletion endpoints

**Dependencies:** Task Groups 1-5

**Acceptance Criteria:**
- POST /replay creates sessions with unique IDs
- GET /replay/{id} returns session status
- DELETE /replay/{id} stops sessions
- Sessions stored in app state
- Session limit enforced (10 concurrent)

---

### Tasks

#### Task 6.1: Define session data structures
- [x] Create `ReplaySession` struct in `state.rs`
- [x] Fields: `id`, `assets`, `analytics`, `start_date`, `end_date`
- [x] Fields: `status`, `dag`, `push_engine`, `replay_engine`
- [x] Fields: `created_at`, `started_at`, `current_date`, `progress`
- [x] Create `SessionStatus` enum: Created, Running, Completed, Stopped, Error
- [x] Create `AnalyticConfig` struct for session analytics
- [x] Implement session methods: `new()`, `start()`, `stop()`

**Estimated Time:** 35 minutes

---

#### Task 6.2: Define session request/response types
- [x] Create `CreateSessionRequest` struct
- [x] Fields: `assets`, `analytics`, `start_date`, `end_date`
- [x] Create `SessionResponse` struct
- [x] Fields: `session_id`, `status`, `assets`, `analytics`, dates, `stream_url`
- [x] Create `SessionStatusResponse` with additional fields
- [x] Fields: add `current_date`, `progress`, `created_at`, `started_at`
- [x] Implement serialization

**Estimated Time:** 25 minutes

---

#### Task 6.3: Implement POST /replay handler
- [x] Create `create_replay_session` handler
- [x] Parse request body
- [x] Validate assets exist in database
- [x] Validate date range
- [x] Check session limit (max 10)
- [x] Generate UUID for session ID
- [x] Build DAG for each asset+analytic combination
- [x] Create PushModeEngine instance
- [x] Initialize push engine with historical data
- [x] Create ReplayEngine instance
- [x] Set replay delay (100ms default)
- [x] Store session in app state
- [x] Return session response with stream URL
- [x] Add route to router

**Estimated Time:** 60 minutes

---

#### Task 6.4: Implement GET /replay/{session_id} handler
- [x] Create `get_session_status` handler
- [x] Extract session ID from path
- [x] Look up session in app state
- [x] Return 404 if not found
- [x] Build status response with all fields
- [x] Include progress calculation
- [x] Add route to router

**Estimated Time:** 25 minutes

---

#### Task 6.5: Implement DELETE /replay/{session_id} handler
- [x] Create `stop_replay_session` handler
- [x] Extract session ID from path
- [x] Look up session in app state
- [x] Return 404 if not found
- [x] Check if already completed/stopped (409 Conflict)
- [x] Call stop on replay engine
- [x] Update session status to Stopped
- [x] Close SSE streams for this session
- [x] Return success response
- [x] Add route to router

**Estimated Time:** 30 minutes

---

#### Task 6.6: Test session management
- [x] Test session creation with valid parameters
- [x] Test session status query
- [x] Test session deletion
- [x] Test 404 for unknown session
- [x] Test session limit enforcement (503)
- [x] Test 400 for invalid parameters
- [x] Test 404 for non-existent assets

**Test Count:** 7 tests

**Estimated Time:** 40 minutes

---

## Task Group 7: Server-Sent Events Streaming

**Goal:** Implement SSE stream for replay session updates

**Dependencies:** Task Groups 1-6

**Acceptance Criteria:**
- GET /stream/{session_id} establishes SSE connection
- Update events sent for each computed data point
- Progress events sent periodically
- Complete/stopped events sent on termination
- Client disconnect handled gracefully

---

### Tasks

#### Task 7.1: Set up SSE infrastructure
- [x] Add `axum::response::sse` imports
- [x] Create SSE event types in separate module
- [x] Define `SseEvent` enum: Update, Progress, Complete, Stopped, Error
- [x] Implement serialization for each event type
- [x] Create channel for broadcasting (mpsc::channel)
- [x] Add broadcaster map to AppState

**Estimated Time:** 30 minutes

---

#### Task 7.2: Implement GET /stream/{session_id} handler
- [x] Create `handle_stream` handler
- [x] Extract session ID from path
- [x] Look up session, return 404 if not found
- [x] Create mpsc channel for this client
- [x] Register broadcaster in app state
- [x] Convert receiver to SSE stream
- [x] Set up keep-alive (15 second interval)
- [x] Return SSE response
- [x] Add route to router

**Estimated Time:** 35 minutes

---

#### Task 7.3: Implement broadcast helper functions
- [x] Create `broadcast_update()` function
- [x] Parameters: session_id, asset, analytic, TimeSeriesPoint
- [x] Look up broadcaster for session
- [x] Create Update event with data
- [x] Send event to all connected clients
- [x] Handle send errors gracefully
- [x] Create `broadcast_progress()` function
- [x] Create `broadcast_complete()` function
- [x] Create `broadcast_error()` function

**Estimated Time:** 30 minutes

---

#### Task 7.4: Wire SSE to push-mode callbacks
- [x] Modify session creation to register callbacks
- [x] For each node in DAG, register callback
- [x] Callback extracts node info (asset, analytic)
- [x] Callback calls broadcast_update()
- [x] Add progress callback to replay engine
- [x] Progress callback calls broadcast_progress()
- [x] Add completion callback to replay engine
- [x] Completion callback calls broadcast_complete()

**Estimated Time:** 45 minutes

---

#### Task 7.5: Handle client disconnect
- [x] Detect when SSE client disconnects
- [x] Remove broadcaster from app state
- [x] Close channel
- [x] Log disconnect event
- [x] Don't stop session (other clients may be connected)

**Estimated Time:** 20 minutes

---

#### Task 7.6: Test SSE streaming
- [x] Create test helper for connecting to SSE stream
- [x] Test connection established successfully
- [x] Test update events received
- [x] Test progress events received
- [x] Test complete event received
- [x] Test multiple clients on same session
- [x] Test client disconnect

**Test Count:** 7 tests

**Estimated Time:** 50 minutes

---

## Task Group 8: Replay Execution Integration

**Goal:** Integrate replay engine to actually run sessions and trigger updates

**Dependencies:** Task Groups 1-7

**Acceptance Criteria:**
- Replay starts automatically after session creation
- Push-mode engine receives data from replay
- Callbacks trigger SSE events
- Session status updates as replay progresses
- Session marked complete when replay finishes

---

### Tasks

#### Task 8.1: Implement replay execution task
- [x] Create `run_replay_session()` async function
- [x] Parameters: session_id, app_state
- [x] Look up session from state
- [x] Extract replay engine and push engine
- [x] Set up data callback for replay engine
- [x] Data callback calls `push_engine.push_data()`
- [x] Set up progress callback
- [x] Progress callback updates session.current_date and progress
- [x] Progress callback broadcasts progress event every N points
- [x] Set up error callback
- [x] Error callback broadcasts error event

**Estimated Time:** 45 minutes

---

#### Task 8.2: Start replay in background
- [x] Modify session creation to spawn background task
- [x] Use `tokio::spawn()` to run replay asynchronously
- [x] Pass session ID and cloned app state
- [x] Update session status to Running
- [x] Call `replay_engine.run()` in task
- [x] Handle replay completion
- [x] Update session status to Completed
- [x] Broadcast complete event
- [x] Handle replay errors
- [x] Update status to Error on failure

**Estimated Time:** 35 minutes

---

#### Task 8.3: Implement session cleanup task
- [x] Create `cleanup_sessions()` background task
- [x] Run every 1 hour
- [x] Iterate through sessions
- [x] Remove Completed sessions older than 1 hour
- [x] Remove Stopped sessions older than 1 hour
- [x] Remove Error sessions older than 15 minutes
- [x] Close associated broadcasters
- [x] Log cleanup actions
- [x] Start cleanup task when server starts

**Estimated Time:** 30 minutes

---

#### Task 8.4: Test replay integration
- [x] Test session starts replay automatically
- [x] Test SSE events received during replay
- [x] Test session status updates correctly
- [x] Test session marked complete after replay
- [x] Test cleanup removes old sessions
- [x] Integration test: full replay lifecycle

**Test Count:** 6 tests

**Estimated Time:** 55 minutes

---

## Task Group 9: Configuration and CLI

**Goal:** Make server configurable via config file or environment variables

**Dependencies:** Task Groups 1-8

**Acceptance Criteria:**
- ServerConfig loaded from file or env vars
- Command-line tool to start server
- Default values work out of box
- Can override host, port, database path

---

### Tasks

#### Task 9.1: Implement configuration loading
- [x] Add `config` crate dependency
- [x] Create default ServerConfig values
- [x] Load config from `server.toml` if exists
- [x] Load config from environment variables
- [x] Precedence: env vars > config file > defaults
- [x] Document configuration options

**Estimated Time:** 35 minutes

---

#### Task 9.2: Create server binary
- [x] Create `src/bin/analytics-server.rs`
- [x] Parse command-line arguments (--host, --port, --db)
- [x] Load ServerConfig
- [x] Initialize SqliteDataProvider
- [x] Create AppState
- [x] Call run_server()
- [x] Handle shutdown signals (Ctrl+C)
- [x] Log startup information
- [x] Test: `cargo run --bin analytics-server`

**Estimated Time:** 40 minutes

---

#### Task 9.3: Add logging configuration
- [x] Set up tracing subscriber
- [x] Log level from config or env (default: INFO)
- [x] Format logs for readability
- [x] Log all HTTP requests
- [x] Log session lifecycle events
- [x] Log errors with full context

**Estimated Time:** 25 minutes

---

## Task Group 10: Documentation and Examples

**Goal:** Create documentation and example usage

**Dependencies:** All previous task groups

**Acceptance Criteria:**
- API documentation in README
- Example curl commands work
- Example client code demonstrates SSE
- Server can be started and tested easily

---

### Tasks

#### Task 10.1: Create API documentation
- [x] Create `docs/API.md` with endpoint descriptions
- [x] Include all 9 endpoints
- [x] Show example requests and responses
- [x] Document error codes
- [x] Document SSE event format
- [x] Add authentication note (none for POC)

**Estimated Time:** 45 minutes

---

#### Task 10.2: Create example curl commands
- [x] Create `examples/api_examples.sh`
- [x] Add curl command for each endpoint
- [x] GET /assets
- [x] GET /assets/AAPL/data
- [x] GET /analytics/AAPL/returns
- [x] GET /analytics/AAPL/volatility with window
- [x] POST /analytics/batch
- [x] POST /replay
- [x] GET /replay/{id}
- [x] DELETE /replay/{id}
- [x] Add comments explaining each command

**Estimated Time:** 30 minutes

---

#### Task 10.3: Create SSE client example
- [x] Create `examples/sse_client.html`
- [x] JavaScript EventSource example
- [x] Connect to /stream endpoint
- [x] Handle update events
- [x] Handle progress events
- [x] Handle complete events
- [x] Display events in browser console
- [x] Add simple HTML UI

**Estimated Time:** 40 minutes

---

#### Task 10.4: Update main README
- [x] Add server section to README.md
- [x] Explain how to start server
- [x] Link to API documentation
- [x] Describe configuration options
- [x] Show basic usage example
- [x] Add troubleshooting section

**Estimated Time:** 30 minutes

---

#### Task 10.5: Create quick start guide
- [x] Create `docs/QUICKSTART.md`
- [x] Step 1: Download sample data
- [x] Step 2: Start server
- [x] Step 3: Query assets
- [x] Step 4: Run pull-mode analytics
- [x] Step 5: Create replay session
- [x] Step 6: Stream updates
- [x] Include expected output for each step

**Estimated Time:** 35 minutes

---

## Summary

### Task Group Completion Order

```
Task Group 1 (Setup)
    ↓
Task Group 2 (Errors)
    ↓
Task Group 3 (Static Endpoints)
    ↓
Task Group 4 (Data Query)
    ↓
Task Group 5 (Analytics)
    ↓
Task Group 6 (Session Management)
    ↓
Task Group 7 (SSE Streaming)
    ↓
Task Group 8 (Replay Integration)
    ↓
Task Group 9 (Configuration)
    ↓
Task Group 10 (Documentation)
```

### Dependencies

- **External:** None (all Rust dependencies available on crates.io)
- **Internal:** All internal analytics engine components already implemented
  - DAG framework ✅
  - Pull-mode engine ✅
  - Push-mode engine ✅
  - Replay engine ✅
  - Analytics library ✅

### Test Coverage

- **Unit Tests:** Handler functions, error conversions, serialization
- **Integration Tests:** Full request/response cycles with real database
- **Manual Tests:** curl commands, SSE streaming in browser
- **Total Estimated Tests:** ~50 tests across all task groups

### Time Estimates

- **Task Group 1:** 1.5 hours
- **Task Group 2:** 1.5 hours
- **Task Group 3:** 1.5 hours
- **Task Group 4:** 1.5 hours
- **Task Group 5:** 3.5 hours
- **Task Group 6:** 3.5 hours
- **Task Group 7:** 3.5 hours
- **Task Group 8:** 3 hours
- **Task Group 9:** 1.5 hours
- **Task Group 10:** 3 hours

**Total:** ~24 hours (3 days of focused development)

### Success Criteria

- ✅ All 9 REST endpoints implemented and tested
- ✅ SSE streaming works for replay sessions
- ✅ Can handle 5-10 concurrent clients
- ✅ Response times meet spec requirements
- ✅ Proper error handling throughout
- ✅ Documentation complete
- ✅ Ready for React UI integration (Item 10)

