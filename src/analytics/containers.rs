use crate::asset_key::AssetKey;
use crate::analytics::calculators::{log_return_value, population_std_dev};

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
}

