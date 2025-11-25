# Analytics Dashboard - React UI

React frontend for the Analytics POC, built with Vite, Material-UI, and Recharts.

## Features

- **Multi-Asset Selection**: Select multiple assets to compare
- **Quick Analytics Presets**: Returns, 10/20/50-day Volatility
- **API URL Display**: Copy REST API URLs for curl testing
- **Time-Series Charts**: View historical analytics data
- **Real-Time Replay**: Watch analytics update live via SSE
- **Progress Tracking**: Monitor replay progress

## Prerequisites

- Node.js 18+ (20.19+ or 22.12+ recommended)
- Backend API server running on `localhost:3000`

## Installation

```bash
cd frontend
npm install
```

## Development

Start the development server:

```bash
npm run dev
```

The app will be available at: `http://localhost:5173`

## Build

Build for production:

```bash
npm run build
```

Output will be in `dist/` directory.

## Usage

### 1. Start the Backend Server

First, ensure the analytics API server is running:

```bash
# In the project root
cargo run --bin analytics-server
```

### 2. Start the Frontend

```bash
cd frontend
npm run dev
```

### 3. Use the Dashboard

1. **Select Assets**: Check one or more assets (AAPL, MSFT, GOOG)
2. **Choose Analytics**: Click a preset button (e.g., "20-Day Volatility")
3. **View Chart**: Historical data loads automatically
4. **Copy API URL**: Click "Copy" to get the REST API URL
5. **Start Replay**: Click "Start Replay" to watch real-time updates
6. **Monitor Progress**: Progress bar shows replay status
7. **Stop Replay**: Click "Stop" to end the session

## Architecture

- **Vite**: Fast build tool and dev server
- **React 18**: UI framework with hooks
- **TypeScript**: Type safety
- **Material-UI**: Component library
- **Recharts**: Charting library
- **Axios**: HTTP client
- **EventSource**: Server-Sent Events for real-time updates

## API Integration

The frontend connects to the backend API via:

- **Base URL**: `http://localhost:3000`
- **Proxy**: Vite dev server proxies `/api` requests
- **Endpoints**:
  - `GET /assets` - List available assets
  - `GET /analytics/{asset}/{type}` - Pull-mode analytics
  - `POST /replay` - Create replay session
  - `GET /stream/{session_id}` - SSE stream
  - `DELETE /replay/{session_id}` - Stop session

## Components

- **AssetSelector**: Multi-select checkboxes for assets
- **AnalyticsPresets**: Quick preset buttons
- **ApiUrlDisplay**: Display and copy API URLs
- **Chart**: Recharts line chart for time-series
- **ReplayControls**: Start/stop replay with progress bar

## Troubleshooting

**Port already in use:**
```bash
# Kill process on port 5173
lsof -ti:5173 | xargs kill -9
```

**Backend not responding:**
- Ensure backend server is running on `localhost:3000`
- Check backend logs for errors
- Verify database file exists

**Build warnings about chunk size:**
- Normal for POC with all dependencies included
- Can optimize later with code splitting

## Future Enhancements

- Pause/resume replay
- Adjustable replay speed
- Save/load configurations
- Export chart data
- Multiple comparison modes
