# Volatility Design Notes

This document mirrors the returns design but focuses on volatility as a windowed analytic over returns.

## 1. Stateless primitive

- Volatility is typically calculated as the standard deviation of a set of returns. We expose a stateless primitive:

```rust
fn population_std_dev(values: &[f64]) -> f64 {
    let n = values.len();
    if n == 0 {
        return f64::NAN;
    }

    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = values
        .iter()
        .map(|&v| (v - mean).powi(2))
        .sum::<f64>()
        / n as f64;
    variance.sqrt()
}
```

- The primitive receives the slice of returns already trimmed to the desired window, so it stays agnostic to buffering and caching.

## 2. Window + lag nodes

- Volatility relies on a rolling window of returns (e.g., the last `N` returns). Instead of hard-coding the window, the analytic declares its `window_size` and asks for that slice of returns.
- A separate **lag node** can provide the `lagged` returns if needed by other analytics.
- The DAG dependencies are `DataProvider → Returns → Volatility`, where the volatility node consumes the return time series slice (handled by the windowing layer) and feeds it to `population_std_dev`.
- Because the volatility analytic declaratively states the window size, the registry/caching layer can prefetch/buffer the required slice ahead of execution.

## 3. Registry implications

- `NodeKey` must include `window_size` and any override tags to keep caches separate for different lookbacks (e.g., 10 vs 20 days).
- The volatility executor pulls the appropriate slice from cache and hands it to `population_std_dev`.
- Push and pull share the same metadata, so incremental updates and batch queries stay consistent.

## 4. Reusable components

1. **Window declarator** – encodes the required `window_size` and supplies the return slice.
2. **Volatility primitive** – `population_std_dev` takes the slice and returns the latest volatility value.
3. **NodeKey metadata** – includes window size, overrides, and analytic type so caching aligns across push/pull.

## 5. Interface sketch

```rust
/// Trait for analytics that require a fixed window of returns.
pub trait WindowedAnalytic {
    fn window_size(&self) -> usize;
}

/// Volatility executor that simply computes std dev over the provided window.
pub trait VolatilityExecutor {
    fn execute(&self, window: &[f64]) -> f64;
}

pub struct DefaultVolatilityExecutor;
impl VolatilityExecutor for DefaultVolatilityExecutor {
    fn execute(&self, window: &[f64]) -> f64 {
        population_std_dev(window)
    }
}
```

The windowing layer feeds the stored return slice to `VolatilityExecutor`, which calls `population_std_dev`. The window size lives in the NodeKey so different lookbacks produce distinct nodes.

