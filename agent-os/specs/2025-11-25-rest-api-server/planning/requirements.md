# Requirements Gathering: REST API Server with WebSocket/SSE

## Clarifying Questions & Answers

### 1. Server Framework
**Q:** Which Rust web framework would you prefer?  
**A:** D - Your preference or let me choose

**Decision Rationale:** Will choose `axum` for its modern async design, type-safe extractors, excellent integration with tokio ecosystem (which we're already using), and strong Server-Sent Events support.

### 2. Real-Time Communication
**Q:** For real-time push updates, which approach?  
**A:** B - Server-Sent Events (SSE)

**Decision Rationale:** SSE is perfect for unidirectional server→client streaming of analytics updates. Simpler than WebSocket, better browser support, automatic reconnection, and sufficient for our use case.

### 3. API Scope
**Q:** What REST endpoints do we need for the POC?  
**A:** B - Read + Execute (Query data and trigger analytics)

**Decision Rationale:** Focus on consumption and execution. No mutation endpoints (CRUD) for now. Clients can query assets, execute analytics, and subscribe to updates.

### 4. DAG Management
**Q:** How should clients interact with the DAG?  
**A:** A - Server-side DAG, API manages DAG, clients just request analytics by name

**Decision Rationale:** Simplified client interface. Server maintains predefined analytics (e.g., "returns", "volatility-10", "volatility-20"). Clients request by name rather than building DAGs.

### 5. Data Formats
**Q:** What response format for time-series data?  
**A:** A - JSON

**Decision Rationale:** Standard, easy to consume in React UI, human-readable for debugging, good tooling support.

### 6. Authentication
**Q:** Security for the POC phase?  
**A:** A - None - Open API for development

**Decision Rationale:** No auth for POC simplicity. Can add later.

### 7. Push-Mode Subscription
**Q:** How should clients subscribe to real-time updates?  
**A:** B - Subscribe to asset+analytic combinations (e.g., "AAPL volatility")

**Decision Rationale:** Intuitive subscription model. Client specifies asset and analytic type, receives updates for that combination.

### 8. Error Handling
**Q:** API error response format?  
**A:** A - Standard HTTP status codes + JSON error objects

**Decision Rationale:** Standard REST practices. HTTP status for category (400/404/500), JSON body with error details.

### 9. Replay Integration
**Q:** How should API expose replay functionality?  
**A:** A - Session-based - Create replay session, clients auto-receive updates

**Decision Rationale:** Clients create a replay session with parameters (assets, date range, speed). Server streams updates via SSE. Clean lifecycle management.

### 10. Concurrent Clients
**Q:** How many concurrent clients should we support?  
**A:** B - Small team (5-10 concurrent clients)

**Decision Rationale:** POC supports small team usage. Not production-scale but more than single-user.

## Follow-up Questions

### Follow-up 1: Predefined Analytics
Since we're using server-side DAG management (option A for Q4), which predefined analytics should the API expose?
- **A)** Just the basics: "returns", "volatility" (with configurable window)
- **B)** Extended set: returns, multiple volatility windows (5/10/20), correlation
- **C)** Dynamic: Client can specify analytic type + parameters (e.g., "volatility" + window=15)
- **D)** Your specific requirements

### Follow-up 2: SSE Stream Format
For SSE updates, what should each event contain?
- **A)** Full state: Complete time series up to current point
- **B)** Incremental: Just the new data point that was computed
- **C)** Both: Incremental updates + periodic full snapshots
- **D)** Your preference

### Follow-up 3: Replay Session Control
What controls should clients have over replay sessions?
- **A)** Basic: start, stop, get status
- **B)** Extended: start, pause, resume, stop, adjust speed, get status
- **C)** Minimal: start only (runs to completion), get status
- **D)** Your requirements

### Follow-up 4: REST Endpoints
Which specific endpoints do we need?
- **A)** Minimal set:
  - `GET /assets` - List assets
  - `GET /analytics/{asset}/{type}` - Get analytics (pull-mode)
  - `POST /replay` - Create replay session
  - `GET /stream/{session_id}` - SSE stream for session
  
- **B)** Extended set (add):
  - `GET /assets/{asset}/data` - Raw price data
  - `GET /replay/{session_id}` - Session details
  - `DELETE /replay/{session_id}` - Stop session
  
- **C)** Comprehensive (also add):
  - `GET /dag/nodes` - List available analytics
  - `POST /analytics/batch` - Batch query multiple assets
  
- **D)** Your specific endpoint list

### Follow-up 5: Error Scenarios
What should happen if a client requests analytics for an asset with no data?
- **A)** Return 404 with error message
- **B)** Return 200 with empty array
- **C)** Return 200 with partial data (NaN for missing points)
- **D)** Your preference

### Follow-up 6: Replay Speed
How should clients specify replay speed?
- **A)** Multiplier (e.g., 10x, 100x real-time)
- **B)** Delay in milliseconds (e.g., 100ms between points)
- **C)** Both options supported
- **D)** Server decides, not configurable

Please answer these follow-up questions to finalize the requirements.

## Follow-up Answers

### Follow-up 1: Predefined Analytics
**Q:** Which predefined analytics should the API expose?  
**A:** C - Dynamic: Client can specify analytic type + parameters (e.g., "volatility" + window=15)

**Decision Rationale:** Maximum flexibility. Clients can request any analytic type with custom parameters. API validates parameters and builds appropriate DAG on-the-fly.

**Example Requests:**
- `GET /analytics/AAPL/returns?start=2024-01-01&end=2024-12-31`
- `GET /analytics/AAPL/volatility?window=15&start=2024-01-01&end=2024-12-31`
- `GET /analytics/MSFT/volatility?window=20&start=2024-06-01&end=2024-12-31`

### Follow-up 2: SSE Stream Format
**Q:** For SSE updates, what should each event contain?  
**A:** B - Incremental: Just the new data point that was computed

**Decision Rationale:** Efficient streaming. Each SSE event contains only the newly computed data point (timestamp + value). Clients maintain state on their end. Reduces bandwidth and improves performance.

**Event Format:**
```json
{
  "asset": "AAPL",
  "analytic": "volatility",
  "timestamp": "2024-01-15T00:00:00Z",
  "value": 0.0234
}
```

### Follow-up 3: Replay Session Control
**Q:** What controls should clients have over replay sessions?  
**A:** A - Basic: start, stop, get status

**Decision Rationale:** Simple control flow for POC. Clients can start a session, stop it early, and check status. No pause/resume complexity.

### Follow-up 4: REST Endpoints
**Q:** Which specific endpoints do we need?  
**A:** C - Comprehensive set

**Endpoint List:**
- `GET /assets` - List all available assets
- `GET /assets/{asset}/data?start={date}&end={date}` - Get raw price data
- `GET /analytics/{asset}/{type}?window={n}&start={date}&end={date}` - Pull-mode analytics query
- `POST /analytics/batch` - Batch query multiple assets
- `GET /dag/nodes` - List available analytic types
- `POST /replay` - Create replay session
- `GET /replay/{session_id}` - Get session details/status
- `DELETE /replay/{session_id}` - Stop/cancel session
- `GET /stream/{session_id}` - SSE stream for replay session

### Follow-up 5: Error Scenarios
**Q:** What should happen if a client requests analytics for an asset with no data?  
**A:** A - Return 404 with error message

**Decision Rationale:** Clear error semantics. 404 indicates resource not found. Client can distinguish between "asset doesn't exist" vs "computation failed" (500) vs "bad request" (400).

**Error Response Format:**
```json
{
  "error": "AssetNotFound",
  "message": "Asset 'AAPL' not found in database",
  "asset": "AAPL"
}
```

### Follow-up 6: Replay Speed
**Q:** How should clients specify replay speed?  
**A:** D - Server decides, not configurable

**Decision Rationale:** Server uses reasonable default delay (e.g., 100ms between points) for POC. Simplifies API and prevents clients from overwhelming the system. Can add configurability post-POC if needed.

## Summary

The REST API Server will provide:

1. **Pull-Mode Analytics API**: Dynamic analytics queries with configurable parameters
2. **Server-Sent Events**: Efficient incremental updates during replay
3. **Session-Based Replay**: Create, monitor, and stop replay sessions
4. **Comprehensive Endpoints**: Full coverage of asset data, analytics, and replay
5. **Standard Error Handling**: HTTP status codes + JSON error objects
6. **Small Team Scale**: Support 5-10 concurrent clients

**Technology Stack:**
- Framework: `axum`
- Real-time: Server-Sent Events (SSE)
- Format: JSON
- Auth: None (POC)
