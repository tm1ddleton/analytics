# Specification: REST API Server with Server-Sent Events

**Date:** 2025-11-25  
**Status:** Approved  
**Roadmap Item:** 9  
**Size:** M (Medium)

---

## Goal

Build an HTTP server with REST API endpoints for querying asset data and analytics, plus Server-Sent Events (SSE) for streaming real-time analytics updates. The server exposes the analytics engine capabilities to external clients, particularly the React UI dashboard (Item 10).

The API server should:
- Provide REST endpoints for asset queries and pull-mode analytics
- Stream incremental updates via Server-Sent Events during replay
- Manage replay sessions with create/monitor/stop controls
- Support 5-10 concurrent clients for small team usage
- Integrate seamlessly with existing DAG, pull-mode, push-mode, and replay systems
- Use standard HTTP error semantics with JSON responses

---

## User Stories

### Story 1: Query Historical Analytics
**As a** React UI developer  
**I want to** query historical volatility for AAPL via REST API  
**So that I can** display baseline analytics before starting real-time replay

**Acceptance Criteria:**
- GET request returns complete time series for date range
- Can specify custom parameters (e.g., volatility window size)
- Returns 404 if asset doesn't exist
- Response is JSON array of timestamped values
- Query completes in < 1 second for 1 year of data

### Story 2: Live Replay Updates
**As a** React UI developer  
**I want to** receive real-time analytics updates during replay  
**So that I can** display live charts updating as replay progresses

**Acceptance Criteria:**
- Create replay session via POST request
- Connect to SSE stream for incremental updates
- Receive one event per new data point computed
- Events include asset, analytic type, timestamp, and value
- Can stop session early via DELETE request

### Story 3: Multi-Asset Batch Query
**As a** portfolio analyst  
**I want to** query returns for multiple assets in one request  
**So that I can** efficiently load portfolio analytics

**Acceptance Criteria:**
- POST request with array of assets
- Returns results grouped by asset
- Parallel execution for performance
- Single request for 10 assets completes in < 3 seconds

### Story 4: Discover Available Analytics
**As a** React UI developer  
**I want to** query which analytics are available  
**So that I can** dynamically build UI controls

**Acceptance Criteria:**
- GET endpoint lists analytic types
- Includes parameter descriptions (e.g., "window" for volatility)
- Returns supported asset types
- Human-readable descriptions

---

## Specific Requirements

### 1. Technology Stack

**Framework:** `axum`
- Modern async Rust web framework
- Type-safe extractors and middleware
- Excellent SSE support via `axum::response::sse`
- Built on tokio (already in use)

**Dependencies:**
- `axum` - Web framework
- `tokio` - Async runtime
- `tower-http` - CORS, logging middleware
- `serde_json` - JSON serialization
- `uuid` - Session ID generation

### 2. REST API Endpoints

#### 2.1: GET /assets
**Purpose:** List all available assets in the database

**Response:**
```json
{
  "assets": [
    {
      "key": "AAPL",
      "type": "equity",
      "name": "Apple Inc.",
      "data_available_from": "2020-01-01",
      "data_available_to": "2024-12-31"
    },
    {
      "key": "MSFT",
      "type": "equity",
      "name": "Microsoft Corporation",
      "data_available_from": "2020-01-01",
      "data_available_to": "2024-12-31"
    }
  ]
}
```

**Status Codes:**
- `200 OK` - Success
- `500 Internal Server Error` - Database query failed

---

#### 2.2: GET /assets/{asset}/data
**Purpose:** Get raw historical price data for an asset

**Query Parameters:**
- `start` (required) - Start date (YYYY-MM-DD)
- `end` (required) - End date (YYYY-MM-DD)

**Example Request:**
```
GET /assets/AAPL/data?start=2024-01-01&end=2024-12-31
```

**Response:**
```json
{
  "asset": "AAPL",
  "start_date": "2024-01-01",
  "end_date": "2024-12-31",
  "data": [
    {
      "timestamp": "2024-01-01T00:00:00Z",
      "close": 185.64
    },
    {
      "timestamp": "2024-01-02T00:00:00Z",
      "close": 187.21
    }
  ]
}
```

**Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Invalid date format
- `404 Not Found` - Asset not found
- `500 Internal Server Error` - Database error

---

#### 2.3: GET /analytics/{asset}/{type}
**Purpose:** Execute pull-mode analytics query for an asset

**Path Parameters:**
- `asset` - Asset identifier (e.g., "AAPL")
- `type` - Analytic type: "returns" or "volatility"

**Query Parameters:**
- `start` (required) - Start date (YYYY-MM-DD)
- `end` (required) - End date (YYYY-MM-DD)
- `window` (optional, for volatility) - Window size in days (default: 10)

**Example Requests:**
```
GET /analytics/AAPL/returns?start=2024-01-01&end=2024-12-31
GET /analytics/AAPL/volatility?window=20&start=2024-01-01&end=2024-12-31
```

**Response:**
```json
{
  "asset": "AAPL",
  "analytic": "volatility",
  "parameters": {
    "window": 20
  },
  "start_date": "2024-01-01",
  "end_date": "2024-12-31",
  "data": [
    {
      "timestamp": "2024-01-01T00:00:00Z",
      "value": null
    },
    {
      "timestamp": "2024-01-02T00:00:00Z",
      "value": 0.0234
    }
  ]
}
```

**Implementation:**
- Server builds DAG on-the-fly based on analytic type
- Uses `execute_pull_mode()` from DAG module
- Automatic burn-in calculation handled by DAG
- Returns `null` for NaN values in JSON

**Status Codes:**
- `200 OK` - Success
- `400 Bad Request` - Invalid parameters or dates
- `404 Not Found` - Asset not found
- `500 Internal Server Error` - Computation failed

---

#### 2.4: POST /analytics/batch
**Purpose:** Query analytics for multiple assets in parallel

**Request Body:**
```json
{
  "queries": [
    {
      "asset": "AAPL",
      "analytic": "returns",
      "start_date": "2024-01-01",
      "end_date": "2024-12-31"
    },
    {
      "asset": "MSFT",
      "analytic": "volatility",
      "parameters": {
        "window": 10
      },
      "start_date": "2024-01-01",
      "end_date": "2024-12-31"
    }
  ]
}
```

**Response:**
```json
{
  "results": [
    {
      "asset": "AAPL",
      "analytic": "returns",
      "data": [...]
    },
    {
      "asset": "MSFT",
      "analytic": "volatility",
      "data": [...]
    }
  ],
  "errors": []
}
```

**Implementation:**
- Uses `execute_pull_mode_parallel()` from DAG module
- Returns partial results if some queries fail
- Errors array contains failed queries with error messages

**Status Codes:**
- `200 OK` - All queries succeeded or partial success
- `400 Bad Request` - Invalid request format
- `500 Internal Server Error` - All queries failed

---

#### 2.5: GET /dag/nodes
**Purpose:** List available analytic types and their parameters

**Response:**
```json
{
  "analytics": [
    {
      "type": "returns",
      "description": "Log returns calculation",
      "parameters": [],
      "burnin_days": 1
    },
    {
      "type": "volatility",
      "description": "Rolling volatility (population std dev)",
      "parameters": [
        {
          "name": "window",
          "type": "integer",
          "required": false,
          "default": 10,
          "description": "Rolling window size in days"
        }
      ],
      "burnin_days": "window + 1"
    }
  ]
}
```

**Status Codes:**
- `200 OK` - Success

---

#### 2.6: POST /replay
**Purpose:** Create a new replay session

**Request Body:**
```json
{
  "assets": ["AAPL", "MSFT"],
  "analytics": [
    {
      "type": "returns"
    },
    {
      "type": "volatility",
      "parameters": {
        "window": 10
      }
    }
  ],
  "start_date": "2024-01-01",
  "end_date": "2024-12-31"
}
```

**Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "created",
  "assets": ["AAPL", "MSFT"],
  "analytics": ["returns", "volatility"],
  "start_date": "2024-01-01",
  "end_date": "2024-12-31",
  "stream_url": "/stream/550e8400-e29b-41d4-a716-446655440000"
}
```

**Implementation:**
- Generate unique session ID (UUID)
- Create DAG for each asset+analytic combination
- Initialize push-mode engine with historical data
- Create replay engine instance
- Store session in server-side map
- Session starts automatically upon creation

**Status Codes:**
- `201 Created` - Session created successfully
- `400 Bad Request` - Invalid parameters
- `404 Not Found` - One or more assets not found
- `500 Internal Server Error` - Failed to create session

---

#### 2.7: GET /replay/{session_id}
**Purpose:** Get replay session details and status

**Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "running",
  "assets": ["AAPL", "MSFT"],
  "analytics": ["returns", "volatility"],
  "start_date": "2024-01-01",
  "end_date": "2024-12-31",
  "current_date": "2024-06-15",
  "progress": 0.45,
  "created_at": "2024-12-01T10:30:00Z",
  "started_at": "2024-12-01T10:30:01Z",
  "stream_url": "/stream/550e8400-e29b-41d4-a716-446655440000"
}
```

**Status Values:**
- `created` - Session created but not started
- `running` - Replay in progress
- `completed` - Replay finished
- `stopped` - Manually stopped
- `error` - Failed with error

**Status Codes:**
- `200 OK` - Success
- `404 Not Found` - Session not found

---

#### 2.8: DELETE /replay/{session_id}
**Purpose:** Stop a running replay session

**Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "stopped",
  "message": "Replay session stopped"
}
```

**Implementation:**
- Stop replay engine for this session
- Close all SSE streams for this session
- Mark session as stopped
- Keep session data for status queries

**Status Codes:**
- `200 OK` - Successfully stopped
- `404 Not Found` - Session not found
- `409 Conflict` - Session already completed or stopped

---

#### 2.9: GET /stream/{session_id}
**Purpose:** Server-Sent Events stream for replay updates

**Response:** SSE stream with events

**Event Format:**
```
event: update
data: {"asset":"AAPL","analytic":"returns","timestamp":"2024-01-02T00:00:00Z","value":0.0198}

event: update
data: {"asset":"AAPL","analytic":"volatility","timestamp":"2024-01-02T00:00:00Z","value":0.0123}

event: update
data: {"asset":"MSFT","analytic":"returns","timestamp":"2024-01-02T00:00:00Z","value":0.0156}

event: progress
data: {"current_date":"2024-01-02","progress":0.01}

event: complete
data: {"session_id":"550e8400-e29b-41d4-a716-446655440000","message":"Replay completed"}
```

**Event Types:**
- `update` - New analytics data point computed
- `progress` - Progress update (every N points)
- `error` - Error during replay
- `complete` - Replay finished
- `stopped` - Replay stopped by client

**Implementation:**
- Register callbacks with push-mode engine nodes
- On each node callback, serialize data point and send as SSE event
- Use `axum::response::sse::Event`
- Keep-alive comments every 15 seconds
- Handle client disconnect gracefully

**Status Codes:**
- `200 OK` - Stream established
- `404 Not Found` - Session not found

---

### 3. Error Response Format

All error responses follow this structure:

```json
{
  "error": "ErrorType",
  "message": "Human-readable error description",
  "details": {
    "field": "additional context"
  }
}
```

**Error Types:**
- `AssetNotFound` - Requested asset doesn't exist
- `InvalidParameter` - Query parameter invalid
- `InvalidDateRange` - Date range invalid
- `ComputationFailed` - Analytics computation error
- `SessionNotFound` - Replay session doesn't exist
- `InternalError` - Server error

**HTTP Status Code Mapping:**
- `400` - Client errors (invalid parameters, bad request)
- `404` - Resource not found (asset, session)
- `409` - Conflict (session already stopped)
- `500` - Server errors (computation failed, database error)

---

### 4. Server Architecture

#### 4.1: Application State

Shared application state (wrapped in `Arc<AppState>`):

```rust
struct AppState {
    /// SQLite data provider
    data_provider: Arc<SqliteDataProvider>,
    /// Active replay sessions
    sessions: Arc<RwLock<HashMap<Uuid, ReplaySession>>>,
    /// SSE broadcaster for each session
    broadcasters: Arc<RwLock<HashMap<Uuid, Sender<Event>>>>,
}

struct ReplaySession {
    id: Uuid,
    assets: Vec<AssetKey>,
    analytics: Vec<AnalyticConfig>,
    start_date: NaiveDate,
    end_date: NaiveDate,
    status: SessionStatus,
    dag: AnalyticsDag,
    push_engine: PushModeEngine,
    replay_engine: ReplayEngine,
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    current_date: Option<NaiveDate>,
}
```

#### 4.2: Server Configuration

```rust
struct ServerConfig {
    /// Bind address (default: 127.0.0.1)
    host: String,
    /// Port (default: 3000)
    port: u16,
    /// Database path
    database_path: String,
    /// Replay delay between points (default: 100ms)
    replay_delay_ms: u64,
    /// Max concurrent sessions (default: 10)
    max_sessions: usize,
    /// Session cleanup interval (default: 1 hour)
    session_cleanup_interval: Duration,
}
```

#### 4.3: Middleware

- **CORS:** Allow all origins for POC (restrict in production)
- **Logging:** Request/response logging with `tower_http::trace`
- **Error handling:** Convert internal errors to JSON responses
- **Request ID:** Add unique ID to each request for tracing

---

### 5. Server-Sent Events Implementation

#### 5.1: SSE Stream Setup

```rust
async fn handle_stream(
    Path(session_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    
    // Register broadcaster for this session
    state.broadcasters.write().await.insert(session_id, tx);
    
    // Create SSE stream from receiver
    let stream = ReceiverStream::new(rx);
    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

#### 5.2: Broadcasting Updates

```rust
fn broadcast_update(
    session_id: Uuid,
    asset: &AssetKey,
    analytic: &str,
    point: &TimeSeriesPoint,
    broadcasters: &HashMap<Uuid, Sender<Event>>,
) {
    if let Some(tx) = broadcasters.get(&session_id) {
        let data = json!({
            "asset": asset.to_string(),
            "analytic": analytic,
            "timestamp": point.timestamp,
            "value": point.close_price,
        });
        
        let event = Event::default()
            .event("update")
            .data(data.to_string());
            
        let _ = tx.try_send(event);
    }
}
```

#### 5.3: Client Reconnection

- SSE automatically handles reconnection with `Last-Event-ID`
- Server maintains event history for last N events per session
- On reconnect, replay missed events from history
- History size: 100 events (configurable)

---

### 6. Replay Session Management

#### 6.1: Session Lifecycle

1. **Create:** `POST /replay` creates session, builds DAG, initializes engines
2. **Start:** Session starts automatically after creation
3. **Stream:** Clients connect to SSE stream
4. **Run:** Replay engine feeds data to push-mode engine
5. **Update:** Push-mode callbacks trigger SSE events
6. **Complete:** Replay finishes, send completion event
7. **Cleanup:** Session kept for status queries, cleaned up after timeout

#### 6.2: Session Cleanup

Background task runs every hour:
- Remove completed sessions older than 1 hour
- Remove stopped sessions older than 1 hour
- Remove error sessions older than 15 minutes
- Close associated SSE streams
- Log cleanup actions

#### 6.3: Concurrent Session Limits

- Max 10 concurrent sessions (configurable)
- Return 503 Service Unavailable if limit reached
- Clients should poll `/replay/{id}` status before creating new session

---

### 7. Integration with Existing Systems

#### 7.1: DAG Integration

Server builds DAGs dynamically:

```rust
fn build_analytics_dag(
    asset: &AssetKey,
    analytic: &str,
    params: &HashMap<String, String>,
) -> Result<(AnalyticsDag, NodeId), ApiError> {
    let mut dag = AnalyticsDag::new();
    
    let data_node = dag.add_node(
        "DataProvider".to_string(),
        NodeParams::None,
        vec![asset.clone()],
    );
    
    match analytic {
        "returns" => {
            let returns_node = dag.add_node(
                "Returns".to_string(),
                NodeParams::None,
                vec![asset.clone()],
            );
            dag.add_edge(data_node, returns_node)?;
            Ok((dag, returns_node))
        }
        "volatility" => {
            let window = params.get("window")
                .and_then(|s| s.parse().ok())
                .unwrap_or(10);
                
            // Build: DataProvider -> Returns -> Volatility
            let returns_node = dag.add_node(...);
            let vol_node = dag.add_node(...);
            dag.add_edge(data_node, returns_node)?;
            dag.add_edge(returns_node, vol_node)?;
            Ok((dag, vol_node))
        }
        _ => Err(ApiError::InvalidParameter("Unknown analytic type")),
    }
}
```

#### 7.2: Pull-Mode Integration

```rust
async fn execute_pull_query(
    asset: &AssetKey,
    analytic: &str,
    params: &HashMap<String, String>,
    date_range: DateRange,
    provider: &SqliteDataProvider,
) -> Result<Vec<TimeSeriesPoint>, ApiError> {
    let (dag, target_node) = build_analytics_dag(asset, analytic, params)?;
    
    let result = dag.execute_pull_mode(
        target_node,
        date_range,
        provider,
    )?;
    
    Ok(result)
}
```

#### 7.3: Push-Mode Integration

For replay sessions:

```rust
async fn create_replay_session(
    config: ReplaySessionConfig,
    state: &AppState,
) -> Result<Uuid, ApiError> {
    let session_id = Uuid::new_v4();
    
    // Build DAG
    let (dag, node_map) = build_session_dag(&config)?;
    
    // Initialize push-mode engine
    let mut push_engine = PushModeEngine::new(dag);
    push_engine.initialize(&*state.data_provider, &config.date_range)?;
    
    // Register callbacks for SSE broadcasting
    for (asset, analytic, node_id) in node_map {
        let session_id_clone = session_id;
        let asset_clone = asset.clone();
        let analytic_clone = analytic.clone();
        let broadcasters_clone = state.broadcasters.clone();
        
        push_engine.register_callback(node_id, move |point| {
            broadcast_update(
                session_id_clone,
                &asset_clone,
                &analytic_clone,
                point,
                &broadcasters_clone.read().unwrap(),
            );
        });
    }
    
    // Create replay engine
    let replay_engine = ReplayEngine::new(state.data_provider.clone());
    replay_engine.set_delay(Duration::from_millis(100));
    
    // Store session
    let session = ReplaySession {
        id: session_id,
        push_engine,
        replay_engine,
        // ... other fields
    };
    
    state.sessions.write().await.insert(session_id, session);
    
    // Start replay in background task
    tokio::spawn(run_replay_session(session_id, state.clone()));
    
    Ok(session_id)
}
```

---

### 8. Performance Requirements

#### 8.1: Response Times

- **GET /assets:** < 100ms
- **GET /assets/{asset}/data:** < 500ms for 1 year of data
- **GET /analytics/{asset}/{type}:** < 1s for 1 year, < 5s for 5 years
- **POST /analytics/batch (10 assets):** < 3s for 1 year per asset
- **POST /replay:** < 500ms (session creation)
- **SSE event latency:** < 50ms from computation to client

#### 8.2: Throughput

- Support 5-10 concurrent clients
- Each client can have 1-2 active SSE streams
- Server should handle 100 requests/second
- Replay can stream 10-20 events/second per session

#### 8.3: Memory

- Keep session data in memory for active sessions
- Limit to 10 concurrent sessions
- Each session ~10-50MB depending on assets/analytics
- Total server memory target: < 1GB for POC

---

### 9. Testing Requirements

#### 9.1: Unit Tests

- Test each handler function in isolation
- Mock application state
- Test error paths (404, 400, 500)
- Test JSON serialization/deserialization

#### 9.2: Integration Tests

- Test full request/response cycle
- Test with real SQLite database
- Test SSE streaming
- Test session lifecycle

#### 9.3: Manual Testing

- Test with `curl` for REST endpoints
- Test with browser EventSource for SSE
- Test with React UI (Item 10)

---

### 10. Future Enhancements (Post-POC)

- **Authentication:** Add API key or JWT authentication
- **Rate limiting:** Prevent abuse
- **Caching:** Cache pull-mode results
- **Compression:** Gzip response bodies
- **WebSocket support:** For bidirectional communication
- **Configurable replay speed:** Client-specified delay
- **Pause/resume replay:** Advanced session controls
- **GraphQL API:** Alternative to REST for complex queries
- **Metrics:** Prometheus metrics for monitoring
- **Health checks:** `/health` and `/ready` endpoints

---

## Implementation Order

1. **Server Setup** - Basic axum server with routes
2. **Static Endpoints** - `/assets`, `/dag/nodes`
3. **Data Query** - `/assets/{asset}/data`
4. **Pull-Mode Analytics** - `/analytics/{asset}/{type}`
5. **Batch Query** - `/analytics/batch`
6. **Session Management** - `POST /replay`, `GET /replay/{id}`, `DELETE /replay/{id}`
7. **SSE Streaming** - `/stream/{session_id}`
8. **Integration** - Wire up DAG, push-mode, replay
9. **Error Handling** - Comprehensive error responses
10. **Testing** - Unit and integration tests

---

## Success Metrics

- ✅ All 9 REST endpoints implemented and tested
- ✅ SSE streaming works with incremental updates
- ✅ Replay sessions can be created, monitored, and stopped
- ✅ 5 concurrent clients can query and stream without issues
- ✅ Response times meet performance requirements
- ✅ Ready for React UI integration (Item 10)

