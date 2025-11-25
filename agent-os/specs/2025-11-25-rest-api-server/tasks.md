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
- [ ] Add `axum = "0.7"` - Web framework
- [ ] Add `tokio = { version = "1", features = ["full"] }` - Already present, verify features
- [ ] Add `tower-http = { version = "0.5", features = ["trace", "cors"] }` - Middleware
- [ ] Add `serde = { version = "1", features = ["derive"] }` - Already present
- [ ] Add `serde_json = "1"` - JSON serialization
- [ ] Add `uuid = { version = "1", features = ["v4", "serde"] }` - Session IDs
- [ ] Add `tracing = "0.1"` - Logging
- [ ] Add `tracing-subscriber = "0.3"` - Log formatting

**Estimated Time:** 15 minutes

---

#### Task 1.2: Create server module structure
- [ ] Create `src/server/mod.rs`
- [ ] Create `src/server/routes.rs` for route definitions
- [ ] Create `src/server/handlers.rs` for endpoint handlers
- [ ] Create `src/server/state.rs` for shared application state
- [ ] Create `src/server/error.rs` for error types
- [ ] Export server module from `src/lib.rs`

**Estimated Time:** 20 minutes

---

#### Task 1.3: Define application state structure
- [ ] Define `AppState` struct in `state.rs`
- [ ] Field: `data_provider: Arc<SqliteDataProvider>`
- [ ] Field: `sessions: Arc<RwLock<HashMap<Uuid, ReplaySession>>>`
- [ ] Field: `broadcasters: Arc<RwLock<HashMap<Uuid, Sender<Event>>>>`
- [ ] Implement `AppState::new()` constructor
- [ ] Add doc comments

**Estimated Time:** 20 minutes

---

#### Task 1.4: Create minimal server
- [ ] Define `ServerConfig` struct with host, port, database_path
- [ ] Implement `run_server()` function that takes config
- [ ] Set up axum Router with basic routes
- [ ] Add CORS middleware (allow all for POC)
- [ ] Add tracing middleware
- [ ] Add `GET /health` endpoint that returns `{"status": "ok"}`
- [ ] Test server starts and health check responds

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
- [ ] Create `ApiError` enum in `error.rs`
- [ ] Variant: `AssetNotFound(String)`
- [ ] Variant: `InvalidParameter(String)`
- [ ] Variant: `InvalidDateRange(String)`
- [ ] Variant: `ComputationFailed(String)`
- [ ] Variant: `SessionNotFound(Uuid)`
- [ ] Variant: `SessionLimitReached`
- [ ] Variant: `InternalError(String)`
- [ ] Implement `Display` trait

**Estimated Time:** 20 minutes

---

#### Task 2.2: Implement error to HTTP status mapping
- [ ] Implement `IntoResponse` for `ApiError`
- [ ] Map `AssetNotFound` → 404
- [ ] Map `InvalidParameter` → 400
- [ ] Map `InvalidDateRange` → 400
- [ ] Map `SessionNotFound` → 404
- [ ] Map `SessionLimitReached` → 503
- [ ] Map `ComputationFailed` → 500
- [ ] Map `InternalError` → 500

**Estimated Time:** 20 minutes

---

#### Task 2.3: Implement error JSON serialization
- [ ] Define `ErrorResponse` struct
- [ ] Fields: `error: String`, `message: String`, `details: Option<Value>`
- [ ] Implement `From<ApiError>` for `ErrorResponse`
- [ ] Serialize to JSON in `IntoResponse` implementation
- [ ] Write tests for error formatting

**Estimated Time:** 25 minutes

---

#### Task 2.4: Convert from existing error types
- [ ] Implement `From<DagError>` for `ApiError`
- [ ] Implement `From<DataProviderError>` for `ApiError`
- [ ] Implement `From<serde_json::Error>` for `ApiError`
- [ ] Implement `From<chrono::ParseError>` for `ApiError`
- [ ] Test error conversions

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
- [ ] Create `list_assets` handler in `handlers.rs`
- [ ] Query all assets from `SqliteDataProvider`
- [ ] Map to response format with asset details
- [ ] Include `data_available_from` and `data_available_to` dates
- [ ] Return JSON array of asset objects
- [ ] Handle database errors
- [ ] Add route to router

**Estimated Time:** 35 minutes

---

#### Task 3.2: Implement GET /dag/nodes endpoint
- [ ] Create `list_analytics` handler
- [ ] Return hardcoded list of available analytics
- [ ] Include: "returns" with no parameters
- [ ] Include: "volatility" with window parameter (default 10)
- [ ] Add descriptions and burn-in information
- [ ] Format as JSON per spec
- [ ] Add route to router

**Estimated Time:** 25 minutes

---

#### Task 3.3: Test static endpoints
- [ ] Write integration test for `/assets`
- [ ] Write integration test for `/dag/nodes`
- [ ] Test error handling (database failure for /assets)
- [ ] Test response format matches spec
- [ ] Manual test with curl

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
- [ ] Create `DataQueryParams` struct with `start` and `end` dates
- [ ] Implement `FromRequest` for parsing query parameters
- [ ] Create `AssetDataResponse` struct
- [ ] Fields: `asset`, `start_date`, `end_date`, `data: Vec<DataPoint>`
- [ ] Define `DataPoint` struct with `timestamp` and `close`
- [ ] Implement serialization

**Estimated Time:** 25 minutes

---

#### Task 4.2: Implement GET /assets/{asset}/data handler
- [ ] Create `get_asset_data` handler
- [ ] Extract asset from path parameter
- [ ] Parse query parameters (start, end dates)
- [ ] Validate date range (start <= end)
- [ ] Create `DateRange` from parameters
- [ ] Query `SqliteDataProvider` for asset data
- [ ] Convert to response format
- [ ] Handle asset not found
- [ ] Handle invalid dates
- [ ] Add route to router

**Estimated Time:** 40 minutes

---

#### Task 4.3: Test asset data endpoint
- [ ] Test successful query with valid date range
- [ ] Test with different date ranges (1 day, 1 month, 1 year)
- [ ] Test 404 for non-existent asset
- [ ] Test 400 for invalid date format
- [ ] Test 400 for invalid date range (end < start)
- [ ] Test edge cases (no data in range)

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
- [ ] Create `build_analytics_dag()` helper function
- [ ] Parameters: asset, analytic_type, params map, returns (DAG, NodeId)
- [ ] Handle "returns" case: DataProvider → Returns
- [ ] Handle "volatility" case: DataProvider → Returns → Volatility
- [ ] Extract window parameter for volatility (default 10)
- [ ] Return DagError if unknown analytic type
- [ ] Add doc comments with examples

**Estimated Time:** 35 minutes

---

#### Task 5.2: Define analytics request/response types
- [ ] Create `AnalyticsQueryParams` struct
- [ ] Fields: `start`, `end` (required), `window` (optional)
- [ ] Create `AnalyticsResponse` struct
- [ ] Fields: `asset`, `analytic`, `parameters`, `start_date`, `end_date`, `data`
- [ ] Define `AnalyticDataPoint` with `timestamp` and `value` (nullable)
- [ ] Implement serialization (NaN → null in JSON)

**Estimated Time:** 25 minutes

---

#### Task 5.3: Implement GET /analytics/{asset}/{type} handler
- [ ] Create `get_analytics` handler
- [ ] Extract path parameters: asset, analytic type
- [ ] Parse query parameters
- [ ] Validate parameters (window > 0 if provided)
- [ ] Build DAG using helper function
- [ ] Create DateRange from start/end
- [ ] Call `dag.execute_pull_mode()`
- [ ] Convert TimeSeriesPoint to response format
- [ ] Handle NaN values (convert to null)
- [ ] Return JSON response
- [ ] Handle all error cases
- [ ] Add route to router

**Estimated Time:** 45 minutes

---

#### Task 5.4: Implement POST /analytics/batch handler
- [ ] Create `BatchQueryRequest` struct
- [ ] Field: `queries: Vec<BatchQuery>`
- [ ] Define `BatchQuery` with asset, analytic, parameters, dates
- [ ] Create `BatchQueryResponse` struct
- [ ] Fields: `results: Vec<AnalyticsResponse>`, `errors: Vec<BatchError>`
- [ ] Create `batch_analytics` handler
- [ ] Parse request body
- [ ] Collect node IDs for all queries
- [ ] Call `dag.execute_pull_mode_parallel()` if possible
- [ ] OR execute each query sequentially for now
- [ ] Collect successful results and errors
- [ ] Return partial success if some fail
- [ ] Add route to router

**Estimated Time:** 50 minutes

---

#### Task 5.5: Test analytics endpoints
- [ ] Test GET /analytics with returns
- [ ] Test GET /analytics with volatility (default window)
- [ ] Test GET /analytics with custom volatility window
- [ ] Test POST /analytics/batch with multiple assets
- [ ] Test 404 for unknown asset
- [ ] Test 400 for invalid parameters
- [ ] Test 400 for invalid analytic type
- [ ] Test response format (NaN → null)
- [ ] Test burn-in handled correctly

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
- [ ] Create `ReplaySession` struct in `state.rs`
- [ ] Fields: `id`, `assets`, `analytics`, `start_date`, `end_date`
- [ ] Fields: `status`, `dag`, `push_engine`, `replay_engine`
- [ ] Fields: `created_at`, `started_at`, `current_date`, `progress`
- [ ] Create `SessionStatus` enum: Created, Running, Completed, Stopped, Error
- [ ] Create `AnalyticConfig` struct for session analytics
- [ ] Implement session methods: `new()`, `start()`, `stop()`

**Estimated Time:** 35 minutes

---

#### Task 6.2: Define session request/response types
- [ ] Create `CreateSessionRequest` struct
- [ ] Fields: `assets`, `analytics`, `start_date`, `end_date`
- [ ] Create `SessionResponse` struct
- [ ] Fields: `session_id`, `status`, `assets`, `analytics`, dates, `stream_url`
- [ ] Create `SessionStatusResponse` with additional fields
- [ ] Fields: add `current_date`, `progress`, `created_at`, `started_at`
- [ ] Implement serialization

**Estimated Time:** 25 minutes

---

#### Task 6.3: Implement POST /replay handler
- [ ] Create `create_replay_session` handler
- [ ] Parse request body
- [ ] Validate assets exist in database
- [ ] Validate date range
- [ ] Check session limit (max 10)
- [ ] Generate UUID for session ID
- [ ] Build DAG for each asset+analytic combination
- [ ] Create PushModeEngine instance
- [ ] Initialize push engine with historical data
- [ ] Create ReplayEngine instance
- [ ] Set replay delay (100ms default)
- [ ] Store session in app state
- [ ] Return session response with stream URL
- [ ] Add route to router

**Estimated Time:** 60 minutes

---

#### Task 6.4: Implement GET /replay/{session_id} handler
- [ ] Create `get_session_status` handler
- [ ] Extract session ID from path
- [ ] Look up session in app state
- [ ] Return 404 if not found
- [ ] Build status response with all fields
- [ ] Include progress calculation
- [ ] Add route to router

**Estimated Time:** 25 minutes

---

#### Task 6.5: Implement DELETE /replay/{session_id} handler
- [ ] Create `stop_replay_session` handler
- [ ] Extract session ID from path
- [ ] Look up session in app state
- [ ] Return 404 if not found
- [ ] Check if already completed/stopped (409 Conflict)
- [ ] Call stop on replay engine
- [ ] Update session status to Stopped
- [ ] Close SSE streams for this session
- [ ] Return success response
- [ ] Add route to router

**Estimated Time:** 30 minutes

---

#### Task 6.6: Test session management
- [ ] Test session creation with valid parameters
- [ ] Test session status query
- [ ] Test session deletion
- [ ] Test 404 for unknown session
- [ ] Test session limit enforcement (503)
- [ ] Test 400 for invalid parameters
- [ ] Test 404 for non-existent assets

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
- [ ] Add `axum::response::sse` imports
- [ ] Create SSE event types in separate module
- [ ] Define `SseEvent` enum: Update, Progress, Complete, Stopped, Error
- [ ] Implement serialization for each event type
- [ ] Create channel for broadcasting (mpsc::channel)
- [ ] Add broadcaster map to AppState

**Estimated Time:** 30 minutes

---

#### Task 7.2: Implement GET /stream/{session_id} handler
- [ ] Create `handle_stream` handler
- [ ] Extract session ID from path
- [ ] Look up session, return 404 if not found
- [ ] Create mpsc channel for this client
- [ ] Register broadcaster in app state
- [ ] Convert receiver to SSE stream
- [ ] Set up keep-alive (15 second interval)
- [ ] Return SSE response
- [ ] Add route to router

**Estimated Time:** 35 minutes

---

#### Task 7.3: Implement broadcast helper functions
- [ ] Create `broadcast_update()` function
- [ ] Parameters: session_id, asset, analytic, TimeSeriesPoint
- [ ] Look up broadcaster for session
- [ ] Create Update event with data
- [ ] Send event to all connected clients
- [ ] Handle send errors gracefully
- [ ] Create `broadcast_progress()` function
- [ ] Create `broadcast_complete()` function
- [ ] Create `broadcast_error()` function

**Estimated Time:** 30 minutes

---

#### Task 7.4: Wire SSE to push-mode callbacks
- [ ] Modify session creation to register callbacks
- [ ] For each node in DAG, register callback
- [ ] Callback extracts node info (asset, analytic)
- [ ] Callback calls broadcast_update()
- [ ] Add progress callback to replay engine
- [ ] Progress callback calls broadcast_progress()
- [ ] Add completion callback to replay engine
- [ ] Completion callback calls broadcast_complete()

**Estimated Time:** 45 minutes

---

#### Task 7.5: Handle client disconnect
- [ ] Detect when SSE client disconnects
- [ ] Remove broadcaster from app state
- [ ] Close channel
- [ ] Log disconnect event
- [ ] Don't stop session (other clients may be connected)

**Estimated Time:** 20 minutes

---

#### Task 7.6: Test SSE streaming
- [ ] Create test helper for connecting to SSE stream
- [ ] Test connection established successfully
- [ ] Test update events received
- [ ] Test progress events received
- [ ] Test complete event received
- [ ] Test multiple clients on same session
- [ ] Test client disconnect

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
- [ ] Create `run_replay_session()` async function
- [ ] Parameters: session_id, app_state
- [ ] Look up session from state
- [ ] Extract replay engine and push engine
- [ ] Set up data callback for replay engine
- [ ] Data callback calls `push_engine.push_data()`
- [ ] Set up progress callback
- [ ] Progress callback updates session.current_date and progress
- [ ] Progress callback broadcasts progress event every N points
- [ ] Set up error callback
- [ ] Error callback broadcasts error event

**Estimated Time:** 45 minutes

---

#### Task 8.2: Start replay in background
- [ ] Modify session creation to spawn background task
- [ ] Use `tokio::spawn()` to run replay asynchronously
- [ ] Pass session ID and cloned app state
- [ ] Update session status to Running
- [ ] Call `replay_engine.run()` in task
- [ ] Handle replay completion
- [ ] Update session status to Completed
- [ ] Broadcast complete event
- [ ] Handle replay errors
- [ ] Update status to Error on failure

**Estimated Time:** 35 minutes

---

#### Task 8.3: Implement session cleanup task
- [ ] Create `cleanup_sessions()` background task
- [ ] Run every 1 hour
- [ ] Iterate through sessions
- [ ] Remove Completed sessions older than 1 hour
- [ ] Remove Stopped sessions older than 1 hour
- [ ] Remove Error sessions older than 15 minutes
- [ ] Close associated broadcasters
- [ ] Log cleanup actions
- [ ] Start cleanup task when server starts

**Estimated Time:** 30 minutes

---

#### Task 8.4: Test replay integration
- [ ] Test session starts replay automatically
- [ ] Test SSE events received during replay
- [ ] Test session status updates correctly
- [ ] Test session marked complete after replay
- [ ] Test cleanup removes old sessions
- [ ] Integration test: full replay lifecycle

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
- [ ] Add `config` crate dependency
- [ ] Create default ServerConfig values
- [ ] Load config from `server.toml` if exists
- [ ] Load config from environment variables
- [ ] Precedence: env vars > config file > defaults
- [ ] Document configuration options

**Estimated Time:** 35 minutes

---

#### Task 9.2: Create server binary
- [ ] Create `src/bin/analytics-server.rs`
- [ ] Parse command-line arguments (--host, --port, --db)
- [ ] Load ServerConfig
- [ ] Initialize SqliteDataProvider
- [ ] Create AppState
- [ ] Call run_server()
- [ ] Handle shutdown signals (Ctrl+C)
- [ ] Log startup information
- [ ] Test: `cargo run --bin analytics-server`

**Estimated Time:** 40 minutes

---

#### Task 9.3: Add logging configuration
- [ ] Set up tracing subscriber
- [ ] Log level from config or env (default: INFO)
- [ ] Format logs for readability
- [ ] Log all HTTP requests
- [ ] Log session lifecycle events
- [ ] Log errors with full context

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
- [ ] Create `docs/API.md` with endpoint descriptions
- [ ] Include all 9 endpoints
- [ ] Show example requests and responses
- [ ] Document error codes
- [ ] Document SSE event format
- [ ] Add authentication note (none for POC)

**Estimated Time:** 45 minutes

---

#### Task 10.2: Create example curl commands
- [ ] Create `examples/api_examples.sh`
- [ ] Add curl command for each endpoint
- [ ] GET /assets
- [ ] GET /assets/AAPL/data
- [ ] GET /analytics/AAPL/returns
- [ ] GET /analytics/AAPL/volatility with window
- [ ] POST /analytics/batch
- [ ] POST /replay
- [ ] GET /replay/{id}
- [ ] DELETE /replay/{id}
- [ ] Add comments explaining each command

**Estimated Time:** 30 minutes

---

#### Task 10.3: Create SSE client example
- [ ] Create `examples/sse_client.html`
- [ ] JavaScript EventSource example
- [ ] Connect to /stream endpoint
- [ ] Handle update events
- [ ] Handle progress events
- [ ] Handle complete events
- [ ] Display events in browser console
- [ ] Add simple HTML UI

**Estimated Time:** 40 minutes

---

#### Task 10.4: Update main README
- [ ] Add server section to README.md
- [ ] Explain how to start server
- [ ] Link to API documentation
- [ ] Describe configuration options
- [ ] Show basic usage example
- [ ] Add troubleshooting section

**Estimated Time:** 30 minutes

---

#### Task 10.5: Create quick start guide
- [ ] Create `docs/QUICKSTART.md`
- [ ] Step 1: Download sample data
- [ ] Step 2: Start server
- [ ] Step 3: Query assets
- [ ] Step 4: Run pull-mode analytics
- [ ] Step 5: Create replay session
- [ ] Step 6: Stream updates
- [ ] Include expected output for each step

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

