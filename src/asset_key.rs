use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Asset key for uniquely identifying assets.
///
/// Supports two key formats:
/// - Equity keys: Simple string-based ticker symbols (e.g., "AAPL", "MSFT")
/// - Futures keys: Composite key with series (underlying) and expiry date
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetKey {
    /// Equity asset key (ticker symbol)
    Equity(String),
    /// Futures asset key (series + expiry date)
    Future {
        series: String,
        expiry_date: NaiveDate,
    },
}

impl AssetKey {
    /// Creates a new equity key from a ticker symbol.
    ///
    /// # Arguments
    /// * `ticker` - The ticker symbol (e.g., "AAPL", "MSFT")
    ///
    /// # Returns
    /// Returns `Ok(AssetKey::Equity(...))` if valid, or `Err` if invalid.
    ///
    /// # Errors
    /// Returns an error if the ticker is empty or contains invalid characters.
    pub fn new_equity(ticker: impl Into<String>) -> Result<Self, AssetKeyError> {
        let ticker = ticker.into();
        Self::validate_equity_key(&ticker)?;
        Ok(AssetKey::Equity(ticker))
    }

    /// Creates a new futures key from a series and expiry date.
    ///
    /// # Arguments
    /// * `series` - The underlying series identifier (e.g., "ES" for E-mini S&P 500)
    /// * `expiry_date` - The contract expiry date
    ///
    /// # Returns
    /// Returns `Ok(AssetKey::Future { ... })` if valid, or `Err` if invalid.
    ///
    /// # Errors
    /// Returns an error if the series is empty or contains invalid characters.
    pub fn new_future(
        series: impl Into<String>,
        expiry_date: NaiveDate,
    ) -> Result<Self, AssetKeyError> {
        let series = series.into();
        Self::validate_futures_key(&series)?;
        Ok(AssetKey::Future {
            series,
            expiry_date,
        })
    }

    /// Validates an equity key format.
    ///
    /// Rejects empty strings and strings containing invalid characters.
    fn validate_equity_key(ticker: &str) -> Result<(), AssetKeyError> {
        if ticker.is_empty() {
            return Err(AssetKeyError::EmptyKey);
        }

        // Check for invalid characters (allow alphanumeric, dots, hyphens, underscores)
        if !ticker
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            return Err(AssetKeyError::InvalidCharacters);
        }

        Ok(())
    }

    /// Validates a futures series key format.
    ///
    /// Rejects empty strings and strings containing invalid characters.
    fn validate_futures_key(series: &str) -> Result<(), AssetKeyError> {
        if series.is_empty() {
            return Err(AssetKeyError::EmptyKey);
        }

        // Check for invalid characters (allow alphanumeric, dots, hyphens, underscores)
        if !series
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            return Err(AssetKeyError::InvalidCharacters);
        }

        Ok(())
    }

    /// Returns the string representation of the key for lookup purposes.
    ///
    /// For equities, returns the ticker symbol.
    /// For futures, returns a formatted string combining series and expiry.
    pub fn as_string(&self) -> String {
        match self {
            AssetKey::Equity(ticker) => ticker.clone(),
            AssetKey::Future {
                series,
                expiry_date,
            } => {
                format!("{}-{}", series, expiry_date.format("%Y-%m-%d"))
            }
        }
    }
}

impl fmt::Display for AssetKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetKey::Equity(ticker) => write!(f, "{}", ticker),
            AssetKey::Future {
                series,
                expiry_date,
            } => {
                write!(f, "{}-{}", series, expiry_date.format("%Y-%m-%d"))
            }
        }
    }
}

/// Errors that can occur when creating or validating asset keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetKeyError {
    /// The key is empty
    EmptyKey,
    /// The key contains invalid characters
    InvalidCharacters,
}

impl fmt::Display for AssetKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetKeyError::EmptyKey => write!(f, "Asset key cannot be empty"),
            AssetKeyError::InvalidCharacters => {
                write!(f, "Asset key contains invalid characters")
            }
        }
    }
}

impl std::error::Error for AssetKeyError {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_equity_key_creation_valid() {
        let key = AssetKey::new_equity("AAPL").unwrap();
        assert!(matches!(key, AssetKey::Equity(_)));
        if let AssetKey::Equity(ticker) = key {
            assert_eq!(ticker, "AAPL");
        }
    }

    #[test]
    fn test_equity_key_creation_empty_string() {
        let result = AssetKey::new_equity("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AssetKeyError::EmptyKey);
    }

    #[test]
    fn test_equity_key_validation_invalid_characters() {
        let result = AssetKey::new_equity("AAPL@");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AssetKeyError::InvalidCharacters);
    }

    #[test]
    fn test_equity_key_immutability() {
        let key1 = AssetKey::new_equity("AAPL").unwrap();
        let key2 = key1.clone();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_futures_key_creation_valid() {
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let key = AssetKey::new_future("ES", expiry).unwrap();
        assert!(matches!(key, AssetKey::Future { .. }));
        if let AssetKey::Future {
            series,
            expiry_date,
        } = key
        {
            assert_eq!(series, "ES");
            assert_eq!(expiry_date, expiry);
        }
    }

    #[test]
    fn test_futures_key_creation_empty_series() {
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let result = AssetKey::new_future("", expiry);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AssetKeyError::EmptyKey);
    }

    #[test]
    fn test_asset_key_display_equity() {
        let key = AssetKey::new_equity("AAPL").unwrap();
        assert_eq!(format!("{}", key), "AAPL");
    }

    #[test]
    fn test_asset_key_display_future() {
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let key = AssetKey::new_future("ES", expiry).unwrap();
        assert_eq!(format!("{}", key), "ES-2024-12-20");
    }

    #[test]
    fn test_asset_key_as_string() {
        let equity_key = AssetKey::new_equity("MSFT").unwrap();
        assert_eq!(equity_key.as_string(), "MSFT");

        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let future_key = AssetKey::new_future("ES", expiry).unwrap();
        assert_eq!(future_key.as_string(), "ES-2024-12-20");
    }

    #[test]
    fn test_asset_key_hashable() {
        use std::collections::HashMap;

        let key1 = AssetKey::new_equity("AAPL").unwrap();
        let key2 = AssetKey::new_equity("AAPL").unwrap();
        let key3 = AssetKey::new_equity("MSFT").unwrap();

        let mut map = HashMap::new();
        map.insert(key1.clone(), "Apple Inc.");
        assert_eq!(map.get(&key2), Some(&"Apple Inc."));
        assert_eq!(map.get(&key3), None);
    }
}
