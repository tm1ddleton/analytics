use crate::asset_key::AssetKey;

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

pub fn population_std_dev(values: &[f64]) -> f64 {
    let valid_values: Vec<f64> = values.iter().copied().filter(|v| !v.is_nan()).collect();
    if valid_values.is_empty() {
        return f64::NAN;
    }

    let n = valid_values.len() as f64;
    let mean = valid_values.iter().sum::<f64>() / n;
    let variance = valid_values
        .iter()
        .map(|&value| (value - mean).powi(2))
        .sum::<f64>()
        / n;
    variance.sqrt()
}

pub fn ema_step(previous: Option<f64>, value: f64, lambda: f64) -> f64 {
    match previous {
        Some(prev) => lambda * value + (1.0 - lambda) * prev,
        None => value,
    }
}

pub fn log_return_window(window: &[f64]) -> f64 {
    if window.len() < 2 {
        return f64::NAN;
    }
    log_return_value(window[window.len() - 1], window[0])
}

/// Stateless analytic for returns.
pub trait ReturnAnalytic: Send + Sync {
    fn name(&self) -> &'static str;
    fn compute(&self, asset: Option<&AssetKey>, current: f64, lagged: f64) -> f64;
}

pub struct LogReturnAnalytic;

impl ReturnAnalytic for LogReturnAnalytic {
    fn name(&self) -> &'static str {
        "log_return"
    }

    fn compute(&self, _asset: Option<&AssetKey>, current: f64, lagged: f64) -> f64 {
        log_return_value(current, lagged)
    }
}

pub struct ArithReturnAnalytic;

impl ReturnAnalytic for ArithReturnAnalytic {
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

/// Stateless analytic for windowed volatility calculations.
pub trait VolatilityAnalytic: Send + Sync {
    fn name(&self) -> &'static str;
    fn compute(&self, asset: Option<&AssetKey>, window: &[f64]) -> f64;
}

pub struct StdDevVolatilityAnalytic;

impl VolatilityAnalytic for StdDevVolatilityAnalytic {
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
        let analytic = LogReturnAnalytic;
        let expected = (105.0_f64 / 100.0_f64).ln();
        assert_eq!(analytic.compute(None, 105.0, 100.0), expected);
        assert_eq!(analytic.compute(None, -1.0, 100.0), 0.0);
    }

    #[test]
    fn arith_return_handles_zero_lag() {
        let analytic = ArithReturnAnalytic;
        let result = analytic.compute(None, 105.0, 100.0);
        assert!((result - 0.05).abs() < 1e-12);
        assert_eq!(analytic.compute(None, 105.0, 0.0), 0.0);
    }

    #[test]
    fn stddev_handles_nan() {
        let analytic = StdDevVolatilityAnalytic;
        let data = vec![1.0, 2.0, f64::NAN];
        assert!(analytic.compute(None, &data).is_finite());
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
