//! Windowing strategies that supply slices (or state) to primitives.
//!
//! These structs implement `WindowStrategy` so the DAG can reason about burn-in
//! requirements without knowing the specific analytic primitive.

/// Common behavior shared by every windowing strategy.
pub trait WindowStrategy {
    /// The number of data points required before meaningful output can be produced.
    fn burn_in(&self) -> usize;
}

/// Fixed-lag sliding window (e.g., rolling std dev, returns).
#[derive(Debug, Clone, Copy)]
pub struct FixedWindow {
    size: usize,
}

impl FixedWindow {
    /// Creates a fixed window that always includes `size` most recent values
    /// (or fewer when the prefix is shorter than `size`).
    pub fn new(size: usize) -> Self {
        FixedWindow { size: size.max(1) }
    }

    /// Applies the given primitive to every prefix window ending at each index.
    pub fn apply<F>(&self, data: &[f64], mut primitive: F) -> Vec<f64>
    where
        F: FnMut(&[f64]) -> f64,
    {
        if data.is_empty() {
            return Vec::new();
        }

        let window_size = self.size;
        let mut result = Vec::with_capacity(data.len());

        for (index, _) in data.iter().enumerate() {
            let start = if index + 1 < window_size {
                0
            } else {
                index + 1 - window_size
            };
            let window = &data[start..=index];
            result.push(primitive(window));
        }

        result
    }
}

impl WindowStrategy for FixedWindow {
    fn burn_in(&self) -> usize {
        self.size
    }
}

/// Exponential smoothing window that depends on the previous output.
#[derive(Debug, Clone, Copy)]
pub struct ExponentialWindow {
    lambda: f64,
    lookback: usize,
}

impl ExponentialWindow {
    pub fn new(lambda: f64, lookback: usize) -> Self {
        let constrained = lambda.clamp(0.0, 1.0);
        ExponentialWindow {
            lambda: constrained,
            lookback,
        }
    }

    /// Applies a stateful primitive that consumes the previous output along
    /// with the new value.
    pub fn apply<F>(&self, data: &[f64], mut primitive: F) -> Vec<f64>
    where
        F: FnMut(Option<f64>, f64) -> f64,
    {
        if data.is_empty() || !(0.0 < self.lambda && self.lambda <= 1.0) {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(data.len());
        let mut previous: Option<f64> = None;

        for &value in data.iter() {
            let next = primitive(previous, value);
            result.push(next);
            previous = Some(next);
        }

        result
    }
}

impl WindowStrategy for ExponentialWindow {
    fn burn_in(&self) -> usize {
        self.lookback
    }
}

#[cfg(test)]
mod tests {
    use super::super::primitives::ema_step;
    use super::ExponentialWindow;
    use super::FixedWindow;
    use super::WindowStrategy;

    #[test]
    fn fixed_window_applies_primitive_to_subarrays() {
        let window = FixedWindow::new(3);
        let data = vec![1.0, 2.0, 3.0, 4.0];
        let result = window.apply(&data, |slice| slice.iter().sum());
        assert_eq!(result, vec![1.0, 3.0, 6.0, 9.0]);
        assert_eq!(window.burn_in(), 3);
    }

    #[test]
    fn exponential_window_produces_ema_chain() {
        let window = ExponentialWindow::new(0.5, 5);
        let data = vec![10.0, 20.0, 40.0];
        let result = window.apply(&data, |prev, value| ema_step(*prev, value, 0.5));
        assert_eq!(result.len(), data.len());
        assert!((result[0] - 10.0).abs() < 1e-12);
        assert!(window.burn_in() == 5);
    }
}
