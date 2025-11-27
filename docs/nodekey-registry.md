# NodeKey + Registry Guide

This documents how the registry-driven execution pipeline works after Stage 5:

## 1. NodeKey metadata

- `NodeKey` (see `src/dag/types.rs`) uniquely describes an analytic node by combining:
  - `AnalyticType` (returns, volatility, etc.)
  - `assets` involved
  - `range` + optional `WindowSpec`
  - `params` map (window size, override tag, etc.)
  - `override_tag` – optional string that produces a separate cache entry when you intentionally want a different dependency graph (arithmetic returns, custom data overrides).
- Each `NodeKey` is hashed and interned by `AnalyticsDag`, so identical analytics reuse the same node.

## 2. Registry-driven DAG construction

- The registry lives in `src/analytics/registry.rs`. Each `AnalyticDefinition` exposes:
  - `dependencies(&NodeKey)` – returns the `NodeKey`s of required parents (data provider → returns → volatility).
  - `executor()` – provides stateless pull/push implementations (`AnalyticExecutor` handles both full-series and scalar updates).
- `AnalyticsDag::resolve_node(NodeKey)` recursively asks the registry for dependencies, builds the DAG, and records the mapping from `NodeKey` → `NodeId` so future requests reuse the same node.
- Every analytic now goes through `_build_node_key` in `src/server/handlers.rs` to ensure window defaults and override tags are captured, and endpoints share the same key (pull APIs, batch API, replay).

## 3. Execution alignment

- Pull-mode (`AnalyticsDag::execute_pull_mode`) and push-mode (`PushModeEngine::execute_node`) now call `execute_pull_node` / `execute_push_node`, which both look up the registry executor via the node’s `NodeKey`. That means:
  - There is no longer a hardcoded `execute_returns_node` or `execute_volatility_node` – the registry care of the primitive execution.
  - Push-mode scalar outputs and pull-mode time-series outputs are produced by the same executor per analytic type.
  - Override tags and window specs affect both DAG wiring and execution results because they are encoded in the `NodeKey`.

## 4. Overrides and caching

- The HTTP APIs (`AnalyticsQueryParams`, `BatchQuery`, `AnalyticConfig`) accept an optional `override` property that flows into the request parameters and becomes part of the `NodeKey`.
- The `override_tag` ensures that, for example, arithmetic returns (tagged `"arith"`) build a distinct DAG/NodeKey from log returns—even if other inputs are the same.
- Because `AnalyticsDag` caches nodes by `NodeKey`, any request with the same analytic/window/override will reuse the previous node and execution pipeline, speeding up repeat queries and keeping push/pull consistency.

## 5. Notes for authors

- To add a new analytic, register it in `AnalyticRegistry` with:
  1. A definition that declares its dependencies based on the incoming `NodeKey`.
  2. An executor implementing both `execute_pull` and `execute_push`.
  3. Optional override logic by looking at `NodeKey.override_tag` or entries in `NodeKey.params`.
- If you need to inspect or mutate a built DAG manually, `AnalyticsDag::register_node_key` allows retrofitting a `NodeKey` onto a node created outside the registry so execution still works.


