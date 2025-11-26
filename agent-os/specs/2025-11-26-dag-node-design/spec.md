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
1. Stateless analytic primitives (average, log return, std dev, EMA) should be separated from window logic.
2. Windowing traits provide necessary inputs (fixed lag windows, exponential smoothing, previous result + new point).
3. DAG nodes must construct themselves from analytic metadata, automatically extending date ranges for burn-in.
4. Push-mode callbacks and replay should reuse the same metadata as pull-mode queries to avoid mismatched node IDs.
5. Override scenarios (e.g., arithmetic returns) must generate distinct keys, not mutate existing node definitions.
6. The new keying approach may leverage salsa-style interning to dedupe nodes and improve caching.
7. Document the registry/DSL of analytics so future functions can declare dependencies without manual wiring.

## Visual Assets
No visuals were supplied.
