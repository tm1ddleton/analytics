# Analytics POC - Real-Time Market Data Analytics Engine

A high-performance analytics engine built in Rust with a React frontend, demonstrating real-time computation of financial analytics with push-mode and pull-mode capabilities.

## 🎯 Overview

This POC implements a complete analytics platform featuring:

- **Rust Backend**: High-performance analytics engine with DAG-based computation
- **REST API**: Full HTTP API with Server-Sent Events for real-time updates
- **React Dashboard**: Interactive UI for visualizing analytics and replay
- **Real-Time Updates**: Watch analytics compute incrementally via SSE
- **Historical Queries**: Pull complete time-series for any date range

## 📋 Prerequisites

- **Rust** - Latest stable toolchain (`rustc --version`)
- **Node.js 18+** - For the React frontend (`node --version`)
  - Currently uses Vite 5.x (compatible with Node 18-20)
- **SQLite** - Included via rusqlite bundled feature

## 🚀 Quick Start

### Run the Demo

```bash
./run-demo.sh
```

This will:
1. Build and start the backend API server (port 3000)
2. Start the React frontend (port 5173)
3. Open your browser to http://localhost:5173

**That's it!** The demo is ready to use.

### Manual Start

If you prefer to run components separately:

**Backend:**
```bash
cargo run --bin analytics-server
```

**Frontend:**
```bash
cd frontend
npm install  # first time only
npm run dev
```

## 📊 Features Implemented

### ✅ POC Phase Complete (Items 1-10)

1. **Core Asset Data Model** - First-class asset objects with time-series
2. **SQLite Data Storage** - Simple, fast storage for POC
3. **Yahoo Finance Downloader** - Historical data ingestion
4. **DAG Computation Framework** - Dependency-based analytics pipeline
5. **Push-Mode Analytics** - Incremental updates as data arrives
6. **Basic Analytics Library** - Returns and volatility calculations
7. **High-Speed Replay** - Feed historical data at high speed
8. **Pull-Mode Analytics** - On-demand historical computation
9. **REST API + SSE** - HTTP server with real-time streaming
10. **React Dashboard** - Interactive visualization UI

## 🎮 Using the Dashboard

### 1. Select Assets
- Check one or more assets (AAPL, MSFT, GOOG)
- Use "Select All" or "Clear All" buttons

### 2. Choose Analytics
Click a preset button:
- **Returns** - Log returns calculation
- **10-Day Volatility** - 10-day rolling volatility
- **20-Day Volatility** - 20-day rolling volatility  
- **50-Day Volatility** - 50-day rolling volatility

### 3. View Historical Data
- Chart loads automatically with pull-mode query
- Shows complete time-series for selected date range

### 4. Copy API URL
- Click "Copy" to get the REST API URL
- Test directly with curl or other tools

### 5. Start Replay
- Click "Start Replay" to begin real-time simulation
- Watch analytics update incrementally on the chart
- Monitor progress bar
- Click "Stop" to end the session

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────┐
│              React Dashboard                     │
│         (Vite + MUI + Recharts)                 │
│                                                  │
│  • Multi-asset selection                        │
│  • Analytics presets                            │
│  • Time-series charts                           │
│  • Real-time updates (SSE)                      │
└───────────────┬─────────────────────────────────┘
                │ HTTP + SSE
┌───────────────▼─────────────────────────────────┐
│          REST API Server (axum)                  │
│                                                  │
│  • GET /assets                                   │
│  • GET /analytics/{asset}/{type}  (Pull-mode)   │
│  • POST /replay                                  │
│  • GET /stream/{session_id}       (SSE)         │
└───────────────┬─────────────────────────────────┘
                │
┌───────────────▼─────────────────────────────────┐
│           Analytics Engine (Rust)                │
│                                                  │
│  ┌──────────────────────────────────────────┐  │
│  │  DAG Framework                            │  │
│  │  • Dependency resolution                  │  │
│  │  • Topological execution                  │  │
│  │  • Cycle detection                        │  │
│  └──────────────────────────────────────────┘  │
│                                                  │
│  ┌────────────────┐  ┌─────────────────────┐   │
│  │  Pull-Mode     │  │   Push-Mode         │   │
│  │  • Batch query │  │   • Incremental     │   │
│  │  • Full range  │  │   • Callbacks       │   │
│  └────────────────┘  └─────────────────────┘   │
│                                                  │
│  ┌──────────────────────────────────────────┐  │
│  │  Analytics Library                        │  │
│  │  • Returns calculation                    │  │
│  │  • Rolling volatility                     │  │
│  └──────────────────────────────────────────┘  │
└───────────────┬─────────────────────────────────┘
                │
┌───────────────▼─────────────────────────────────┐
│         SQLite Data Provider                     │
│                                                  │
│  • Asset metadata                                │
│  • Time-series data                              │
│  • Date-range queries                            │
└──────────────────────────────────────────────────┘
```

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

**Result:** 299 tests passing ✅

## 📁 Project Structure

```
analytics/
├── src/
│   ├── asset_key.rs           # Asset identification
│   ├── sqlite_provider.rs     # Data storage
│   ├── yahoo_finance.rs       # Data ingestion
│   ├── dag.rs                 # DAG framework
│   ├── analytics.rs           # Analytics functions
│   ├── push_mode.rs           # Push-mode engine
│   ├── replay.rs              # Replay system
│   └── server/                # REST API
│       ├── mod.rs
│       ├── handlers.rs
│       ├── routes.rs
│       ├── state.rs
│       └── error.rs
├── frontend/                   # React dashboard
│   ├── src/
│   │   ├── components/
│   │   ├── services/
│   │   ├── types/
│   │   └── App.tsx
│   └── package.json
├── examples/                   # Example programs
├── docs/                       # API documentation
└── run-demo.sh                # Demo launcher
```

## 🔧 API Examples

### Query Historical Analytics (Pull-Mode)

```bash
# Get 20-day volatility for AAPL
curl "http://localhost:3000/analytics/AAPL/volatility?start=2024-01-01&end=2024-12-31&window=20"
```

### List Available Assets

```bash
curl http://localhost:3000/assets
```

### Create Replay Session

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

Full API documentation: [`docs/API.md`](docs/API.md)

## 🛠️ Technology Stack

### Backend
- **Rust** - Systems programming language
- **axum** - Web framework
- **tokio** - Async runtime
- **daggy** - DAG library
- **rusqlite** - SQLite bindings
- **serde** - Serialization

### Frontend
- **React 18** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool
- **Material-UI** - Component library
- **Recharts** - Charting library
- **Axios** - HTTP client

## 📈 Performance

- **Pull-Mode Query**: < 1 second for 1 year of data
- **SSE Event Latency**: < 50ms from computation to UI
- **Chart Updates**: 60fps smooth rendering
- **Concurrent Sessions**: Supports 10+ simultaneous replay sessions

## 🚧 Future Enhancements (Post-POC)

- Embedded Rust API
- Python PyO3 bindings
- Polars dataframe integration
- Real-time data ingestion
- Strategy output system
- Distributed architecture
- Performance optimizations

See [`agent-os/product/roadmap.md`](agent-os/product/roadmap.md) for complete roadmap.

## 📝 License

POC/Demo project - not licensed for production use.

## 🤝 Contributing

This is a proof-of-concept project. For production use, see the roadmap for planned enhancements.

---

**Built with ❤️ using Rust and React**

