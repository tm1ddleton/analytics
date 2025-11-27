#![allow(dead_code)]

use crate::analytics::calculators::{
    LogReturnAnalytic, ReturnAnalytic, StdDevVolatilityAnalytic, VolatilityAnalytic,
};
use crate::time_series::TimeSeriesPoint;

/// Test helper: computes log returns across adjacent prices.
pub(crate) fn calculate_returns(prices: &[f64]) -> Vec<f64> {
    let primitive = LogReturnAnalytic;
    if prices.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(prices.len());
    result.push(f64::NAN);

    for window in prices.windows(2) {
        let value = primitive.compute(None, window[1], window[0]);
        result.push(value);
    }

    result
}

/// Test helper: rolling volatility using StdDev primitive.
pub(crate) fn calculate_volatility(returns: &[f64], window_size: usize) -> Vec<f64> {
    if window_size == 0 || returns.is_empty() {
        return Vec::new();
    }

    let primitive = StdDevVolatilityAnalytic;
    returns
        .iter()
        .enumerate()
        .map(|(idx, _)| {
            let start = idx.saturating_sub(window_size - 1);
            primitive.compute(None, &returns[start..=idx])
        })
        .collect()
}

/// Test helper: single-step returns update.
pub(crate) fn calculate_returns_update(prices: &[TimeSeriesPoint]) -> f64 {
    if prices.len() < 2 {
        return f64::NAN;
    }

    let primitive = LogReturnAnalytic;
    let current = prices.last().unwrap().close_price;
    let lagged = prices[prices.len() - 2].close_price;
    primitive.compute(None, current, lagged)
}

/// Test helper: rolling volatility update.
pub(crate) fn calculate_volatility_update(returns: &[TimeSeriesPoint], window_size: usize) -> f64 {
    if returns.is_empty() || window_size == 0 {
        return f64::NAN;
    }

    let primitive = StdDevVolatilityAnalytic;
    let closes: Vec<f64> = returns.iter().map(|point| point.close_price).collect();
    let start = closes.len().saturating_sub(window_size);
    primitive.compute(None, &closes[start..])
}
