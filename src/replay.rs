//! High-Speed Data Replay System
//!
//! This module implements a replay engine that feeds historical market data
//! into the push-mode analytics engine at configurable speeds for backtesting
//! and visualization.
//!
//! # Future Roadmap Integration
//!
//! ## Item 8: WebSocket Streaming Support
//! The replay system will support streaming replay data over WebSocket connections,
//! allowing remote clients to receive simulated live market data.
//!
//! ## Item 9: UI Controls (pause/resume)
//! The replay system will support pause/resume controls via a `ReplayHandle`,
//! enabling interactive debugging and visualization workflows.

use crate::asset_key::AssetKey;
use crate::time_series::{DataProvider, DataProviderError, DateRange, TimeSeriesPoint};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Duration;

/// Errors that can occur during replay
#[derive(Debug, Clone, PartialEq)]
pub enum ReplayError {
    /// Failed to load data from provider
    DataLoadFailed(String),
    /// No data available for specified assets/range
    NoDataFound,
    /// Invalid date range specification
    InvalidDateRange,
    /// Error in user callback
    CallbackError(String),
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplayError::DataLoadFailed(msg) => write!(f, "Data load failed: {}", msg),
            ReplayError::NoDataFound => write!(f, "No data found for specified assets/range"),
            ReplayError::InvalidDateRange => write!(f, "Invalid date range"),
            ReplayError::CallbackError(msg) => write!(f, "Callback error: {}", msg),
        }
    }
}

impl std::error::Error for ReplayError {}

impl From<DataProviderError> for ReplayError {
    fn from(err: DataProviderError) -> Self {
        ReplayError::DataLoadFailed(err.to_string())
    }
}

/// Summary of replay execution
///
/// Contains statistics about the replay run including success/failure counts
/// and timing information.
#[derive(Debug, Clone)]
pub struct ReplayResult {
    /// Total data points attempted
    pub total_points: usize,
    /// Successfully replayed data points
    pub successful: usize,
    /// Failed data points
    pub failed: usize,
    /// When replay started (wall-clock time)
    pub start_time: DateTime<Utc>,
    /// When replay finished (wall-clock time)
    pub end_time: DateTime<Utc>,
    /// Wall-clock elapsed time
    pub elapsed: Duration,
    /// First data point timestamp (simulated time)
    pub simulated_start: DateTime<Utc>,
    /// Last data point timestamp (simulated time)
    pub simulated_end: DateTime<Utc>,
}

impl std::fmt::Display for ReplayResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Replay complete: {} points ({} successful, {} failed)",
            self.total_points, self.successful, self.failed
        )?;
        writeln!(
            f,
            "Simulated: {} to {}",
            self.simulated_start.format("%Y-%m-%d"),
            self.simulated_end.format("%Y-%m-%d")
        )?;
        write!(f, "Elapsed: {:.2}s", self.elapsed.as_secs_f64())
    }
}

/// High-speed data replay engine
///
/// Reads historical market data from a DataProvider and feeds it into
/// the push-mode analytics engine at configurable speeds.
///
/// # Example
///
/// ```rust,no_run
/// use analytics::{ReplayEngine, SqliteDataProvider, DateRange, AssetKey};
/// use std::sync::Arc;
/// use std::time::Duration;
/// use chrono::NaiveDate;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = Arc::new(SqliteDataProvider::new("data.db")?);
///
/// let mut replay = ReplayEngine::new(provider);
/// replay.set_delay(Duration::from_millis(100))
///       .set_progress_callback(|date| {
///           println!("Replaying: {}", date.format("%Y-%m-%d"));
///       });
///
/// let assets = vec![AssetKey::new_equity("AAPL")?];
/// let date_range = DateRange::new(
///     NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
///     NaiveDate::from_ymd_opt(2024, 3, 31).unwrap(),
/// );
///
/// let result = replay.run(assets, date_range, |asset, timestamp, value| {
///     println!("{:?} @ {}: {}", asset, timestamp, value);
///     Ok(())
/// })?;
///
/// println!("{}", result);
/// # Ok(())
/// # }
/// ```
pub struct ReplayEngine {
    /// Data source for historical data
    provider: Arc<dyn DataProvider>,
    /// Delay between data points
    delay: Duration,
    /// Optional progress callback invoked for each data point
    progress_callback: Option<Box<dyn Fn(DateTime<Utc>)>>,
    /// Optional error callback invoked when data callback fails
    error_callback: Option<Box<dyn Fn(&AssetKey, &DateTime<Utc>, &str)>>,
}

impl ReplayEngine {
    /// Creates a new ReplayEngine with default configuration
    ///
    /// Default delay is 100ms between data points.
    ///
    /// # Arguments
    /// * `provider` - Data source for historical market data
    pub fn new(provider: Arc<dyn DataProvider>) -> Self {
        ReplayEngine {
            provider,
            delay: Duration::from_millis(100), // Default 100ms delay
            progress_callback: None,
            error_callback: None,
        }
    }

    /// Sets the delay between data points
    ///
    /// # Arguments
    /// * `delay` - Duration to wait between replaying consecutive data points
    ///
    /// # Panics
    /// Panics if delay is zero
    pub fn set_delay(&mut self, delay: Duration) -> &mut Self {
        assert!(!delay.is_zero(), "Delay must be greater than zero");
        self.delay = delay;
        self
    }

    /// Sets a progress callback that is invoked for each data point
    ///
    /// The callback receives the timestamp of the data point being replayed.
    ///
    /// # Arguments
    /// * `callback` - Function called with timestamp of each replayed point
    pub fn set_progress_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(DateTime<Utc>) + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Sets an error callback that is invoked when data callback fails
    ///
    /// The callback receives the asset, timestamp, and error message.
    ///
    /// # Arguments
    /// * `callback` - Function called when replay of a data point fails
    pub fn set_error_callback<F>(&mut self, callback: F) -> &mut Self
    where
        F: Fn(&AssetKey, &DateTime<Utc>, &str) + 'static,
    {
        self.error_callback = Some(Box::new(callback));
        self
    }

    /// Loads data for a single asset
    ///
    /// # Arguments
    /// * `asset` - The asset to load data for
    /// * `date_range` - The date range to query
    ///
    /// # Returns
    /// Vector of (AssetKey, TimeSeriesPoint) tuples, or error if query fails
    fn load_asset_data(
        &self,
        asset: &AssetKey,
        date_range: &DateRange,
    ) -> Result<Vec<(AssetKey, TimeSeriesPoint)>, ReplayError> {
        let data = self.provider.get_time_series(asset, date_range)?;

        // Tag each TimeSeriesPoint with its AssetKey
        Ok(data
            .into_iter()
            .map(|point| (asset.clone(), point))
            .collect())
    }

    /// Loads and sorts data for multiple assets
    ///
    /// Queries data for all assets, merges them, and sorts chronologically.
    ///
    /// # Arguments
    /// * `assets` - Vector of assets to load
    /// * `date_range` - The date range to query
    ///
    /// # Returns
    /// Chronologically sorted vector of (AssetKey, TimeSeriesPoint) tuples
    ///
    /// # Errors
    /// Returns error if any query fails or if no data is found
    fn load_and_sort_data(
        &self,
        assets: &[AssetKey],
        date_range: &DateRange,
    ) -> Result<Vec<(AssetKey, TimeSeriesPoint)>, ReplayError> {
        let mut all_data = Vec::new();

        // Load data for each asset
        for asset in assets {
            let asset_data = self.load_asset_data(asset, date_range)?;
            all_data.extend(asset_data);
        }

        // Check if we have any data
        if all_data.is_empty() {
            return Err(ReplayError::NoDataFound);
        }

        // Sort by timestamp (chronological order)
        all_data.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));

        Ok(all_data)
    }

    /// Runs the replay simulation
    ///
    /// Loads historical data for the specified assets and date range, then
    /// replays it chronologically with configured delay between data points.
    ///
    /// The data callback is invoked for each data point. If the callback fails,
    /// the error is logged and replay continues with the next data point.
    ///
    /// # Arguments
    /// * `assets` - Vector of assets to replay
    /// * `date_range` - Date range to replay
    /// * `data_callback` - Function called for each data point with (asset, timestamp, value)
    ///
    /// # Returns
    /// Summary of replay execution with success/failure counts and timing
    ///
    /// # Errors
    /// Returns error if data loading fails or no data is found
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use analytics::{ReplayEngine, InMemoryDataProvider, DateRange, AssetKey};
    /// # use std::sync::Arc;
    /// # use chrono::NaiveDate;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let provider = Arc::new(InMemoryDataProvider::new());
    /// # let mut engine = ReplayEngine::new(provider);
    /// let assets = vec![AssetKey::new_equity("AAPL")?];
    /// let date_range = DateRange::new(
    ///     NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    ///     NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
    /// );
    ///
    /// let result = engine.run(assets, date_range, |asset, timestamp, value| {
    ///     println!("{:?} @ {}: {}", asset, timestamp, value);
    ///     Ok(())
    /// })?;
    ///
    /// println!("Replayed {} points ({} successful, {} failed)",
    ///     result.total_points, result.successful, result.failed);
    /// # Ok(())
    /// # }
    /// ```
    pub fn run<F>(
        &mut self,
        assets: Vec<AssetKey>,
        date_range: DateRange,
        mut data_callback: F,
    ) -> Result<ReplayResult, ReplayError>
    where
        F: FnMut(AssetKey, DateTime<Utc>, f64) -> Result<(), Box<dyn std::error::Error>>,
    {
        log::info!(
            "Starting replay: {} assets, date range {} to {}",
            assets.len(),
            date_range.start,
            date_range.end
        );

        // Load and sort all data
        let data = self.load_and_sort_data(&assets, &date_range)?;
        let total_points = data.len();

        log::info!("Loaded {} data points", total_points);

        // Initialize counters
        let mut successful = 0;
        let mut failed = 0;

        // Record start time
        let start_time = Utc::now();
        let simulated_start = data.first().unwrap().1.timestamp;
        let simulated_end = data.last().unwrap().1.timestamp;

        // Replay loop
        for (asset, point) in data {
            // Invoke data callback
            match data_callback(asset.clone(), point.timestamp, point.close_price) {
                Ok(()) => {
                    successful += 1;
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    log::warn!(
                        "Failed to replay {} at {}: {}",
                        asset,
                        point.timestamp,
                        error_msg
                    );

                    // Invoke error callback if set
                    if let Some(ref error_callback) = self.error_callback {
                        error_callback(&asset, &point.timestamp, &error_msg);
                    }

                    failed += 1;
                    // Continue with next data point (don't return error)
                }
            }

            // Sleep for configured delay
            std::thread::sleep(self.delay);

            // Invoke progress callback if set
            if let Some(ref progress_callback) = self.progress_callback {
                // Catch panics in progress callback to avoid crashing replay
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    progress_callback(point.timestamp);
                }));
            }
        }

        // Record end time
        let end_time = Utc::now();
        let elapsed = end_time
            .signed_duration_since(start_time)
            .to_std()
            .unwrap_or(Duration::from_secs(0));

        log::info!(
            "Replay complete: {}/{} successful",
            successful,
            total_points
        );

        // Build and return result
        Ok(ReplayResult {
            total_points,
            successful,
            failed,
            start_time,
            end_time,
            elapsed,
            simulated_start,
            simulated_end,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time_series::InMemoryDataProvider;
    use chrono::{Datelike, NaiveDate, TimeZone};

    #[test]
    fn test_new_creates_engine_with_default_delay() {
        let provider = Arc::new(InMemoryDataProvider::new());
        let engine = ReplayEngine::new(provider);

        assert_eq!(engine.delay, Duration::from_millis(100));
        assert!(engine.progress_callback.is_none());
        assert!(engine.error_callback.is_none());
    }

    #[test]
    fn test_set_delay_updates_delay() {
        let provider = Arc::new(InMemoryDataProvider::new());
        let mut engine = ReplayEngine::new(provider);

        engine.set_delay(Duration::from_millis(50));
        assert_eq!(engine.delay, Duration::from_millis(50));
    }

    #[test]
    fn test_set_delay_chains_properly() {
        let provider = Arc::new(InMemoryDataProvider::new());
        let mut engine = ReplayEngine::new(provider);

        // Test builder pattern chaining
        engine
            .set_delay(Duration::from_millis(200))
            .set_delay(Duration::from_millis(150));

        assert_eq!(engine.delay, Duration::from_millis(150));
    }

    #[test]
    #[should_panic(expected = "Delay must be greater than zero")]
    fn test_set_delay_panics_on_zero() {
        let provider = Arc::new(InMemoryDataProvider::new());
        let mut engine = ReplayEngine::new(provider);
        engine.set_delay(Duration::from_millis(0));
    }

    #[test]
    fn test_set_progress_callback_stores_callback() {
        let provider = Arc::new(InMemoryDataProvider::new());
        let mut engine = ReplayEngine::new(provider);

        engine.set_progress_callback(|_date| {
            // Test callback
        });

        assert!(engine.progress_callback.is_some());
    }

    #[test]
    fn test_configuration_with_method_chaining() {
        let provider = Arc::new(InMemoryDataProvider::new());
        let mut engine = ReplayEngine::new(provider);

        // Test full builder pattern
        engine
            .set_delay(Duration::from_millis(75))
            .set_progress_callback(|_| {})
            .set_error_callback(|_, _, _| {});

        assert_eq!(engine.delay, Duration::from_millis(75));
        assert!(engine.progress_callback.is_some());
        assert!(engine.error_callback.is_some());
    }

    #[test]
    fn test_replay_error_display() {
        let err = ReplayError::DataLoadFailed("Connection timeout".to_string());
        assert_eq!(err.to_string(), "Data load failed: Connection timeout");

        let err = ReplayError::NoDataFound;
        assert_eq!(err.to_string(), "No data found for specified assets/range");

        let err = ReplayError::InvalidDateRange;
        assert_eq!(err.to_string(), "Invalid date range");
    }

    #[test]
    fn test_replay_result_display() {
        let start = Utc::now();
        let end = start + chrono::Duration::seconds(10);

        let result = ReplayResult {
            total_points: 100,
            successful: 98,
            failed: 2,
            start_time: start,
            end_time: end,
            elapsed: Duration::from_secs(10),
            simulated_start: DateTime::from_naive_utc_and_offset(
                NaiveDate::from_ymd_opt(2024, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                Utc,
            ),
            simulated_end: DateTime::from_naive_utc_and_offset(
                NaiveDate::from_ymd_opt(2024, 3, 31)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                Utc,
            ),
        };

        let display = result.to_string();
        assert!(display.contains("100 points"));
        assert!(display.contains("98 successful"));
        assert!(display.contains("2 failed"));
        assert!(display.contains("2024-01-01"));
        assert!(display.contains("2024-03-31"));
    }

    // Task Group 2: Data Loading and Sorting Tests

    #[test]
    fn test_load_single_asset_returns_correct_data() {
        let mut provider = InMemoryDataProvider::new();
        let asset = AssetKey::new_equity("AAPL").unwrap();

        // Add test data
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
        );

        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 101.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(), 102.0),
        ];

        provider.add_data(asset.clone(), test_data.clone());

        let engine = ReplayEngine::new(Arc::new(provider));
        let result = engine.load_asset_data(&asset, &date_range).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, asset);
        assert_eq!(result[0].1.close_price, 100.0);
        assert_eq!(result[1].1.close_price, 101.0);
        assert_eq!(result[2].1.close_price, 102.0);
    }

    #[test]
    fn test_load_multiple_assets_merges_correctly() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();
        let msft = AssetKey::new_equity("MSFT").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );

        // Add AAPL data
        provider.add_data(
            aapl.clone(),
            vec![
                TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
                TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 101.0),
            ],
        );

        // Add MSFT data
        provider.add_data(
            msft.clone(),
            vec![
                TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 200.0),
                TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 201.0),
            ],
        );

        let engine = ReplayEngine::new(Arc::new(provider));
        let result = engine
            .load_and_sort_data(&[aapl, msft], &date_range)
            .unwrap();

        // Should have 4 total points (2 from each asset)
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_chronological_sorting_with_interleaved_timestamps() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();
        let msft = AssetKey::new_equity("MSFT").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );

        // Add data with interleaved timestamps
        // AAPL on day 1 and 3
        provider.add_data(
            aapl.clone(),
            vec![
                TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
                TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(), 102.0),
            ],
        );

        // MSFT on day 2
        provider.add_data(
            msft.clone(),
            vec![TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(),
                200.0,
            )],
        );

        let engine = ReplayEngine::new(Arc::new(provider));
        let result = engine
            .load_and_sort_data(&[aapl.clone(), msft.clone()], &date_range)
            .unwrap();

        // Verify sorted order: day 1 (AAPL), day 2 (MSFT), day 3 (AAPL)
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, aapl);
        assert_eq!(result[0].1.timestamp.day(), 1);
        assert_eq!(result[1].0, msft);
        assert_eq!(result[1].1.timestamp.day(), 2);
        assert_eq!(result[2].0, aapl);
        assert_eq!(result[2].1.timestamp.day(), 3);
    }

    #[test]
    fn test_empty_data_returns_error() {
        let provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        let engine = ReplayEngine::new(Arc::new(provider));
        let result = engine.load_and_sort_data(&[aapl], &date_range);

        // InMemoryDataProvider returns AssetNotFound error for non-existent assets
        // which gets mapped to DataLoadFailed
        assert!(result.is_err());
        match result {
            Err(ReplayError::DataLoadFailed(_)) | Err(ReplayError::NoDataFound) => {}
            _ => panic!("Expected error for empty data"),
        }
    }

    #[test]
    fn test_data_provider_error_maps_to_replay_error() {
        // InMemoryDataProvider returns AssetNotFound for non-existent assets
        let provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        let engine = ReplayEngine::new(Arc::new(provider));
        let result = engine.load_and_sort_data(&[aapl], &date_range);

        // Should get DataLoadFailed error
        assert!(matches!(result, Err(ReplayError::DataLoadFailed(_))));
    }

    // Task Group 3: Replay Execution Loop Tests

    #[test]
    fn test_run_calls_data_callback_for_each_point() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
        );

        // Add 5 data points
        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 101.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(), 102.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 4, 0, 0, 0).unwrap(), 103.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 5, 0, 0, 0).unwrap(), 104.0),
        ];
        provider.add_data(aapl.clone(), test_data);

        let mut engine = ReplayEngine::new(Arc::new(provider));
        engine.set_delay(Duration::from_millis(1)); // Very short delay for testing

        // Use counter to verify callback invocations
        let mut callback_count = 0;
        let result = engine
            .run(vec![aapl], date_range, |_asset, _timestamp, _value| {
                callback_count += 1;
                Ok(())
            })
            .unwrap();

        assert_eq!(callback_count, 5);
        assert_eq!(result.total_points, 5);
        assert_eq!(result.successful, 5);
        assert_eq!(result.failed, 0);
    }

    #[test]
    fn test_run_respects_delay_between_points() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );

        // Add 3 data points
        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 101.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(), 102.0),
        ];
        provider.add_data(aapl.clone(), test_data);

        let mut engine = ReplayEngine::new(Arc::new(provider));
        engine.set_delay(Duration::from_millis(50)); // 50ms delay

        let result = engine
            .run(vec![aapl], date_range, |_asset, _timestamp, _value| Ok(()))
            .unwrap();

        // With 3 points and 50ms delay, should take at least 150ms
        // Allow some variance for test timing
        assert!(result.elapsed >= Duration::from_millis(140));
    }

    #[test]
    fn test_run_returns_correct_replay_result_counts() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        // Add 10 data points
        let mut test_data = Vec::new();
        for day in 1..=10 {
            test_data.push(TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, day, 0, 0, 0).unwrap(),
                100.0 + day as f64,
            ));
        }
        provider.add_data(aapl.clone(), test_data);

        let mut engine = ReplayEngine::new(Arc::new(provider));
        engine.set_delay(Duration::from_millis(1));

        // Make callback that fails on specific values
        let result = engine
            .run(vec![aapl], date_range, |_asset, _timestamp, value| {
                if value > 105.0 {
                    Err("Value too high".into())
                } else {
                    Ok(())
                }
            })
            .unwrap();

        assert_eq!(result.total_points, 10);
        assert_eq!(result.successful, 5); // Days 1-5
        assert_eq!(result.failed, 5); // Days 6-10
    }

    #[test]
    fn test_run_with_zero_data_points_returns_no_data_found() {
        let provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        let mut engine = ReplayEngine::new(Arc::new(provider));
        let result = engine.run(vec![aapl], date_range, |_asset, _timestamp, _value| Ok(()));

        // Should get error because no data exists
        assert!(result.is_err());
    }

    #[test]
    fn test_replay_result_contains_correct_timing_info() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );

        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 101.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(), 102.0),
        ];
        provider.add_data(aapl.clone(), test_data);

        let mut engine = ReplayEngine::new(Arc::new(provider));
        engine.set_delay(Duration::from_millis(1));

        let result = engine
            .run(vec![aapl], date_range, |_asset, _timestamp, _value| Ok(()))
            .unwrap();

        // Check timing fields are populated
        assert!(result.start_time <= result.end_time);
        assert!(result.elapsed > Duration::from_millis(0));
        assert_eq!(result.simulated_start.day(), 1);
        assert_eq!(result.simulated_end.day(), 3);
    }

    // Task Group 4: Progress and Callback System Tests

    #[test]
    fn test_progress_callback_invoked_for_each_point() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );

        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 101.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(), 102.0),
        ];
        provider.add_data(aapl.clone(), test_data);

        let mut engine = ReplayEngine::new(Arc::new(provider));
        engine.set_delay(Duration::from_millis(1));

        // Track progress callback invocations
        let progress_count = Arc::new(std::sync::Mutex::new(0));
        let progress_count_clone = progress_count.clone();
        engine.set_progress_callback(move |_date| {
            *progress_count_clone.lock().unwrap() += 1;
        });

        engine
            .run(vec![aapl], date_range, |_asset, _timestamp, _value| Ok(()))
            .unwrap();

        assert_eq!(*progress_count.lock().unwrap(), 3);
    }

    #[test]
    fn test_progress_callback_receives_correct_timestamps() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );

        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 101.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(), 102.0),
        ];
        provider.add_data(aapl.clone(), test_data);

        let mut engine = ReplayEngine::new(Arc::new(provider));
        engine.set_delay(Duration::from_millis(1));

        // Collect timestamps from progress callback
        let timestamps = Arc::new(std::sync::Mutex::new(Vec::new()));
        let timestamps_clone = timestamps.clone();
        engine.set_progress_callback(move |date| {
            timestamps_clone.lock().unwrap().push(date.day());
        });

        engine
            .run(vec![aapl], date_range, |_asset, _timestamp, _value| Ok(()))
            .unwrap();

        let collected = timestamps.lock().unwrap();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], 1);
        assert_eq!(collected[1], 2);
        assert_eq!(collected[2], 3);
    }

    #[test]
    fn test_replay_works_without_progress_callback() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
        );

        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 101.0),
        ];
        provider.add_data(aapl.clone(), test_data);

        let mut engine = ReplayEngine::new(Arc::new(provider));
        engine.set_delay(Duration::from_millis(1));
        // No progress callback set

        let result = engine
            .run(vec![aapl], date_range, |_asset, _timestamp, _value| Ok(()))
            .unwrap();

        assert_eq!(result.successful, 2);
    }

    #[test]
    fn test_error_callback_receives_failure_information() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );

        let test_data = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 101.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 3, 0, 0, 0).unwrap(), 102.0),
        ];
        provider.add_data(aapl.clone(), test_data);

        let mut engine = ReplayEngine::new(Arc::new(provider));
        engine.set_delay(Duration::from_millis(1));

        // Track error callback invocations
        let error_count = Arc::new(std::sync::Mutex::new(0));
        let error_count_clone = error_count.clone();
        engine.set_error_callback(move |_asset, _timestamp, _error| {
            *error_count_clone.lock().unwrap() += 1;
        });

        // Make all callbacks fail
        let result = engine
            .run(vec![aapl], date_range, |_asset, _timestamp, _value| {
                Err("Test error".into())
            })
            .unwrap();

        assert_eq!(result.failed, 3);
        assert_eq!(*error_count.lock().unwrap(), 3);
    }

    // Task Group 5: Error Handling and Resilience Tests

    #[test]
    fn test_data_callback_always_fails_completes_without_crashing() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
        );

        let mut test_data = Vec::new();
        for day in 1..=5 {
            test_data.push(TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, day, 0, 0, 0).unwrap(),
                100.0,
            ));
        }
        provider.add_data(aapl.clone(), test_data);

        let mut engine = ReplayEngine::new(Arc::new(provider));
        engine.set_delay(Duration::from_millis(1));

        // Callback always fails
        let result = engine
            .run(vec![aapl], date_range, |_asset, _timestamp, _value| {
                Err("Always fails".into())
            })
            .unwrap();

        // Verify replay completed without crashing
        assert_eq!(result.total_points, 5);
        assert_eq!(result.successful, 0);
        assert_eq!(result.failed, 5);
    }

    #[test]
    fn test_callback_fails_on_specific_asset() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();
        let msft = AssetKey::new_equity("MSFT").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );

        // Add data for both assets
        provider.add_data(
            aapl.clone(),
            vec![
                TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 100.0),
                TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 101.0),
            ],
        );
        provider.add_data(
            msft.clone(),
            vec![
                TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), 200.0),
                TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap(), 201.0),
            ],
        );

        let mut engine = ReplayEngine::new(Arc::new(provider));
        engine.set_delay(Duration::from_millis(1));

        // Fail only for MSFT
        let result = engine
            .run(
                vec![aapl.clone(), msft.clone()],
                date_range,
                move |asset, _timestamp, _value| {
                    if asset == msft {
                        Err("MSFT not allowed".into())
                    } else {
                        Ok(())
                    }
                },
            )
            .unwrap();

        // AAPL should succeed, MSFT should fail
        assert_eq!(result.total_points, 4);
        assert_eq!(result.successful, 2); // AAPL points
        assert_eq!(result.failed, 2); // MSFT points
    }

    #[test]
    fn test_callback_fails_intermittently() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        let mut test_data = Vec::new();
        for day in 1..=10 {
            test_data.push(TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, day, 0, 0, 0).unwrap(),
                100.0 + day as f64,
            ));
        }
        provider.add_data(aapl.clone(), test_data);

        let mut engine = ReplayEngine::new(Arc::new(provider));
        engine.set_delay(Duration::from_millis(1));

        // Fail on even days
        let result = engine
            .run(vec![aapl], date_range, |_asset, timestamp, _value| {
                if timestamp.day() % 2 == 0 {
                    Err("Even day error".into())
                } else {
                    Ok(())
                }
            })
            .unwrap();

        assert_eq!(result.total_points, 10);
        assert_eq!(result.successful, 5); // Odd days
        assert_eq!(result.failed, 5); // Even days
    }

    // Task Group 6: Integration Testing

    #[test]
    fn test_integration_replay_collects_data_correctly() {
        // Set up data provider with 20 days of AAPL data
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();

        let mut test_data = Vec::new();
        for day in 1..=20 {
            test_data.push(TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, day, 0, 0, 0).unwrap(),
                100.0 + day as f64,
            ));
        }
        provider.add_data(aapl.clone(), test_data.clone());

        let provider_arc = Arc::new(provider);

        // Create ReplayEngine
        let mut replay = ReplayEngine::new(provider_arc);
        replay.set_delay(Duration::from_millis(1));

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(),
        );

        // Collect data points via callback
        let collected_data = Arc::new(std::sync::Mutex::new(Vec::new()));
        let collected_data_clone = collected_data.clone();

        // Run replay
        let result = replay
            .run(
                vec![aapl.clone()],
                date_range,
                move |asset, timestamp, value| {
                    collected_data_clone
                        .lock()
                        .unwrap()
                        .push((asset.clone(), timestamp, value));
                    Ok(())
                },
            )
            .unwrap();

        // Verify replay completed successfully
        assert_eq!(result.total_points, 20);
        assert_eq!(result.successful, 20);
        assert_eq!(result.failed, 0);

        // Verify all data was collected
        let collected = collected_data.lock().unwrap();
        assert_eq!(collected.len(), 20);

        // Verify data values match
        for (i, (asset, timestamp, value)) in collected.iter().enumerate() {
            assert_eq!(*asset, aapl);
            assert_eq!(timestamp.day(), (i + 1) as u32);
            assert_eq!(*value, 100.0 + (i + 1) as f64);
        }
    }

    #[test]
    fn test_multi_asset_replay_chronological_order() {
        let mut provider = InMemoryDataProvider::new();
        let aapl = AssetKey::new_equity("AAPL").unwrap();
        let msft = AssetKey::new_equity("MSFT").unwrap();
        let goog = AssetKey::new_equity("GOOG").unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
        );

        // Add 5 days of data for each asset
        for asset in [&aapl, &msft, &goog] {
            let mut data = Vec::new();
            for day in 1..=5 {
                data.push(TimeSeriesPoint::new(
                    Utc.with_ymd_and_hms(2024, 1, day, 0, 0, 0).unwrap(),
                    100.0 + day as f64,
                ));
            }
            provider.add_data(asset.clone(), data);
        }

        let mut engine = ReplayEngine::new(Arc::new(provider));
        engine.set_delay(Duration::from_millis(1));

        // Track order of asset processing
        let processing_order = Arc::new(std::sync::Mutex::new(Vec::new()));
        let processing_order_clone = processing_order.clone();

        let result = engine
            .run(
                vec![aapl.clone(), msft.clone(), goog.clone()],
                date_range,
                move |asset, timestamp, _value| {
                    processing_order_clone
                        .lock()
                        .unwrap()
                        .push((asset.clone(), timestamp.day()));
                    Ok(())
                },
            )
            .unwrap();

        // Should have 15 total points (5 per asset)
        assert_eq!(result.total_points, 15);
        assert_eq!(result.successful, 15);

        // Verify chronological ordering: each day should have all 3 assets before next day
        let order = processing_order.lock().unwrap();
        assert_eq!(order.len(), 15);

        // Days should be in order: 1,1,1,2,2,2,3,3,3,4,4,4,5,5,5
        for i in 0..15 {
            let expected_day = (i / 3) + 1;
            assert_eq!(order[i].1, expected_day as u32);
        }
    }

    #[test]
    fn test_replay_performance_benchmark() {
        let mut provider = InMemoryDataProvider::new();

        // Create 1 year of data (252 trading days) for 5 assets
        let assets: Vec<AssetKey> = vec!["AAPL", "MSFT", "GOOG", "AMZN", "TSLA"]
            .into_iter()
            .map(|ticker| AssetKey::new_equity(ticker).unwrap())
            .collect();

        let mut day_count = 0;
        for month in 1..=12 {
            let days_in_month = match month {
                2 => 20, // Approximate trading days
                _ => 21,
            };

            for day in 1..=days_in_month {
                day_count += 1;
                if day_count > 252 {
                    break;
                }

                for asset in &assets {
                    let data = vec![TimeSeriesPoint::new(
                        Utc.with_ymd_and_hms(2024, month, day, 0, 0, 0).unwrap(),
                        100.0 + day_count as f64,
                    )];

                    // Add to provider (need to check if asset exists first)
                    let existing = provider
                        .get_time_series(
                            asset,
                            &DateRange::new(
                                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                                NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
                            ),
                        )
                        .unwrap_or_default();

                    let mut combined = existing;
                    combined.extend(data);
                    provider.add_data(asset.clone(), combined);
                }
            }

            if day_count > 252 {
                break;
            }
        }

        let mut engine = ReplayEngine::new(Arc::new(provider));
        engine.set_delay(Duration::from_millis(10)); // 10ms delay

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        );

        let result = engine
            .run(assets, date_range, |_asset, _timestamp, _value| Ok(()))
            .unwrap();

        // Total points should be approximately 252 days × 5 assets = 1,260
        // (might be slightly less due to month/day calculation)
        assert!(result.total_points >= 1000);
        assert!(result.total_points <= 1300);
        assert_eq!(result.successful, result.total_points);

        // With 10ms delay, elapsed time should be approximately (points × 10ms)
        // Allow ±30% variance for test timing variability
        let expected_ms = result.total_points as u64 * 10;
        let actual_ms = result.elapsed.as_millis() as u64;
        let lower_bound = (expected_ms as f64 * 0.7) as u64;
        let upper_bound = (expected_ms as f64 * 1.3) as u64;

        assert!(
            actual_ms >= lower_bound && actual_ms <= upper_bound,
            "Expected ~{}ms, got {}ms (bounds: {}-{})",
            expected_ms,
            actual_ms,
            lower_bound,
            upper_bound
        );
    }
}
