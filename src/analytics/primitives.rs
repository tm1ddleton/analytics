//! Stateless analytic primitives used by the DAG execution helpers.
//!
//! These are pure functions that only operate on slices of numeric data and
//! can be composed with windowing strategies.

/// Calculates the log return between the first and last values in the window.
///
/// Returns `f64::NAN` when there are fewer than two values available. If any
/// price is invalid (non-positive or NaN) the function mirrors the legacy
/// behavior by returning `0.0`.
pub fn log_return_window(window: &[f64]) -> f64 {
    if window.len() < 2 {
        return f64::NAN;
    }

    let first = window[0];
    let last = window[window.len() - 1];

    if first <= 0.0 || last <= 0.0 || first.is_nan() || last.is_nan() {
        return 0.0;
    }

    let log_return = (last / first).ln();
    if log_return.is_nan() {
        0.0
    } else {
        log_return
    }
}

/// Calculates the population standard deviation of available (non-NaN) values.
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

#[cfg(test)]
mod tests {
    use super::*;

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
