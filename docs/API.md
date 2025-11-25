# REST API Documentation

## Overview

The Analytics API Server provides REST endpoints for querying asset data and analytics, plus Server-Sent Events (SSE) for real-time replay updates.

**Base URL:** `http://127.0.0.1:3000`

## Authentication

No authentication required for POC.

## Endpoints

### Health Check

**GET /health**

Returns server status.

**Response:**
```json
{
  "status": "ok"
}
```

---

### List Assets

**GET /assets**

Returns all available assets.

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
    }
  ]
}
```

---

### Get Asset Data

**GET /assets/{asset}/data**

Returns raw price data for an asset.

**Parameters:**
- `start` (required): Start date (YYYY-MM-DD)
- `end` (required): End date (YYYY-MM-DD)

**Example:**
```bash
curl "http://localhost:3000/assets/AAPL/data?start=2024-01-01&end=2024-01-31"
```

**Response:**
```json
{
  "asset": "AAPL",
  "start_date": "2024-01-01",
  "end_date": "2024-01-31",
  "data": [
    {
      "timestamp": "2024-01-01T00:00:00Z",
      "close": 185.64
    }
  ]
}
```

---

### Get Analytics

**GET /analytics/{asset}/{type}**

Executes pull-mode analytics query.

**Path Parameters:**
- `asset`: Asset identifier (e.g., "AAPL")
- `type`: Analytic type ("returns" or "volatility")

**Query Parameters:**
- `start` (required): Start date (YYYY-MM-DD)
- `end` (required): End date (YYYY-MM-DD)
- `window` (optional, for volatility): Window size (default: 10)

**Example:**
```bash
curl "http://localhost:3000/analytics/AAPL/volatility?start=2024-01-01&end=2024-12-31&window=20"
```

**Response:**
```json
{
  "asset": "AAPL",
  "analytic": "volatility",
  "parameters": {
    "window": "20"
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

---

### Batch Analytics

**POST /analytics/batch**

Executes multiple analytics queries in parallel.

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
        "window": "10"
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
    }
  ],
  "errors": []
}
```

---

### List Analytics

**GET /dag/nodes**

Returns available analytic types and their parameters.

**Response:**
```json
{
  "analytics": [
    {
      "type": "returns",
      "description": "Log returns calculation",
      "parameters": [],
      "burnin_days": "1"
    },
    {
      "type": "volatility",
      "description": "Rolling volatility (population std dev)",
      "parameters": [
        {
          "name": "window",
          "type": "integer",
          "required": false,
          "default": "10",
          "description": "Rolling window size in days"
        }
      ],
      "burnin_days": "window + 1"
    }
  ]
}
```

---

### Create Replay Session

**POST /replay**

Creates a new replay session for streaming analytics.

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
        "window": "10"
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

---

### Get Session Status

**GET /replay/{session_id}**

Returns replay session status.

**Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "running",
  "assets": ["AAPL"],
  "analytics": ["volatility"],
  "start_date": "2024-01-01",
  "end_date": "2024-12-31",
  "current_date": "2024-06-15",
  "progress": 0.45,
  "created_at": "2024-12-01T10:30:00Z",
  "started_at": "2024-12-01T10:30:01Z",
  "stream_url": "/stream/550e8400-e29b-41d4-a716-446655440000"
}
```

---

### Stop Session

**DELETE /replay/{session_id}**

Stops a running replay session.

**Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "stopped",
  "message": "Replay session stopped"
}
```

---

### SSE Stream

**GET /stream/{session_id}**

Server-Sent Events stream for replay updates.

**Example (JavaScript):**
```javascript
const eventSource = new EventSource('http://localhost:3000/stream/SESSION_ID');

eventSource.addEventListener('update', (event) => {
  const data = JSON.parse(event.data);
  console.log('Update:', data);
});

eventSource.addEventListener('connected', (event) => {
  const data = JSON.parse(event.data);
  console.log('Connected:', data);
});
```

---

## Error Responses

All errors follow this format:

```json
{
  "error": "ErrorType",
  "message": "Human-readable error description"
}
```

**HTTP Status Codes:**
- `200 OK` - Success
- `201 Created` - Resource created
- `400 Bad Request` - Invalid parameters
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error
- `503 Service Unavailable` - Session limit reached

## Quick Start

1. Start the server:
```bash
cargo run --bin analytics-server
```

2. Query available assets:
```bash
curl http://localhost:3000/assets
```

3. Get analytics:
```bash
curl "http://localhost:3000/analytics/AAPL/volatility?start=2024-01-01&end=2024-12-31&window=10"
```

4. Create replay session:
```bash
curl -X POST http://localhost:3000/replay \
  -H "Content-Type: application/json" \
  -d '{
    "assets": ["AAPL"],
    "analytics": [{"type": "volatility", "parameters": {"window": "10"}}],
    "start_date": "2024-01-01",
    "end_date": "2024-12-31"
  }'
```

