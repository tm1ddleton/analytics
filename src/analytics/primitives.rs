//! Stateless analytic primitives used by the DAG execution helpers.
//!
//! These are pure, composable primitives described as traits. The registry
//! dispatches the correct primitive implementation for each `AnalyticType`.

use crate::asset_key::AssetKey;

/// Helper: returns log ratio between two prices same as previous `log_return_window`.
fn log_return_value(current: f64, lagged: f64) -> f64 {
    if lagged <= 0.0 || current <= 0.0 || lagged.is_nan() || current.is_nan() {
        return 0.0;
    }

    let value = (current / lagged).ln();
    if value.is_nan() {
        0.0
    } else {
        value
    }
}

/// Calculates the population standard deviation of the provided values.
pub fn population_std_dev(values: &[f64]) -> f64 {
    let valid_values: Vec<f64> = values.iter().copied().filter(|v| !v.is_nan()).collect();

    if valid_values.is_empty() {
        return f64::NAN;
    }

    let n = valid_values.len() as f64;
    let mean = valid_values.iter().sum::<f64>() / n;
    let sum_squared_diff: f64 = valid_values
        .iter()
        .map(|&value| (value - mean).powi(2))
        .sum();

    (sum_squared_diff / n).sqrt()
}

/// Computes the next value for an exponential smoothing primitive.
pub fn ema_step(previous: Option<f64>, value: f64, lambda: f64) -> f64 {
    match previous {
        Some(prev) => lambda * value + (1.0 - lambda) * prev,
        None => value,
    }
}

/// Public helper that mimics the earlier `log_return_window`.
pub fn log_return_window(window: &[f64]) -> f64 {
    if window.len() < 2 {
        return f64::NAN;
    }

    log_return_value(window[window.len() - 1], window[0])
}

/// Primitive trait that computes returns given current and lagged prices.
pub trait ReturnPrimitive: Send + Sync {
    /// Name used for logging/diagnostics.
    fn name(&self) -> &'static str;

    /// Computes a return from the supplied pair.
    fn compute(&self, asset: Option<&AssetKey>, current: f64, lagged: f64) -> f64;
}

/// Log return primitive that mirrors legacy behavior.
pub struct LogReturnPrimitive;

impl ReturnPrimitive for LogReturnPrimitive {
    fn name(&self) -> &'static str {
        "log_return"
    }

    fn compute(&self, _asset: Option<&AssetKey>, current: f64, lagged: f64) -> f64 {
        log_return_value(current, lagged)
    }
}

/// Arithmetic return primitive.
pub struct ArithReturnPrimitive;

impl ReturnPrimitive for ArithReturnPrimitive {
    fn name(&self) -> &'static str {
        "arith_return"
    }

    fn compute(&self, _asset: Option<&AssetKey>, current: f64, lagged: f64) -> f64 {
        if lagged == 0.0 || lagged.is_nan() || current.is_nan() {
            return 0.0;
        }

        current / lagged - 1.0
    }
}

/// Primitive trait for volatility calculations.
pub trait VolatilityPrimitive: Send + Sync {
    /// Name for the primitive.
    fn name(&self) -> &'static str;

    /// Computes volatility over a window of return values.
    fn compute(&self, asset: Option<&AssetKey>, window: &[f64]) -> f64;
}

/// Standard deviation primitive.
pub struct StdDevVolatilityPrimitive;

impl VolatilityPrimitive for StdDevVolatilityPrimitive {
    fn name(&self) -> &'static str {
        "population_std_dev"
    }

    fn compute(&self, _asset: Option<&AssetKey>, window: &[f64]) -> f64 {
        population_std_dev(window)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_return_has_nan_guardrails() {
        let primitive = LogReturnPrimitive;
        let expected = (105.0_f64 / 100.0_f64).ln();
        assert_eq!(primitive.compute(None, 105.0, 100.0), expected);
        assert_eq!(primitive.compute(None, -1.0, 100.0), 0.0);
    }

    #[test]
    fn arith_return_handles_zero_lag() {
        let primitive = ArithReturnPrimitive;
        let result = primitive.compute(None, 105.0, 100.0);
        assert!((result - 0.05).abs() < 1e-12);
        assert_eq!(primitive.compute(None, 105.0, 0.0), 0.0);
    }

    #[test]
    fn stddev_handles_nan() {
        let primitive = StdDevVolatilityPrimitive;
        let data = vec![1.0, 2.0, f64::NAN];
        assert!((primitive.compute(None, &data).is_finite()));
    }

    #[test]
    fn log_return_window_requires_two_values() {
        assert!(log_return_window(&[]).is_nan());
        assert!(log_return_window(&[100.0]).is_nan());
    }

    #[test]
    fn log_return_window_handles_invalid_prices() {
        assert_eq!(log_return_window(&[100.0, -5.0]), 0.0);
        assert_eq!(log_return_window(&[f64::NAN, 200.0]), 0.0);
    }

    #[test]
    fn log_return_window_uses_first_and_last() {
        let result = log_return_window(&[100.0, 105.0, 110.0]);
        assert!((result - (110.0_f64 / 100.0_f64).ln()).abs() < 1e-10);
    }

    #[test]
    fn population_std_dev_ignores_nan() {
        let values = vec![1.0, 2.0, f64::NAN, 3.0];
        let result = population_std_dev(&values);
        assert!((result - 0.816496580927726).abs() < 1e-12);
    }

    #[test]
    fn population_std_dev_empty_returns_nan() {
        assert!(population_std_dev(&[]).is_nan());
    }

    #[test]
    fn ema_step_defaults_to_value_without_previous() {
        assert_eq!(ema_step(None, 42.0, 0.5), 42.0);
    }

    #[test]
    fn ema_step_weights_new_value_and_previous() {
        let first = ema_step(None, 100.0, 0.1);
        let second = ema_step(Some(first), 110.0, 0.1);
        assert!((second - (0.1 * 110.0 + 0.9 * first)).abs() < 1e-12);
    }
}
