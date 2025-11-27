# DAG Node Design Tasks

## Task Group 1 – NodeKey & Metadata
- [x] Define a `NodeKey`/`NodeMetadata` struct that captures analytic primitive, input asset, date-range, window parameters, and override flags.
- [x] Ensure keys can be interned (e.g., hash + dedupe, Salsa-style) so identical analytics reuse the same node.
- [x] Add serialization/Display helpers for tooling and logging clarity.

## Task Group 2 – Stateless analytic primitives
- [x] Move core analytic functions (average, std deviation, log return, ratio, EMA) into stateless helpers with clear input/output contracts.
- [x] Keep analytic math agnostic of data supply; only consume numeric slices and emit scalar output or new time-series points.

## Task Group 3 – Windowing/aggregation wrappers
- [x] Implement a windowing abstraction (trait) that feeds time-series slices or previous value + new point to analytic primitives.
- [x] Cover fixed-lag windows, rolling windows, and EMA-style previous-value windows.
- [x] Make the windowing trait pluggable so new analytics (with same inputs) can reuse the implementation.
- [ ] Define `LagAnalytic` descriptor trait + ensure window layer can supply `lag +1` slices.

## Task Group 4 – Self-constructing DAG definitions
- [x] Create a registry/DSL where each analytic definition declares dependencies (e.g., volatility → returns → prices).
- [x] Use NodeKey metadata to derive the DAG edges automatically when a user requests an analytic.
- [x] Ensure window lag metadata automatically adjusts the consumed date range to include burn-in data.

## Task Group 5 – Push-mode & pull-mode alignment
- [x] Update both execution paths to resolve nodes via NodeKey metadata rather than manual string identifiers.
- [x] Make sure push-mode callbacks and pull-mode queries share the same keys so caching and replay reuse definitions.
- [x] Handle override keys explicitly, so alternate inputs (arithmetic returns, custom windows) produce unique NodeKeys.

## Task Group 7 – Lag & Calendar dependencies
- [ ] Introduce a LagAnalytic descriptor + window interface so lag nodes declare “I need the last `lag + 1` points”.
- [ ] Add calendar metadata derived from the data provider so analytics know which timestamps exist and how much burn-in to request (e.g., 20-day volatility with 5-day returns needs prices from `T - 25`).
- [ ] Wire the pull-mode DAG construction to simulate push updates using the calendar sequence, so push/pull share the same execution path for overlapping windows.

## Task Group 6 – Documentation & examples
- [x] Document the registry/DSL for declaring new analytics, including how to declare dependencies and overrides.
- [x] Provide example specs showing how requesting `rolling volatility` infers its dependencies and burn-in.
- [x] Outline how to add new primitives + windowing logic without touching the DAG wiring.
