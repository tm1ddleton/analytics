# Tasks: React UI Dashboard

**Spec:** `agent-os/specs/2025-11-25-react-ui-dashboard/spec.md`  
**Date:** 2025-11-25  
**Status:** Ready for Implementation

---

## Overview

This tasks list breaks down the React UI Dashboard implementation into 8 task groups, ordered by dependencies. The dashboard will provide a single-page interface for viewing historical analytics and watching real-time replay updates.

**Total Estimated Time:** ~8-10 hours

---

## Task Group 1: Project Setup

**Goal:** Initialize Vite + React project with TypeScript and dependencies

**Dependencies:** None

**Acceptance Criteria:**
- Vite project created and runs
- TypeScript configured
- All dependencies installed
- Dev server starts successfully

---

### Tasks

#### Task 1.1: Initialize Vite project
- [x] Run `npm create vite@latest frontend -- --template react-ts`
- [x] Navigate to frontend directory
- [x] Verify project structure created
- [x] Test dev server: `npm run dev`

**Estimated Time:** 10 minutes

---

#### Task 1.2: Install dependencies
- [x] Install Material-UI: `npm install @mui/material @emotion/react @emotion/styled`
- [x] Install Recharts: `npm install recharts`
- [x] Install Axios: `npm install axios`
- [x] Install types: `npm install -D @types/node`
- [x] Verify all packages in package.json

**Estimated Time:** 10 minutes

---

#### Task 1.3: Configure Vite
- [x] Create `vite.config.ts` with API proxy
- [x] Configure proxy: `/api` -> `http://localhost:3000`
- [x] Set up CORS handling
- [x] Test dev server restarts correctly

**Estimated Time:** 15 minutes

---

#### Task 1.4: Set up project structure
- [x] Create `src/components/` directory
- [x] Create `src/services/` directory
- [x] Create `src/types/` directory
- [x] Create placeholder files for each component
- [x] Update `src/App.tsx` with basic layout

**Estimated Time:** 15 minutes

---

## Task Group 2: Types and API Service

**Goal:** Define TypeScript types and create API client

**Dependencies:** Task Group 1

**Acceptance Criteria:**
- All types defined in `src/types/index.ts`
- API service functions implemented
- Can make requests to REST API
- Error handling in place

---

### Tasks

#### Task 2.1: Define TypeScript types
- [x] Create `src/types/index.ts`
- [x] Define `Asset` interface
- [x] Define `AnalyticsParams` interface
- [x] Define `AnalyticsResponse` interface
- [x] Define `ChartData` interface
- [x] Define `Preset` interface
- [x] Define `SessionResponse` interface
- [x] Define `AnalyticUpdate` interface

**Estimated Time:** 20 minutes

---

#### Task 2.2: Create API service
- [x] Create `src/services/api.ts`
- [x] Set up Axios instance with base URL
- [x] Implement `getAssets()` function
- [x] Implement `getAnalytics()` function
- [x] Implement `createReplaySession()` function
- [x] Implement `stopReplaySession()` function
- [x] Add error handling for all functions
- [x] Export all functions

**Estimated Time:** 30 minutes

---

#### Task 2.3: Test API service
- [x] Start backend server
- [x] Test `getAssets()` in browser console
- [x] Test `getAnalytics()` with sample parameters
- [x] Verify responses match expected types
- [x] Test error handling (server offline)

**Estimated Time:** 15 minutes

---

## Task Group 3: AssetSelector Component

**Goal:** Implement multi-asset selection with checkboxes

**Dependencies:** Task Groups 1, 2

**Acceptance Criteria:**
- Component displays list of assets
- Checkboxes work for selection
- Select All / Clear All buttons functional
- Loading state and errors displayed

---

### Tasks

#### Task 3.1: Create component structure
- [x] Create `src/components/AssetSelector.tsx`
- [x] Define component props interface
- [x] Set up component state
- [x] Create basic JSX structure with Material-UI

**Estimated Time:** 15 minutes

---

#### Task 3.2: Implement asset loading
- [x] Call `getAssets()` on component mount
- [x] Store assets in state
- [x] Display loading spinner while fetching
- [x] Handle errors and display inline message

**Estimated Time:** 20 minutes

---

#### Task 3.3: Implement selection logic
- [x] Render checkbox for each asset
- [x] Handle individual checkbox changes
- [x] Implement "Select All" button
- [x] Implement "Clear All" button
- [x] Emit selection changes to parent

**Estimated Time:** 25 minutes

---

#### Task 3.4: Styling and polish
- [x] Apply Material-UI styling
- [x] Add proper spacing
- [x] Test with multiple assets
- [x] Verify accessibility (keyboard navigation)

**Estimated Time:** 15 minutes

---

## Task Group 4: AnalyticsPresets Component

**Goal:** Quick selection buttons for common analytics

**Dependencies:** Task Groups 1, 2

**Acceptance Criteria:**
- Preset buttons display correctly
- Clicking preset updates selection
- Selected preset is highlighted
- Presets include Returns and Volatility variations

---

### Tasks

#### Task 4.1: Create component
- [x] Create `src/components/AnalyticsPresets.tsx`
- [x] Define props interface
- [x] Define preset configurations
- [x] Create button group layout

**Estimated Time:** 15 minutes

---

#### Task 4.2: Implement preset logic
- [x] Render button for each preset
- [x] Handle preset selection
- [x] Highlight selected preset
- [x] Emit preset change to parent

**Estimated Time:** 20 minutes

---

#### Task 4.3: Define presets
- [x] "Returns" preset
- [x] "10-Day Volatility" preset
- [x] "20-Day Volatility" preset
- [x] "50-Day Volatility" preset
- [x] Test each preset selection

**Estimated Time:** 10 minutes

---

## Task Group 5: ApiUrlDisplay and Chart Components

**Goal:** Display API URL and render time-series chart

**Dependencies:** Task Groups 1, 2, 3, 4

**Acceptance Criteria:**
- API URL displays correctly
- Copy button works
- Chart displays time-series data
- Chart handles multiple assets
- Loading and error states work

---

### Tasks

#### Task 5.1: Create ApiUrlDisplay component
- [x] Create `src/components/ApiUrlDisplay.tsx`
- [x] Display URL in read-only text field
- [x] Implement copy button
- [x] Show success feedback on copy
- [x] Add helper text

**Estimated Time:** 20 minutes

---

#### Task 5.2: Create Chart component structure
- [x] Create `src/components/Chart.tsx`
- [x] Define props interface
- [x] Set up Recharts LineChart
- [x] Configure axes (X: timestamp, Y: value)
- [x] Add responsive container

**Estimated Time:** 25 minutes

---

#### Task 5.3: Implement chart data rendering
- [x] Transform data for Recharts format
- [x] Render line for each asset
- [x] Add legend with asset names
- [x] Configure colors per asset
- [x] Add chart title

**Estimated Time:** 30 minutes

---

#### Task 5.4: Add loading and error states
- [x] Show loading spinner when fetching
- [x] Display inline error message on failure
- [x] Show "No data" message when empty
- [x] Test all states

**Estimated Time:** 15 minutes

---

## Task Group 6: ReplayControls Component

**Goal:** Start/stop replay with progress display

**Dependencies:** Task Groups 1, 2

**Acceptance Criteria:**
- Start button creates replay session
- Stop button terminates session
- Progress bar shows replay progress
- Status text updates correctly
- Errors displayed inline

---

### Tasks

#### Task 6.1: Create component structure
- [x] Create `src/components/ReplayControls.tsx`
- [x] Define props interface
- [x] Create button layout
- [x] Add progress bar
- [x] Add status text

**Estimated Time:** 20 minutes

---

#### Task 6.2: Implement start replay
- [x] Handle start button click
- [x] Call `createReplaySession()` API
- [x] Store session ID in state
- [x] Update status to "Running"
- [x] Emit session ID to parent

**Estimated Time:** 25 minutes

---

#### Task 6.3: Implement stop replay
- [x] Handle stop button click
- [x] Call `stopReplaySession()` API
- [x] Update status to "Stopped"
- [x] Reset progress bar
- [x] Emit stop event to parent

**Estimated Time:** 15 minutes

---

#### Task 6.4: Progress bar updates
- [x] Bind progress bar to progress prop
- [x] Format progress percentage
- [x] Style progress bar
- [x] Test progress updates

**Estimated Time:** 10 minutes

---

## Task Group 7: SSE Integration

**Goal:** Connect to SSE stream and update chart in real-time

**Dependencies:** Task Groups 1, 2, 5, 6

**Acceptance Criteria:**
- SSE connection established on replay start
- Chart updates as events arrive
- Progress updates from SSE events
- Connection closes properly
- Errors handled gracefully

---

### Tasks

#### Task 7.1: Create SSE service
- [x] Create `src/services/sse.ts`
- [x] Implement `connectToStream()` function
- [x] Set up EventSource
- [x] Handle 'update' events
- [x] Handle 'progress' events
- [x] Handle 'complete' events
- [x] Implement `closeStream()` function

**Estimated Time:** 30 minutes

---

#### Task 7.2: Integrate SSE in App
- [x] Connect to SSE stream after session creation
- [x] Pass session ID to SSE service
- [x] Handle update events: append to chart data
- [x] Handle progress events: update progress bar
- [x] Handle complete events: update status
- [x] Store EventSource in state

**Estimated Time:** 35 minutes

---

#### Task 7.3: Implement chart updates
- [x] Append new data points to existing chart data
- [x] Ensure chart re-renders smoothly
- [x] Limit chart data to last 1000 points (performance)
- [x] Test with real replay session

**Estimated Time:** 25 minutes

---

#### Task 7.4: Clean up connections
- [x] Close SSE on stop button click
- [x] Close SSE on component unmount
- [x] Close SSE on completion
- [x] Prevent memory leaks
- [x] Test connection cleanup

**Estimated Time:** 15 minutes

---

## Task Group 8: App Integration and Polish

**Goal:** Wire all components together in main App and add final polish

**Dependencies:** Task Groups 1-7

**Acceptance Criteria:**
- All components integrated in App.tsx
- Data flows correctly between components
- Pull-mode queries work end-to-end
- Replay mode works end-to-end
- Error handling complete
- UI is polished and professional

---

### Tasks

#### Task 8.1: Integrate components in App
- [x] Import all components
- [x] Set up app-level state
- [x] Wire AssetSelector
- [x] Wire AnalyticsPresets
- [x] Wire ApiUrlDisplay
- [x] Wire Chart
- [x] Wire ReplayControls
- [x] Define state update handlers

**Estimated Time:** 30 minutes

---

#### Task 8.2: Implement pull-mode workflow
- [x] When preset selected, build API URL
- [x] Fetch analytics for all selected assets
- [x] Combine results into chart data format
- [x] Update chart with fetched data
- [x] Handle errors at each step
- [x] Test complete workflow

**Estimated Time:** 35 minutes

---

#### Task 8.3: Implement replay workflow
- [x] When start clicked, create session
- [x] Connect to SSE stream
- [x] Update chart as events arrive
- [x] Update progress bar
- [x] Handle stop button
- [x] Test complete workflow

**Estimated Time:** 30 minutes

---

#### Task 8.4: Error handling
- [x] Add error state for each section
- [x] Display inline errors
- [x] Test error scenarios
- [x] Ensure errors don't crash app

**Estimated Time:** 20 minutes

---

#### Task 8.5: Styling and polish
- [x] Apply consistent spacing
- [x] Use Material-UI Paper for sections
- [x] Add section titles
- [x] Improve typography
- [x] Test responsive layout
- [x] Add app header/title

**Estimated Time:** 30 minutes

---

#### Task 8.6: Final testing
- [x] Test with backend server running
- [x] Test asset selection
- [x] Test all presets
- [x] Test pull-mode queries
- [x] Test replay sessions
- [x] Test error scenarios
- [x] Test copy URL functionality
- [x] Verify performance

**Estimated Time:** 40 minutes

---

## Summary

### Task Group Completion Order

```
Task Group 1 (Project Setup)
    ↓
Task Group 2 (Types & API)
    ↓
Task Group 3 (AssetSelector)
    ↓
Task Group 4 (AnalyticsPresets)
    ↓
Task Group 5 (ApiUrlDisplay & Chart)
    ↓
Task Group 6 (ReplayControls)
    ↓
Task Group 7 (SSE Integration)
    ↓
Task Group 8 (Integration & Polish)
```

### Dependencies

- **External:** Node.js 18+, npm/yarn
- **Backend:** Analytics API server running on localhost:3000
- **Internal:** All backend components complete (Items 1-9) ✅

### Test Coverage

- **Component Testing:** Manual testing of each component
- **Integration Testing:** End-to-end workflows (pull-mode, replay)
- **Error Testing:** Network failures, invalid inputs
- **Performance Testing:** Large datasets, rapid updates

### Time Estimates

- **Task Group 1:** 50 minutes
- **Task Group 2:** 65 minutes
- **Task Group 3:** 75 minutes
- **Task Group 4:** 45 minutes
- **Task Group 5:** 90 minutes
- **Task Group 6:** 70 minutes
- **Task Group 7:** 105 minutes
- **Task Group 8:** 185 minutes

**Total:** ~11 hours

### Success Criteria

- ✅ React app runs on localhost:5173
- ✅ Can select multiple assets
- ✅ Quick preset buttons work
- ✅ API URL displays and copies
- ✅ Chart displays pull-mode data
- ✅ Can start replay and see real-time updates
- ✅ Progress bar updates during replay
- ✅ Can stop replay
- ✅ Error messages display correctly
- ✅ Professional, clean UI

