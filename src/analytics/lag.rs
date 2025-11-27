/// Descriptor for a lag analytic in the registry.
pub trait LagAnalytic {
    /// Lag distance (e.g., 5 for a 5-day lag).
    fn lag(&self) -> usize;

    /// Required number of points (`lag + 1`).
    fn required_points(&self) -> usize {
        self.lag() + 1
    }

    /// Returns the value at index `lag` from the provided slice.
    fn compute_lagged(&self, values: &[f64]) -> Option<f64> {
        values.get(self.lag()).copied()
    }
}

/// Simple fixed lag analytic that uses a static lag distance.
pub struct FixedLag {
    lag: usize,
}

impl FixedLag {
    pub fn new(lag: usize) -> Self {
        FixedLag { lag }
    }
}

impl LagAnalytic for FixedLag {
    fn lag(&self) -> usize {
        self.lag
    }
}
