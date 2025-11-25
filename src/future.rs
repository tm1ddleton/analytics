use crate::asset::{Asset, AssetType};
use crate::asset_key::AssetKey;
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
        let key = AssetKey::new_future(series.clone(), expiry_date)?;
        Ok(Future {
            key,
            series: series.into(),
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
}

