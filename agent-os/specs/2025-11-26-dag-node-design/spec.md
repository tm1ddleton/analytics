# DAG Node Design Refactor

## Objective
Refactor the DAG node model so analytics are expressed as composable, stateless primitives wrapped by windowing/aggregation helpers, while the underlying keying matches Salsa-style interning so push-mode and pull-mode execution paths reference the same node identity metadata.

## Background
- Analytics today are coded directly within DAG nodes and require manual wiring (DataProvider → Returns → Volatility).
- Node types are referenced by strings, leading to mismatches and duplicated logic between pull-mode and push-mode engines.
- Rolling analytics depend on windowed input (returns/volatility) yet their dependencies are manually assembled.

## Scope
- Introduce a `NodeKey` structure that encodes:
  * The analytic primitive (average / log returns / volatility / standard deviation / exponential smoothing, etc.)
  * Input metadata (asset, date range, window parameters, override flags)
  * Optional overrides (e.g., arithmetic returns vs default log returns) yields different keys.
- Keep analytics stateless; windowing/aggregation (moving average, volatility roll) are wrappers providing slices or previous values to the primitive functions.
- Ensure the push-mode and pull-mode engines compute the same nodes by deriving their DAG dependencies from the `NodeKey` metadata rather than hardcoding edges.
- Provide a registry or DSL of analytic definitions so requesting `rolling volatility` automatically infers the required dependency chain and burn-in.
- Maintain API-level ability to override inferred inputs via distinct keys while defaulting to sensible dependencies.

## Requirements
1. Stateless analytic primitives (returns, volatility, std dev, EMA) are separate from window logic; the registry dispatches the right primitive via metadata/override tags.
2. Lag/windowing nodes merely describe the slice they require (e.g., last `lag + 1` points) and remain stateless; buffering/caching happens at the window layer.
3. DAG nodes self-construct by reading `NodeKey` metadata and declaring what dependencies (data provider, lag, return, calendar, etc.) plus how much history they need (burn-in).
4. Push-mode callbacks, replay, and pull-mode queries share the same `NodeKey` definitions so caching/replay reuse identical nodes.
5. Override scenarios (arithmetic returns, custom volatility lookbacks) produce distinct keys rather than mutating existing ones; NodeKey encodes analytic type, lag/window, override tag, dates, and assets.
6. Introduce a calendar node sourced from the data provider that describes the ordered timestamps/working days; analytics use that to extend price queries (e.g., 20-day volatility with overlapping 5-day returns must fetch prices from `T - 25`).
7. Provide interface traits (`ReturnPrimitive`, `LagAnalytic`, `Windowing layer`, `ReturnExecutor`, `WindowedAnalytic`, `VolatilityExecutor`) so new analytics plug into the registry without manual wiring.
8. Document the registry/DSL so future primitives can declare dependencies and required history automatically.

## Visual Assets
No visuals were supplied.
