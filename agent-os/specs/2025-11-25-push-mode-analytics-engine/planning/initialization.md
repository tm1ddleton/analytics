# Feature Initialization: Push-Mode Analytics Engine

## Initial Description

Push-Mode Analytics Engine — Implement incremental computation system where analytics update automatically when new data arrives, with dependency propagation through the DAG.

## Roadmap Context

This is **Item 5** from the product roadmap (POC Phase).

**Size:** XL (Extra Large)

**Dependencies:**
- ✅ Item 1: Core Asset Data Model (complete)
- ✅ Item 2: SQLite Data Storage (complete)
- ✅ Item 3: Yahoo Finance Data Downloader (complete)
- ✅ Item 4: DAG Computation Framework (complete)
- 🟡 Item 6: Basic Analytics Library (partially complete - returns & volatility done)

**Enables:**
- Item 7: High-Speed Data Replay System
- Item 8: REST API Server with WebSocket/SSE
- Item 9: React UI Dashboard
- Item 15: Real-Time Data Ingestion

## Core Concept

When a new data point arrives (e.g., a new price for AAPL):
1. System identifies which analytics depend on this data
2. Recomputes only the affected analytics (not full history)
3. Propagates updates through the DAG dependency chain
4. Notifies subscribers of changed values

Example: New price arrives → Returns recalculated (only new point) → Volatility updated (rolling window) → Dependent strategies notified

## Key Questions to Address

1. How do we track which data points have been computed vs. need recomputation?
2. How do we handle rolling windows efficiently in incremental mode?
3. What's the API for receiving new data points?
4. How do we propagate updates through the DAG?
5. How do we handle multiple data points arriving in rapid succession?
6. What's the subscription/notification mechanism for analytics updates?

---

**Date Created:** 2025-11-25
**Status:** Initialization Phase

