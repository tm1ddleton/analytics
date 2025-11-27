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
