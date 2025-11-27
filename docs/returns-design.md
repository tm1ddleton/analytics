# Returns-First Design Notes

This document explores how returns can be composed from lightweight, reusable components in the registry/DAG.

## 1. Stateless primitives

- Returns compare the current value to the value `lag` steps ago. We provide primitives such as:

```rust
fn log_return(current: f64, lagged: f64) -> f64 {
    (current / lagged).ln()
}

fn arith_return(current: f64, lagged: f64) -> f64 {
    current / lagged - 1.0
}
```

- These share the same signature, so the registry can swap them via the `override_tag` (e.g., `"arith"`) without touching windowing.
- The primitive is agnostic to DAG structure; windowing supplies the `(current, lagged)` pair.

## 2. Lag analytic as descriptor

- A **lag analytic** knows the `lag` parameter and declares the data it needs: the vector of the latest `lag + 1` points. When invoked, it returns the entry at index `lag`. It carries no buffers and stays stateless beyond `lag`.
- The surrounding windowing/caching layer feeds that vector (the last `lag + 1` working days) so the lag analytic can return the required value immediately. Because it doesn’t manage buffers, it remains reusable across analytics.
- The **return analytic** depends on the lag analytic (lagged price) and the `DataProvider` (current price). It simply applies the primitive (`log_return` or `arith_return`) and thus avoids owning its own `WindowSpec`.
- The DAG becomes `DataProvider → Lag(N)` plus a direct edge `DataProvider → Returns`, letting the return node read both parents before computing the primitive.

## 3. Registry implications

- The `NodeKey` for returns captures `lag` and any override tag so caching/intering knows `lag=1` differs from `lag=5` (and log differs from arithmetic).
- The lag node reuses window metadata by declaring it needs a `WindowSpec::fixed(lag + 1)` slice and returns the `lag`th entry when provided.
- Push and pull both resolve the same `NodeKey`, so they share cached nodes even when primitives change.

## 4. Reusable components

1. **Lag analytic descriptor** – stateless aside from `lag`; takes the provided vector and yields the `lag`th element.
2. **Return primitive** – `log_return`/`arith_return` share a signature so overrides can swap them via metadata.
3. **Windowing/caching layer** – caches the slice of recent points and feeds it to the lag node; it also supplies the current value to the return node.
4. **NodeKey metadata** – encodes lag, override tags, and analytic type so push/pull share the same DAG.

## 6. Interface sketch

Below is an interface outline that follows the above architecture. A stateless lag analytic simply declares `lag` and its data requirements, the window layer supplies the cached slice, and the return executor consumes the current price plus the lagged value through a primitive that can be swapped via override tags:

```rust
use chrono::{DateTime, Utc};

/// Stateless return analytic plugged into the registry executor.
pub trait ReturnAnalytic: Send + Sync {
    fn compute(&self, current: f64, lagged: f64) -> f64;
}

pub struct LogReturn;
impl ReturnAnalytic for LogReturn {
    fn compute(&self, current: f64, lagged: f64) -> f64 {
        (current / lagged).ln()
    }
}

pub struct ArithReturn;
impl ReturnAnalytic for ArithReturn {
    fn compute(&self, current: f64, lagged: f64) -> f64 {
        current / lagged - 1.0
    }
}

/// Descriptor for a lag analytic: no buffers, just `lag` and the selector logic.
pub trait LagAnalytic {
    fn lag(&self) -> usize;

    fn compute_lagged(&self, values: &[f64]) -> Option<f64> {
        values.get(self.lag()).copied()
    }
}

/// Window layer drives cached slices into the lag analytic.
pub trait LagWindow {
    fn push(&mut self, timestamp: DateTime<Utc>, price: f64);
    fn ready(&self) -> bool;
    fn slice(&self) -> &[f64];
}

/// Return executor applies the primitive to the current and lagged values.
pub trait ReturnExecutor {
    fn execute(
        &self,
        current_price: f64,
        lagged_price: f64,
        primitive: &dyn ReturnAnalytic,
    ) -> f64;
}

pub struct DefaultReturnExecutor;
impl ReturnExecutor for DefaultReturnExecutor {
    fn execute(
        &self,
        current_price: f64,
        lagged_price: f64,
        primitive: &dyn ReturnAnalytic,
    ) -> f64 {
        primitive.compute(current_price, lagged_price)
    }
}
```

The registry ties these pieces together: the lag node indicates which slice it needs, the windowing layer caches the inputs and feeds both parents, and the return executor runs the primitive, leaving the rest of the DAG ignorant of data-fetch strategies.

## 5. Pandas analogy

```
lagged = df.shift(1)
returns = (df / lagged).apply(np.log)
```

The DAG equivalent is `Lag` plus `Returns`: lag provides the shifted value, returns applies the primitive, and the registry keeps the wiring consistent across execution modes.

