# DAG Node Design Tasks

## Task Group 1 – NodeKey & Metadata
- Define a `NodeKey`/`NodeMetadata` struct that captures analytic primitive, input asset, date-range, window parameters, and override flags.
- Ensure keys can be interned (e.g., hash + dedupe, Salsa-style) so identical analytics reuse the same node.
- Add serialization/Display helpers for tooling and logging clarity.

## Task Group 2 – Stateless analytic primitives
- Move core analytic functions (average, std deviation, log return, ratio, EMA) into stateless helpers with clear input/output contracts.
- Keep analytic math agnostic of data supply; only consume numeric slices and emit scalar output or new time-series points.

## Task Group 3 – Windowing/aggregation wrappers
- Implement a windowing abstraction (trait) that feeds time-series slices or previous value + new point to analytic primitives.
- Cover fixed-lag windows, rolling windows, and EMA-style previous-value windows.
- Make the windowing trait pluggable so new analytics (with same inputs) can reuse the implementation.

## Task Group 4 – Self-constructing DAG definitions
- Create a registry/DSL where each analytic definition declares dependencies (e.g., volatility → returns → prices).
- Use NodeKey metadata to derive the DAG edges automatically when a user requests an analytic.
- Ensure window lag metadata automatically adjusts the consumed date range to include burn-in data.

## Task Group 5 – Push-mode & pull-mode alignment
- Update both execution paths to resolve nodes via NodeKey metadata rather than manual string identifiers.
- Make sure push-mode callbacks and pull-mode queries share the same keys so caching and replay reuse definitions.
- Handle override keys explicitly, so alternate inputs (arithmetic returns, custom windows) produce unique NodeKeys.

## Task Group 6 – Documentation & examples
- Document the registry/DSL for declaring new analytics, including how to declare dependencies and overrides.
- Provide example specs showing how requesting `rolling volatility` infers its dependencies and burn-in.
- Outline how to add new primitives + windowing logic without touching the DAG wiring.
