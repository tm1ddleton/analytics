# Requirements Gathering: React UI Dashboard

## Clarifying Questions & Answers

### 1. React Framework/Setup
**Q:** Which React setup would you prefer?  
**A:** B - Vite + React

**Decision Rationale:** Vite provides fast development with HMR, modern ES modules, and optimized builds. Lighter than CRA, perfect for POC.

### 2. Charting Library
**Q:** Which library for time-series visualization?  
**A:** A - Recharts

**Decision Rationale:** Simple, declarative React charts. Easy to integrate, good for time-series data, responsive out of the box.

### 3. UI Component Library
**Q:** For buttons, inputs, layout?  
**A:** A - Material-UI (MUI)

**Decision Rationale:** Comprehensive component library with Material Design. Professional look, accessible, well-documented.

### 4. Page Layout
**Q:** How should the dashboard be organized?  
**A:** A - Single page - Everything visible at once

**Decision Rationale:** Simple, no navigation needed for POC. All functionality visible and accessible immediately.

### 5. Asset Selection
**Q:** How should users select assets?  
**A:** B - Multi-select with checkboxes

**Decision Rationale:** Allows selecting multiple assets for comparison. Clear visual feedback of selected assets.

### 6. Analytics Configuration
**Q:** How to configure analytics?  
**A:** B - Quick preset buttons (10-day vol, 20-day vol, etc.) + show cut & paste URL for REST API in pull mode

**Decision Rationale:** Fast workflow with presets. URL display helps users understand API calls and enables curl testing.

### 7. Chart Features
**Q:** What chart features are essential?  
**A:** A - Basic - Just display the line chart

**Decision Rationale:** Keep it simple for POC. Focus on core functionality - displaying time-series data clearly.

### 8. Real-Time Updates
**Q:** How should SSE updates appear?  
**A:** B - Chart scrolls/updates as data arrives

**Decision Rationale:** Live feel as replay progresses. Chart updates continuously showing the growing time series.

### 9. Replay Controls
**Q:** What replay controls are needed?  
**A:** A - Basic - Start, Stop buttons + progress bar

**Decision Rationale:** Essential controls only. Clear feedback on replay status and progress.

### 10. Error Handling
**Q:** How should errors be displayed?  
**A:** C - Inline error messages

**Decision Rationale:** Contextual feedback. Errors appear near the relevant component (asset selection, chart area, etc.).

## Summary

The React UI Dashboard will provide:

1. **Technology Stack:**
   - Vite + React
   - Material-UI for components
   - Recharts for visualization
   - Axios for HTTP requests
   - EventSource for SSE

2. **Layout:**
   - Single-page application
   - Top section: Asset selection (multi-select checkboxes)
   - Middle section: Analytics presets + API URL display
   - Bottom section: Chart display + replay controls

3. **Features:**
   - Multi-asset selection with checkboxes
   - Quick analytics presets (Returns, 10-day vol, 20-day vol, etc.)
   - Display REST API URL for pull-mode queries
   - Basic line charts showing time-series data
   - Real-time chart updates via SSE during replay
   - Start/Stop replay buttons with progress bar
   - Inline error messages for failed operations

4. **User Flow:**
   - Select one or more assets via checkboxes
   - Choose analytics preset (or custom)
   - See API URL for the query
   - View historical data chart (pull-mode)
   - Start replay to see real-time updates
   - Monitor progress bar
   - Stop replay when done

**Visual Assets:** None provided - will design clean, functional UI

## Technical Requirements

### API Integration
- Connect to `http://localhost:3000` (configurable)
- Use endpoints:
  - `GET /assets` - List assets for checkboxes
  - `GET /analytics/{asset}/{type}` - Pull-mode queries
  - `POST /replay` - Create replay session
  - `GET /stream/{session_id}` - SSE stream
  - `DELETE /replay/{session_id}` - Stop session

### State Management
- React useState for component state
- Asset selection state (array of selected assets)
- Chart data state (time-series points)
- Replay session state (session_id, status, progress)
- Error state (per component)

### Chart Updates
- Initial load: Query pull-mode API, display full time series
- Replay mode: Start session, connect SSE, append points as they arrive
- Chart library auto-scales Y-axis, X-axis shows timestamps

### Responsive Design
- Works on desktop (1920x1080 primary)
- Minimum width: 1024px
- Clean, readable typography
- Proper spacing and padding

