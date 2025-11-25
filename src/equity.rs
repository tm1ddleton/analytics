use crate::asset::{Asset, AssetType};
use crate::asset_key::AssetKey;
use crate::time_series::{DataProvider, DateRange, TimeSeriesPoint};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Common metadata fields shared across asset types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetMetadata {
    /// Asset name (e.g., "Apple Inc.")
    pub name: String,
    /// Exchange where the asset is traded (e.g., "NASDAQ", "NYSE")
    pub exchange: String,
    /// Currency code (e.g., "USD", "EUR")
    pub currency: String,
}

impl AssetMetadata {
    /// Creates a new AssetMetadata instance.
    pub fn new(name: impl Into<String>, exchange: impl Into<String>, currency: impl Into<String>) -> Self {
        AssetMetadata {
            name: name.into(),
            exchange: exchange.into(),
            currency: currency.into(),
        }
    }
}

/// Corporate action types for equities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorporateAction {
    /// Stock split (e.g., 2-for-1 split)
    Split {
        ratio: f64,
        effective_date: NaiveDate,
    },
    /// Dividend payment
    Dividend {
        amount: f64,
        ex_date: NaiveDate,
        payment_date: NaiveDate,
    },
}

/// Equity asset representing a stock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equity {
    /// Unique asset key (ticker symbol)
    key: AssetKey,
    /// Common metadata (name, exchange, currency)
    metadata: AssetMetadata,
    /// Sector classification (e.g., "Technology", "Healthcare")
    sector: String,
    /// Corporate actions (splits, dividends, etc.)
    corporate_actions: Vec<CorporateAction>,
}

impl Equity {
    /// Creates a new Equity asset.
    /// 
    /// # Arguments
    /// * `ticker` - The ticker symbol (e.g., "AAPL")
    /// * `name` - The company name
    /// * `exchange` - The exchange where it's traded
    /// * `currency` - The currency code
    /// * `sector` - The sector classification
    /// 
    /// # Returns
    /// Returns `Ok(Equity)` if the ticker is valid, or `Err` if invalid.
    pub fn new(
        ticker: impl Into<String>,
        name: impl Into<String>,
        exchange: impl Into<String>,
        currency: impl Into<String>,
        sector: impl Into<String>,
    ) -> Result<Self, crate::asset_key::AssetKeyError> {
        let key = AssetKey::new_equity(ticker)?;
        Ok(Equity {
            key,
            metadata: AssetMetadata::new(name, exchange, currency),
            sector: sector.into(),
            corporate_actions: Vec::new(),
        })
    }

    /// Creates a new Equity asset with corporate actions.
    /// 
    /// # Arguments
    /// * `ticker` - The ticker symbol
    /// * `name` - The company name
    /// * `exchange` - The exchange where it's traded
    /// * `currency` - The currency code
    /// * `sector` - The sector classification
    /// * `corporate_actions` - Vector of corporate actions
    /// 
    /// # Returns
    /// Returns `Ok(Equity)` if the ticker is valid, or `Err` if invalid.
    pub fn with_corporate_actions(
        ticker: impl Into<String>,
        name: impl Into<String>,
        exchange: impl Into<String>,
        currency: impl Into<String>,
        sector: impl Into<String>,
        corporate_actions: Vec<CorporateAction>,
    ) -> Result<Self, crate::asset_key::AssetKeyError> {
        let key = AssetKey::new_equity(ticker)?;
        Ok(Equity {
            key,
            metadata: AssetMetadata::new(name, exchange, currency),
            sector: sector.into(),
            corporate_actions,
        })
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

    /// Returns the sector.
    pub fn sector(&self) -> &str {
        &self.sector
    }

    /// Returns a reference to the corporate actions.
    pub fn corporate_actions(&self) -> &[CorporateAction] {
        &self.corporate_actions
    }

    /// Queries time-series data for this equity from a data provider.
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

    /// Applies corporate actions to a price data point.
    /// 
    /// This is a rudimentary implementation for POC that adjusts prices
    /// based on corporate actions that occurred before or on the given date.
    /// 
    /// # Arguments
    /// * `price` - The original price
    /// * `date` - The date of the price point
    /// 
    /// # Returns
    /// Returns the adjusted price after applying relevant corporate actions.
    pub fn apply_corporate_actions(&self, price: f64, date: NaiveDate) -> f64 {
        let mut adjusted_price = price;

        // Apply splits (price adjustment)
        for action in &self.corporate_actions {
            if let CorporateAction::Split { ratio, effective_date } = action {
                if date >= *effective_date {
                    // Adjust price for split (divide by ratio)
                    adjusted_price = adjusted_price / ratio;
                }
            }
        }

        adjusted_price
    }

    /// Applies corporate actions to a vector of time-series points.
    /// 
    /// # Arguments
    /// * `points` - Vector of time-series points to adjust
    /// 
    /// # Returns
    /// Returns a new vector with adjusted prices.
    pub fn apply_corporate_actions_to_series(&self, points: Vec<TimeSeriesPoint>) -> Vec<TimeSeriesPoint> {
        points
            .into_iter()
            .map(|point| {
                let adjusted_price = self.apply_corporate_actions(
                    point.close_price,
                    point.timestamp.date_naive(),
                );
                TimeSeriesPoint::new(point.timestamp, adjusted_price)
            })
            .collect()
    }
}

impl Asset for Equity {
    fn key(&self) -> &AssetKey {
        &self.key
    }

    fn asset_type(&self) -> AssetType {
        AssetType::Equity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_equity_creation_with_metadata() {
        let equity = Equity::new(
            "AAPL",
            "Apple Inc.",
            "NASDAQ",
            "USD",
            "Technology",
        ).unwrap();

        assert_eq!(equity.name(), "Apple Inc.");
        assert_eq!(equity.exchange(), "NASDAQ");
        assert_eq!(equity.currency(), "USD");
        assert_eq!(equity.sector(), "Technology");
        assert!(matches!(equity.key(), AssetKey::Equity(_)));
    }

    #[test]
    fn test_equity_creation_with_corporate_actions() {
        let split = CorporateAction::Split {
            ratio: 2.0,
            effective_date: NaiveDate::from_ymd_opt(2020, 8, 31).unwrap(),
        };
        let dividend = CorporateAction::Dividend {
            amount: 0.82,
            ex_date: NaiveDate::from_ymd_opt(2023, 11, 10).unwrap(),
            payment_date: NaiveDate::from_ymd_opt(2023, 11, 16).unwrap(),
        };

        let equity = Equity::with_corporate_actions(
            "AAPL",
            "Apple Inc.",
            "NASDAQ",
            "USD",
            "Technology",
            vec![split.clone(), dividend.clone()],
        ).unwrap();

        assert_eq!(equity.corporate_actions().len(), 2);
        assert_eq!(equity.corporate_actions()[0], split);
        assert_eq!(equity.corporate_actions()[1], dividend);
    }

    #[test]
    fn test_equity_immutability() {
        let equity1 = Equity::new(
            "AAPL",
            "Apple Inc.",
            "NASDAQ",
            "USD",
            "Technology",
        ).unwrap();

        let equity2 = equity1.clone();
        assert_eq!(equity1, equity2);
    }

    #[test]
    fn test_equity_asset_type_discrimination() {
        let equity = Equity::new(
            "MSFT",
            "Microsoft Corporation",
            "NASDAQ",
            "USD",
            "Technology",
        ).unwrap();

        assert_eq!(equity.asset_type(), AssetType::Equity);
    }

    #[test]
    fn test_equity_metadata_field_access() {
        let equity = Equity::new(
            "GOOGL",
            "Alphabet Inc.",
            "NASDAQ",
            "USD",
            "Technology",
        ).unwrap();

        assert_eq!(equity.name(), "Alphabet Inc.");
        assert_eq!(equity.exchange(), "NASDAQ");
        assert_eq!(equity.currency(), "USD");
        assert_eq!(equity.sector(), "Technology");
    }

    #[test]
    fn test_equity_invalid_ticker() {
        let result = Equity::new(
            "",
            "Invalid Corp",
            "NYSE",
            "USD",
            "Finance",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_equity_query_time_series() {
        use crate::time_series::{InMemoryDataProvider, DateRange};
        use chrono::{TimeZone, Utc};

        let equity = Equity::new(
            "AAPL",
            "Apple Inc.",
            "NASDAQ",
            "USD",
            "Technology",
        ).unwrap();

        let mut provider = InMemoryDataProvider::new();
        let points = vec![
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap(),
                150.0,
            ),
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, 16, 16, 0, 0).unwrap(),
                151.0,
            ),
        ];
        provider.add_data(equity.key().clone(), points);

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        );

        let result = equity.get_time_series(&provider, &date_range).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].close_price, 150.0);
    }

    #[test]
    fn test_equity_serialize_deserialize() {
        let equity = Equity::new(
            "AAPL",
            "Apple Inc.",
            "NASDAQ",
            "USD",
            "Technology",
        ).unwrap();

        let json = serde_json::to_string(&equity).unwrap();
        let deserialized: Equity = serde_json::from_str(&json).unwrap();

        assert_eq!(equity.name(), deserialized.name());
        assert_eq!(equity.exchange(), deserialized.exchange());
        assert_eq!(equity.currency(), deserialized.currency());
        assert_eq!(equity.sector(), deserialized.sector());
    }

    #[test]
    fn test_equity_apply_corporate_actions_split() {
        let split = CorporateAction::Split {
            ratio: 2.0,
            effective_date: NaiveDate::from_ymd_opt(2020, 8, 31).unwrap(),
        };

        let equity = Equity::with_corporate_actions(
            "AAPL",
            "Apple Inc.",
            "NASDAQ",
            "USD",
            "Technology",
            vec![split],
        ).unwrap();

        // Price before split should be unchanged
        let price_before = equity.apply_corporate_actions(200.0, NaiveDate::from_ymd_opt(2020, 8, 30).unwrap());
        assert_eq!(price_before, 200.0);

        // Price after split should be adjusted (divided by ratio)
        let price_after = equity.apply_corporate_actions(200.0, NaiveDate::from_ymd_opt(2020, 9, 1).unwrap());
        assert_eq!(price_after, 100.0);
    }

    #[test]
    fn test_equity_apply_corporate_actions_to_series() {
        use chrono::{TimeZone, Utc};

        let split = CorporateAction::Split {
            ratio: 2.0,
            effective_date: NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        };

        let equity = Equity::with_corporate_actions(
            "AAPL",
            "Apple Inc.",
            "NASDAQ",
            "USD",
            "Technology",
            vec![split],
        ).unwrap();

        let points = vec![
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap(),
                200.0, // Before split
            ),
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, 16, 16, 0, 0).unwrap(),
                200.0, // On split date
            ),
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, 17, 16, 0, 0).unwrap(),
                200.0, // After split
            ),
        ];

        let adjusted = equity.apply_corporate_actions_to_series(points);
        assert_eq!(adjusted[0].close_price, 200.0); // Before split
        assert_eq!(adjusted[1].close_price, 100.0); // On split date
        assert_eq!(adjusted[2].close_price, 100.0); // After split
    }
}

