use crate::asset_key::AssetKey;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Range;

/// A single time-series data point containing timestamp and close price.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    /// Timestamp of the data point
    pub timestamp: DateTime<Utc>,
    /// Close price at this timestamp
    pub close_price: f64,
}

impl TimeSeriesPoint {
    /// Creates a new TimeSeriesPoint.
    pub fn new(timestamp: DateTime<Utc>, close_price: f64) -> Self {
        TimeSeriesPoint {
            timestamp,
            close_price,
        }
    }
}

/// Date range for querying time-series data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRange {
    /// Start date (inclusive)
    pub start: NaiveDate,
    /// End date (inclusive)
    pub end: NaiveDate,
}

impl DateRange {
    /// Creates a new DateRange.
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        DateRange { start, end }
    }

    /// Creates a DateRange from a standard Range.
    pub fn from_range(range: Range<NaiveDate>) -> Self {
        DateRange {
            start: range.start,
            end: range.end,
        }
    }
}

/// Trait for data source abstraction.
///
/// This trait allows assets to query time-series data from any source
/// without being coupled to a specific database or storage implementation.
///
/// Implementations can be:
/// - In-memory HashMap (for testing)
/// - SQLite database
/// - REST API client
/// - File-based storage
/// - Any other data source
pub trait DataProvider {
    /// Retrieves time-series data for a given asset key and date range.
    ///
    /// # Arguments
    /// * `asset_key` - The asset key to query data for
    /// * `date_range` - The date range to query (inclusive on both ends)
    ///
    /// # Returns
    /// Returns `Ok(Vec<TimeSeriesPoint>)` if successful, or an error if the query fails.
    ///
    /// # Errors
    /// Returns an error if the asset key is not found, the date range is invalid,
    /// or if there's an issue accessing the data source.
    fn get_time_series(
        &self,
        asset_key: &AssetKey,
        date_range: &DateRange,
    ) -> Result<Vec<TimeSeriesPoint>, DataProviderError>;
}

/// Errors that can occur when querying a data provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataProviderError {
    /// Asset key not found in the data source
    AssetNotFound,
    /// Invalid date range (e.g., start > end)
    InvalidDateRange,
    /// Generic error message
    Other(String),
}

impl std::fmt::Display for DataProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataProviderError::AssetNotFound => write!(f, "Asset not found"),
            DataProviderError::InvalidDateRange => write!(f, "Invalid date range"),
            DataProviderError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DataProviderError {}

/// In-memory data provider implementation for testing.
///
/// Stores time-series data in a HashMap keyed by AssetKey.
/// This allows testing without requiring a database connection.
#[derive(Debug, Clone)]
pub struct InMemoryDataProvider {
    data: HashMap<AssetKey, Vec<TimeSeriesPoint>>,
}

impl InMemoryDataProvider {
    /// Creates a new empty in-memory data provider.
    pub fn new() -> Self {
        InMemoryDataProvider {
            data: HashMap::new(),
        }
    }

    /// Adds time-series data for an asset.
    ///
    /// # Arguments
    /// * `asset_key` - The asset key
    /// * `points` - Vector of time-series points (should be sorted by timestamp)
    pub fn add_data(&mut self, asset_key: AssetKey, points: Vec<TimeSeriesPoint>) {
        self.data.insert(asset_key, points);
    }

    /// Clears all data from the provider.
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for InMemoryDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DataProvider for InMemoryDataProvider {
    fn get_time_series(
        &self,
        asset_key: &AssetKey,
        date_range: &DateRange,
    ) -> Result<Vec<TimeSeriesPoint>, DataProviderError> {
        // Validate date range
        if date_range.start > date_range.end {
            return Err(DataProviderError::InvalidDateRange);
        }

        // Look up asset data
        let all_points = self
            .data
            .get(asset_key)
            .ok_or(DataProviderError::AssetNotFound)?;

        // Filter points within date range
        let filtered: Vec<TimeSeriesPoint> = all_points
            .iter()
            .filter(|point| {
                let point_date = point.timestamp.date_naive();
                point_date >= date_range.start && point_date <= date_range.end
            })
            .cloned()
            .collect();

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone};

    #[test]
    fn test_time_series_point_creation() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap();
        let point = TimeSeriesPoint::new(timestamp, 150.25);
        assert_eq!(point.close_price, 150.25);
        assert_eq!(point.timestamp, timestamp);
    }

    #[test]
    fn test_time_series_point_immutability() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap();
        let point1 = TimeSeriesPoint::new(timestamp, 150.25);
        let point2 = point1.clone();
        assert_eq!(point1, point2);
    }

    #[test]
    fn test_in_memory_data_provider_add_and_query() {
        let mut provider = InMemoryDataProvider::new();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();

        let points = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap(), 150.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 16, 16, 0, 0).unwrap(), 151.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 17, 16, 0, 0).unwrap(), 152.0),
        ];

        provider.add_data(asset_key.clone(), points);

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        );

        let result = provider.get_time_series(&asset_key, &date_range).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].close_price, 150.0);
        assert_eq!(result[1].close_price, 151.0);
    }

    #[test]
    fn test_in_memory_data_provider_asset_not_found() {
        let provider = InMemoryDataProvider::new();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        );

        let result = provider.get_time_series(&asset_key, &date_range);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DataProviderError::AssetNotFound);
    }

    #[test]
    fn test_in_memory_data_provider_invalid_date_range() {
        let provider = InMemoryDataProvider::new();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), // Invalid: start > end
        );

        let result = provider.get_time_series(&asset_key, &date_range);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DataProviderError::InvalidDateRange);
    }

    #[test]
    fn test_data_provider_trait_implementation() {
        let mut provider = InMemoryDataProvider::new();
        let asset_key = AssetKey::new_equity("MSFT").unwrap();

        let points = vec![TimeSeriesPoint::new(
            Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap(),
            400.0,
        )];

        provider.add_data(asset_key.clone(), points);

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let result = provider.get_time_series(&asset_key, &date_range).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].close_price, 400.0);
    }

    #[test]
    fn test_date_range_filtering() {
        let mut provider = InMemoryDataProvider::new();
        let asset_key = AssetKey::new_equity("GOOGL").unwrap();

        let points = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 14, 16, 0, 0).unwrap(), 100.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap(), 101.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 16, 16, 0, 0).unwrap(), 102.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 17, 16, 0, 0).unwrap(), 103.0),
        ];

        provider.add_data(asset_key.clone(), points);

        // Query only middle dates
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        );

        let result = provider.get_time_series(&asset_key, &date_range).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].close_price, 101.0);
        assert_eq!(result[1].close_price, 102.0);
    }
}
