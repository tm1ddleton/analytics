use crate::asset::{Asset, AssetType};
use crate::asset_key::AssetKey;
use crate::time_series::{DataProvider, DateRange, TimeSeriesPoint};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Expiry calendar for futures contracts.
/// 
/// Provides functionality to determine contract rollover dates
/// and manage expiry calendar information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpiryCalendar {
    /// Calendar identifier (e.g., "CME", "ICE")
    pub calendar_id: String,
    /// Days before expiry to rollover to next contract
    pub rollover_days: u32,
}

impl ExpiryCalendar {
    /// Creates a new expiry calendar.
    pub fn new(calendar_id: impl Into<String>, rollover_days: u32) -> Self {
        ExpiryCalendar {
            calendar_id: calendar_id.into(),
            rollover_days,
        }
    }

    /// Calculates the rollover date based on the expiry date and rollover days.
    /// 
    /// # Arguments
    /// * `expiry_date` - The contract expiry date
    /// 
    /// # Returns
    /// Returns the date when the contract should be rolled over.
    pub fn rollover_date(&self, expiry_date: NaiveDate) -> NaiveDate {
        expiry_date - chrono::Duration::days(self.rollover_days as i64)
    }
}

/// Futures contract asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Future {
    /// Unique asset key (series + expiry date)
    key: AssetKey,
    /// Series identifier (underlying, e.g., "ES" for E-mini S&P 500)
    series: String,
    /// Contract expiry date
    expiry_date: NaiveDate,
    /// Contract month (e.g., "2024-12" for December 2024)
    contract_month: String,
    /// Common metadata (name, exchange, currency)
    metadata: crate::equity::AssetMetadata,
    /// Expiry calendar for rollover calculations
    expiry_calendar: ExpiryCalendar,
}

impl Future {
    /// Creates a new Future asset.
    /// 
    /// # Arguments
    /// * `series` - The underlying series identifier (e.g., "ES")
    /// * `expiry_date` - The contract expiry date
    /// * `contract_month` - The contract month (e.g., "2024-12")
    /// * `name` - The contract name/description
    /// * `exchange` - The exchange where it's traded
    /// * `currency` - The currency code
    /// * `calendar_id` - The expiry calendar identifier
    /// * `rollover_days` - Days before expiry to rollover
    /// 
    /// # Returns
    /// Returns `Ok(Future)` if valid, or `Err` if invalid.
    pub fn new(
        series: impl Into<String>,
        expiry_date: NaiveDate,
        contract_month: impl Into<String>,
        name: impl Into<String>,
        exchange: impl Into<String>,
        currency: impl Into<String>,
        calendar_id: impl Into<String>,
        rollover_days: u32,
    ) -> Result<Self, crate::asset_key::AssetKeyError> {
        let series_str: String = series.into();
        let key = AssetKey::new_future(series_str.clone(), expiry_date)?;
        Ok(Future {
            key,
            series: series_str,
            expiry_date,
            contract_month: contract_month.into(),
            metadata: crate::equity::AssetMetadata::new(name, exchange, currency),
            expiry_calendar: ExpiryCalendar::new(calendar_id, rollover_days),
        })
    }

    /// Returns the series identifier.
    pub fn series(&self) -> &str {
        &self.series
    }

    /// Returns the expiry date.
    pub fn expiry_date(&self) -> NaiveDate {
        self.expiry_date
    }

    /// Returns the contract month.
    pub fn contract_month(&self) -> &str {
        &self.contract_month
    }

    /// Returns the asset name.
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Returns the exchange.
    pub fn exchange(&self) -> &str {
        &self.metadata.exchange
    }

    /// Returns the currency.
    pub fn currency(&self) -> &str {
        &self.metadata.currency
    }

    /// Returns a reference to the expiry calendar.
    pub fn expiry_calendar(&self) -> &ExpiryCalendar {
        &self.expiry_calendar
    }

    /// Calculates the rollover date for this contract.
    pub fn rollover_date(&self) -> NaiveDate {
        self.expiry_calendar.rollover_date(self.expiry_date)
    }

    /// Queries time-series data for this future from a data provider.
    /// 
    /// # Arguments
    /// * `provider` - The data provider to query from (not stored in the asset)
    /// * `date_range` - The date range to query
    /// 
    /// # Returns
    /// Returns `Ok(Vec<TimeSeriesPoint>)` if successful, or an error if the query fails.
    /// 
    /// # Errors
    /// Returns an error if the asset is not found in the data provider or if the query fails.
    pub fn get_time_series(
        &self,
        provider: &dyn DataProvider,
        date_range: &DateRange,
    ) -> Result<Vec<TimeSeriesPoint>, crate::time_series::DataProviderError> {
        provider.get_time_series(self.key(), date_range)
    }

    /// Generates a rolling futures price series from multiple contracts.
    /// 
    /// This method creates a continuous price series by switching between
    /// contracts at the specified rollover points (days before expiry).
    /// 
    /// # Arguments
    /// * `provider` - The data provider to query from
    /// * `contracts` - Vector of futures contracts (ordered by expiry date)
    /// * `date_range` - The date range for the rolling series
    /// * `rollover_days` - Days before expiry to switch to next contract
    /// 
    /// # Returns
    /// Returns a continuous price series with prices from the appropriate contract
    /// at each point in time, switching contracts at rollover dates.
    /// 
    /// # Errors
    /// Returns an error if any contract data cannot be retrieved.
    pub fn generate_rolling_price_series(
        provider: &dyn DataProvider,
        contracts: &[&Self],
        date_range: &DateRange,
        rollover_days: u32,
    ) -> Result<Vec<TimeSeriesPoint>, crate::time_series::DataProviderError> {
        if contracts.is_empty() {
            return Ok(Vec::new());
        }

        let mut rolling_series = Vec::new();
        let mut current_contract_idx = 0;

        // Generate all dates in the range
        let mut current_date = date_range.start;
        while current_date <= date_range.end {
            // Check if we need to rollover to next contract
            while current_contract_idx < contracts.len() - 1 {
                let current_contract = contracts[current_contract_idx];
                let rollover_date = current_contract.expiry_date
                    - chrono::Duration::days(rollover_days as i64);

                if current_date >= rollover_date {
                    current_contract_idx += 1;
                } else {
                    break;
                }
            }

            // Get data from current contract for this date
            let contract = contracts[current_contract_idx];
            let single_day_range = DateRange::new(current_date, current_date);
            let contract_data = provider.get_time_series(contract.key(), &single_day_range)?;

            // Add the first point for this date (if any)
            if let Some(point) = contract_data.first() {
                rolling_series.push(point.clone());
            }

            // Move to next day
            current_date = current_date.succ_opt().unwrap_or(current_date);
        }

        Ok(rolling_series)
    }
}

impl Asset for Future {
    fn key(&self) -> &AssetKey {
        &self.key
    }

    fn asset_type(&self) -> AssetType {
        AssetType::Future
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_future_creation_with_series_and_expiry() {
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let future = Future::new(
            "ES",
            expiry,
            "2024-12",
            "E-mini S&P 500",
            "CME",
            "USD",
            "CME",
            5,
        ).unwrap();

        assert_eq!(future.series(), "ES");
        assert_eq!(future.expiry_date(), expiry);
        assert_eq!(future.contract_month(), "2024-12");
        assert!(matches!(future.key(), AssetKey::Future { .. }));
    }

    #[test]
    fn test_future_immutability() {
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let future1 = Future::new(
            "ES",
            expiry,
            "2024-12",
            "E-mini S&P 500",
            "CME",
            "USD",
            "CME",
            5,
        ).unwrap();

        let future2 = future1.clone();
        assert_eq!(future1, future2);
    }

    #[test]
    fn test_future_asset_type_discrimination() {
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let future = Future::new(
            "NQ",
            expiry,
            "2024-12",
            "E-mini NASDAQ-100",
            "CME",
            "USD",
            "CME",
            5,
        ).unwrap();

        assert_eq!(future.asset_type(), AssetType::Future);
    }

    #[test]
    fn test_future_metadata_field_access() {
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let future = Future::new(
            "ES",
            expiry,
            "2024-12",
            "E-mini S&P 500",
            "CME",
            "USD",
            "CME",
            5,
        ).unwrap();

        assert_eq!(future.name(), "E-mini S&P 500");
        assert_eq!(future.exchange(), "CME");
        assert_eq!(future.currency(), "USD");
    }

    #[test]
    fn test_future_expiry_calendar() {
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let future = Future::new(
            "ES",
            expiry,
            "2024-12",
            "E-mini S&P 500",
            "CME",
            "USD",
            "CME",
            5,
        ).unwrap();

        let rollover = future.rollover_date();
        let expected_rollover = expiry - chrono::Duration::days(5);
        assert_eq!(rollover, expected_rollover);
    }

    #[test]
    fn test_future_invalid_series() {
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let result = Future::new(
            "",
            expiry,
            "2024-12",
            "Invalid Future",
            "CME",
            "USD",
            "CME",
            5,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_future_query_time_series() {
        use crate::time_series::{InMemoryDataProvider, DateRange};
        use chrono::{TimeZone, Utc};

        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let future = Future::new(
            "ES",
            expiry,
            "2024-12",
            "E-mini S&P 500",
            "CME",
            "USD",
            "CME",
            5,
        ).unwrap();

        let mut provider = InMemoryDataProvider::new();
        let points = vec![
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 12, 15, 16, 0, 0).unwrap(),
                4500.0,
            ),
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 12, 16, 16, 0, 0).unwrap(),
                4510.0,
            ),
        ];
        provider.add_data(future.key().clone(), points);

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 12, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 16).unwrap(),
        );

        let result = future.get_time_series(&provider, &date_range).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].close_price, 4500.0);
    }

    #[test]
    fn test_future_serialize_deserialize() {
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let future = Future::new(
            "ES",
            expiry,
            "2024-12",
            "E-mini S&P 500",
            "CME",
            "USD",
            "CME",
            5,
        ).unwrap();

        let json = serde_json::to_string(&future).unwrap();
        let deserialized: Future = serde_json::from_str(&json).unwrap();

        assert_eq!(future.series(), deserialized.series());
        assert_eq!(future.expiry_date(), deserialized.expiry_date());
        assert_eq!(future.contract_month(), deserialized.contract_month());
        assert_eq!(future.name(), deserialized.name());
    }

    #[test]
    fn test_future_expiry_calendar_rollover() {
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let future = Future::new(
            "ES",
            expiry,
            "2024-12",
            "E-mini S&P 500",
            "CME",
            "USD",
            "CME",
            5,
        ).unwrap();

        let rollover = future.rollover_date();
        let expected = expiry - chrono::Duration::days(5);
        assert_eq!(rollover, expected);
    }

    #[test]
    fn test_future_generate_rolling_price_series() {
        use crate::time_series::{InMemoryDataProvider, DateRange};
        use chrono::{TimeZone, Utc};

        // Create two contracts with different expiry dates
        let expiry1 = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let contract1 = Future::new(
            "ES",
            expiry1,
            "2024-12",
            "E-mini S&P 500 Dec 2024",
            "CME",
            "USD",
            "CME",
            5,
        ).unwrap();

        let expiry2 = NaiveDate::from_ymd_opt(2025, 3, 20).unwrap();
        let contract2 = Future::new(
            "ES",
            expiry2,
            "2025-03",
            "E-mini S&P 500 Mar 2025",
            "CME",
            "USD",
            "CME",
            5,
        ).unwrap();

        let mut provider = InMemoryDataProvider::new();

        // Add data for contract1 (Dec 2024)
        let points1 = vec![
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 12, 10, 16, 0, 0).unwrap(),
                4500.0,
            ),
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 12, 15, 16, 0, 0).unwrap(), // Rollover date
                4510.0,
            ),
        ];
        provider.add_data(contract1.key().clone(), points1);

        // Add data for contract2 (Mar 2025)
        let points2 = vec![
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 12, 16, 16, 0, 0).unwrap(), // After rollover
                4520.0,
            ),
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 12, 17, 16, 0, 0).unwrap(),
                4530.0,
            ),
        ];
        provider.add_data(contract2.key().clone(), points2);

        let contracts = vec![&contract1, &contract2];
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 12, 10).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 17).unwrap(),
        );

        let rolling_series = Future::generate_rolling_price_series(
            &provider,
            &contracts,
            &date_range,
            5, // 5 days before expiry
        ).unwrap();

        // Should have prices from both contracts, switching at rollover
        assert!(rolling_series.len() >= 2);
        // Prices should come from appropriate contracts based on rollover date
    }
}

