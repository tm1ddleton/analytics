# Specification: React UI Dashboard

**Date:** 2025-11-25  
**Status:** Approved  
**Roadmap Item:** 10  
**Size:** L (Large)

---

## Goal

Create a React frontend dashboard that connects to the REST API, displays real-time analytics updates via Server-Sent Events, shows asset data visualization with charts, and includes controls for replay sessions and asset selection.

The dashboard should:
- Provide a clean, single-page interface for all functionality
- Allow multi-asset selection for analytics queries
- Display historical analytics via pull-mode API calls
- Show real-time analytics updates during replay sessions via SSE
- Include quick preset buttons for common analytics configurations
- Display the REST API URL for pull-mode queries (copy-paste for curl)
- Use Material-UI for professional appearance
- Implement basic but effective line charts with Recharts

---

## User Stories

### Story 1: View Historical Analytics
**As a** portfolio analyst  
**I want to** select multiple assets and view their historical volatility  
**So that I can** compare performance across assets

**Acceptance Criteria:**
- Can select multiple assets via checkboxes
- Can choose analytics preset (e.g., "20-day Volatility")
- Chart displays time-series for all selected assets
- Can see the API URL that was called
- Chart loads within 2 seconds for 1 year of data

### Story 2: Watch Real-Time Replay
**As a** trader  
**I want to** start a replay session and watch analytics update in real-time  
**So that I can** visualize how analytics change over time

**Acceptance Criteria:**
- Can click "Start Replay" button
- Chart updates smoothly as new data arrives via SSE
- Progress bar shows replay progress
- Can stop replay at any time
- Chart scrolls to show new data as it arrives

### Story 3: Quick Analytics Configuration
**As a** user  
**I want to** quickly select common analytics without complex configuration  
**So that I can** efficiently explore different metrics

**Acceptance Criteria:**
- Preset buttons: Returns, 10-day Vol, 20-day Vol, 50-day Vol
- Single click selects preset and updates chart
- API URL updates to show the query parameters
- No need to manually enter parameters

### Story 4: Copy API URLs
**As a** developer  
**I want to** see the REST API URLs being called  
**So that I can** test the API directly with curl or integrate elsewhere

**Acceptance Criteria:**
- API URL displayed prominently
- Can copy URL to clipboard
- URL updates when assets or analytics change
- URL is valid and can be used directly in curl

---

## Specific Requirements

### 1. Technology Stack

**Core:**
- **Vite** - Build tool and dev server
- **React 18** - UI framework
- **TypeScript** - Type safety (optional but recommended)

**UI Components:**
- **Material-UI (MUI)** - Component library
- **Recharts** - Charting library

**HTTP & SSE:**
- **Axios** - HTTP requests to REST API
- **EventSource** - Native SSE support

**Development:**
- **ESLint** - Code linting
- **Prettier** - Code formatting

### 2. Project Structure

```
frontend/
├── public/
│   └── index.html
├── src/
│   ├── components/
│   │   ├── AssetSelector.tsx
│   │   ├── AnalyticsPresets.tsx
│   │   ├── ApiUrlDisplay.tsx
│   │   ├── Chart.tsx
│   │   └── ReplayControls.tsx
│   ├── services/
│   │   ├── api.ts
│   │   └── sse.ts
│   ├── types/
│   │   └── index.ts
│   ├── App.tsx
│   ├── App.css
│   └── main.tsx
├── package.json
├── vite.config.ts
└── tsconfig.json
```

### 3. Component Specifications

#### 3.1: AssetSelector Component

**Purpose:** Allow user to select multiple assets for analysis

**UI Elements:**
- Section title: "Select Assets"
- Checkbox list of available assets (AAPL, MSFT, GOOG, etc.)
- "Select All" / "Clear All" buttons
- Inline error message if no assets selected

**State:**
```typescript
interface AssetSelectorProps {
  availableAssets: string[];
  selectedAssets: string[];
  onSelectionChange: (assets: string[]) => void;
  loading: boolean;
  error: string | null;
}
```

**Behavior:**
- Load assets from `GET /assets` on mount
- Allow selecting/deselecting individual assets
- Show loading spinner while fetching
- Display error inline if fetch fails

---

#### 3.2: AnalyticsPresets Component

**Purpose:** Quick selection of common analytics configurations

**UI Elements:**
- Section title: "Analytics"
- Button group with presets:
  - "Returns"
  - "10-Day Volatility"
  - "20-Day Volatility"
  - "50-Day Volatility"
- Selected preset highlighted

**State:**
```typescript
interface Preset {
  label: string;
  analyticType: 'returns' | 'volatility';
  params?: { window?: number };
}

interface AnalyticsPresetsProps {
  selectedPreset: Preset | null;
  onPresetChange: (preset: Preset) => void;
}
```

**Presets:**
```typescript
const PRESETS: Preset[] = [
  { label: "Returns", analyticType: "returns" },
  { label: "10-Day Volatility", analyticType: "volatility", params: { window: 10 } },
  { label: "20-Day Volatility", analyticType: "volatility", params: { window: 20 } },
  { label: "50-Day Volatility", analyticType: "volatility", params: { window: 50 } },
];
```

---

#### 3.3: ApiUrlDisplay Component

**Purpose:** Show the REST API URL for current query

**UI Elements:**
- Section title: "API URL"
- Text field with URL (read-only)
- "Copy" button
- Small helper text: "Use this URL with curl to query the API directly"

**State:**
```typescript
interface ApiUrlDisplayProps {
  url: string;
}
```

**URL Format:**
```
http://localhost:3000/analytics/AAPL/volatility?start=2024-01-01&end=2024-12-31&window=20
```

**Behavior:**
- Updates automatically when assets/analytics change
- Copy button copies URL to clipboard
- Shows success feedback on copy

---

#### 3.4: Chart Component

**Purpose:** Display time-series analytics data

**UI Elements:**
- Recharts LineChart
- X-axis: Timestamp (formatted as dates)
- Y-axis: Value
- Lines for each selected asset
- Legend showing asset names
- Title showing analytic type

**State:**
```typescript
interface ChartData {
  timestamp: string;
  [asset: string]: number | string;
}

interface ChartProps {
  data: ChartData[];
  assets: string[];
  analyticType: string;
  loading: boolean;
  error: string | null;
}
```

**Behavior:**
- Display loading spinner while fetching data
- Show inline error if query fails
- Auto-scale Y-axis based on data
- X-axis shows dates in readable format (MM/DD or MMM DD)
- Different colored line per asset
- Smooth transitions when data updates (replay mode)

**Example Data Format:**
```typescript
[
  { timestamp: "2024-01-01", AAPL: 0.023, MSFT: 0.019 },
  { timestamp: "2024-01-02", AAPL: 0.025, MSFT: 0.021 },
]
```

---

#### 3.5: ReplayControls Component

**Purpose:** Control replay sessions and show progress

**UI Elements:**
- "Start Replay" button (primary)
- "Stop Replay" button (secondary, only when running)
- Progress bar showing replay progress (0-100%)
- Status text: "Ready" / "Running" / "Stopped" / "Completed"
- Inline error message if session creation fails

**State:**
```typescript
interface ReplayControlsProps {
  onStartReplay: () => Promise<void>;
  onStopReplay: () => Promise<void>;
  status: 'idle' | 'running' | 'stopped' | 'completed';
  progress: number;
  error: string | null;
}
```

**Behavior:**
- Start button creates replay session via `POST /replay`
- Opens SSE connection to `/stream/{session_id}`
- Updates chart as SSE events arrive
- Progress bar updates based on events
- Stop button calls `DELETE /replay/{session_id}`
- Closes SSE connection on stop or completion

---

### 4. Page Layout

**Single-Page Layout:**

```
┌─────────────────────────────────────────────────┐
│  Analytics Dashboard                             │
├─────────────────────────────────────────────────┤
│                                                  │
│  ┌─ Select Assets ────────────────────────┐    │
│  │ ☑ AAPL  ☑ MSFT  ☐ GOOG                 │    │
│  │ [Select All] [Clear All]                │    │
│  └──────────────────────────────────────────┘    │
│                                                  │
│  ┌─ Analytics ─────────────────────────────┐    │
│  │ [Returns] [10-Day Vol] [20-Day Vol]     │    │
│  │                        [50-Day Vol]      │    │
│  └──────────────────────────────────────────┘    │
│                                                  │
│  ┌─ API URL ───────────────────────────────┐    │
│  │ http://localhost:3000/analytics/...      │    │
│  │ [Copy]                                    │    │
│  └──────────────────────────────────────────┘    │
│                                                  │
│  ┌─ Chart ─────────────────────────────────┐    │
│  │                                          │    │
│  │     [Line chart with time-series data]   │    │
│  │                                          │    │
│  └──────────────────────────────────────────┘    │
│                                                  │
│  ┌─ Replay ────────────────────────────────┐    │
│  │ [Start Replay]  [Stop]                   │    │
│  │ Progress: ▓▓▓▓▓▓░░░░ 60%                 │    │
│  │ Status: Running...                       │    │
│  └──────────────────────────────────────────┘    │
│                                                  │
└─────────────────────────────────────────────────┘
```

**Spacing:**
- 24px padding around page
- 16px margin between sections
- Material-UI Paper components for sections
- Elevation 1 for subtle depth

---

### 5. API Integration

#### 5.1: api.ts Service

**Functions:**

```typescript
// Fetch available assets
async function getAssets(): Promise<Asset[]>

// Fetch analytics data (pull-mode)
async function getAnalytics(
  asset: string,
  analyticType: string,
  params: AnalyticsParams
): Promise<AnalyticsResponse>

// Create replay session
async function createReplaySession(
  assets: string[],
  analytics: AnalyticConfig[],
  startDate: string,
  endDate: string
): Promise<SessionResponse>

// Stop replay session
async function stopReplaySession(
  sessionId: string
): Promise<void>
```

**Configuration:**
```typescript
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';
```

#### 5.2: sse.ts Service

**Functions:**

```typescript
// Connect to SSE stream
function connectToStream(
  sessionId: string,
  onUpdate: (data: AnalyticUpdate) => void,
  onProgress: (progress: number) => void,
  onComplete: () => void,
  onError: (error: Error) => void
): EventSource

// Close SSE connection
function closeStream(eventSource: EventSource): void
```

**Event Handling:**
```typescript
eventSource.addEventListener('update', (event) => {
  const data = JSON.parse(event.data);
  onUpdate(data);
});

eventSource.addEventListener('progress', (event) => {
  const { progress } = JSON.parse(event.data);
  onProgress(progress);
});

eventSource.addEventListener('complete', () => {
  onComplete();
  eventSource.close();
});

eventSource.onerror = (error) => {
  onError(error);
  eventSource.close();
};
```

---

### 6. Data Flow

#### 6.1: Initial Load (Pull-Mode)

1. User selects assets (AAPL, MSFT)
2. User clicks analytics preset (20-Day Volatility)
3. App builds API URL
4. App calls `GET /analytics/AAPL/volatility?window=20&start=...&end=...`
5. App calls `GET /analytics/MSFT/volatility?window=20&start=...&end=...`
6. App combines results into chart data format
7. Chart displays combined data

#### 6.2: Replay Mode (SSE Updates)

1. User clicks "Start Replay"
2. App calls `POST /replay` with selected assets/analytics
3. Server returns `session_id`
4. App connects to `GET /stream/{session_id}`
5. SSE events arrive with new data points
6. App appends points to chart data
7. Chart re-renders with new data (smooth update)
8. Progress bar updates
9. User clicks "Stop" or replay completes
10. App closes SSE connection

---

### 7. State Management

**App-Level State:**

```typescript
interface AppState {
  // Assets
  availableAssets: Asset[];
  selectedAssets: string[];
  assetsLoading: boolean;
  assetsError: string | null;
  
  // Analytics
  selectedPreset: Preset | null;
  
  // Chart data
  chartData: ChartData[];
  chartLoading: boolean;
  chartError: string | null;
  
  // Replay
  replaySession: {
    id: string | null;
    status: 'idle' | 'running' | 'stopped' | 'completed';
    progress: number;
    error: string | null;
  };
  
  // SSE
  eventSource: EventSource | null;
}
```

**State Updates:**
- Use React's `useState` for each section
- Pass state and setters to child components
- Components update their own section's state

---

### 8. Error Handling

**Error Types:**

1. **Network Errors** - Failed API calls
   - Display: "Failed to fetch data. Please check if the server is running."
   - Location: Inline below relevant component

2. **No Assets Selected** - User tries to query without selecting assets
   - Display: "Please select at least one asset"
   - Location: Below asset selector

3. **Session Creation Failed** - Replay session cannot start
   - Display: Error message from API
   - Location: Below replay controls

4. **SSE Connection Lost** - Stream disconnects unexpectedly
   - Display: "Connection lost. Replay stopped."
   - Location: Below replay controls

**Error UI:**
```typescript
{error && (
  <Typography color="error" variant="caption">
    {error}
  </Typography>
)}
```

---

### 9. Styling

**Theme:**
- Material-UI default theme
- Primary color: Blue (#1976d2)
- Secondary color: Orange (#ff9800)
- Clean, professional appearance

**Typography:**
- Headers: Material-UI `h5` or `h6`
- Body: Material-UI `body1`
- Captions: Material-UI `caption`

**Spacing:**
- Use Material-UI spacing system (8px base)
- Consistent padding: `padding: 2` (16px)
- Consistent margin: `margin: 2` (16px)

---

### 10. Performance

**Optimization:**
- Debounce API calls when selection changes
- Limit chart data points (show last 1000 points max)
- Use React.memo for components that don't need frequent re-renders
- Close SSE connections properly to avoid memory leaks

**Target Metrics:**
- Initial page load: < 1 second
- API query response: < 2 seconds for 1 year of data
- SSE update latency: < 100ms from server to UI update
- Chart re-render: < 16ms (60fps)

---

### 11. Development Workflow

**Setup:**
```bash
npm create vite@latest frontend -- --template react-ts
cd frontend
npm install
npm install @mui/material @emotion/react @emotion/styled
npm install recharts
npm install axios
npm run dev
```

**Development Server:**
- Vite dev server: `http://localhost:5173`
- API proxy in vite.config.ts to avoid CORS:
```typescript
export default defineConfig({
  server: {
    proxy: {
      '/api': 'http://localhost:3000'
    }
  }
})
```

**Build:**
```bash
npm run build
# Output in dist/
```

---

### 12. Testing

**Manual Testing:**
1. Asset selection works
2. Presets update chart correctly
3. API URL displays and copies
4. Chart displays data
5. Replay starts and updates chart
6. Progress bar updates
7. Stop button works
8. Error messages appear correctly

**Integration Testing:**
1. Test with live API server
2. Test with real data in database
3. Test replay with actual SSE events
4. Test error scenarios (server down, no data)

---

## Implementation Order

1. **Project Setup** - Vite, React, dependencies
2. **API Service** - HTTP client, types
3. **AssetSelector** - Load and display assets
4. **AnalyticsPresets** - Preset buttons
5. **ApiUrlDisplay** - Show and copy URLs
6. **Chart** - Display pull-mode data
7. **ReplayControls** - Start/stop replay
8. **SSE Integration** - Real-time updates
9. **Error Handling** - Inline errors
10. **Styling & Polish** - Final UI improvements

---

## Success Criteria

- ✅ Single-page React app with clean UI
- ✅ Multi-asset selection with checkboxes
- ✅ Quick analytics presets
- ✅ API URL display with copy functionality
- ✅ Line charts showing time-series data
- ✅ Real-time chart updates via SSE
- ✅ Basic replay controls (start, stop, progress)
- ✅ Inline error messages
- ✅ Works with live API server
- ✅ Responsive design (1024px+ width)

