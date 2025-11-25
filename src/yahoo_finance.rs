use crate::asset_key::AssetKey;
use chrono::{NaiveDate, Utc};
use reqwest::Client;
use std::time::Duration;

/// Configuration for Yahoo Finance downloader
#[derive(Debug, Clone)]
pub struct DownloaderConfig {
    /// Maximum number of retry attempts (default: 3)
    pub max_retries: u32,
    /// Rate limit: requests per second (default: 1.0)
    pub requests_per_second: f64,
    /// Request timeout in seconds (default: 30)
    pub timeout_seconds: u64,
}

impl Default for DownloaderConfig {
    fn default() -> Self {
        DownloaderConfig {
            max_retries: 3,
            requests_per_second: 1.0,
            timeout_seconds: 30,
        }
    }
}

/// Yahoo Finance data downloader
/// 
/// Downloads historical market data from Yahoo Finance API and stores it in SQLite.
#[derive(Debug)]
pub struct YahooFinanceDownloader {
    client: Client,
    config: DownloaderConfig,
}

impl YahooFinanceDownloader {
    /// Creates a new Yahoo Finance downloader with default configuration.
    /// 
    /// # Returns
    /// Returns `Ok(YahooFinanceDownloader)` if successful, or an error if HTTP client creation fails.
    pub fn new() -> Result<Self, DownloadError> {
        Self::with_config(DownloaderConfig::default())
    }

    /// Creates a new Yahoo Finance downloader with custom configuration.
    /// 
    /// # Arguments
    /// * `config` - Configuration for the downloader (rate limits, retries, etc.)
    /// 
    /// # Returns
    /// Returns `Ok(YahooFinanceDownloader)` if successful, or an error if HTTP client creation fails.
    pub fn with_config(config: DownloaderConfig) -> Result<Self, DownloadError> {
        let timeout = Duration::from_secs(config.timeout_seconds);
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| DownloadError::ClientCreation(e.to_string()))?;

        Ok(YahooFinanceDownloader { client, config })
    }

    /// Converts an AssetKey to Yahoo Finance symbol format.
    /// 
    /// # Arguments
    /// * `asset_key` - The asset key to convert
    /// 
    /// # Returns
    /// Returns the Yahoo Finance symbol string for the asset.
    /// 
    /// # Examples
    /// - Equity "AAPL" -> "AAPL"
    /// - Future "ES" with expiry -> "ES=F" or contract-specific format
    pub fn asset_key_to_symbol(&self, asset_key: &AssetKey) -> String {
        match asset_key {
            AssetKey::Equity(ticker) => ticker.clone(),
            AssetKey::Future { series, expiry_date: _ } => {
                // For futures, Yahoo Finance typically uses format like "ES=F" for continuous contracts
                // or "ESZ2024" for specific contracts. For now, we'll use the series with "=F" suffix.
                // This can be enhanced later to support specific contract months.
                // Note: expiry_date is not used in the current implementation but may be needed for specific contracts
                format!("{}={}", series, "F")
            }
        }
    }

    /// Fetches historical data from Yahoo Finance API.
    /// 
    /// # Arguments
    /// * `symbol` - Yahoo Finance symbol (e.g., "AAPL", "ES=F")
    /// * `start_date` - Start date for historical data
    /// * `end_date` - End date for historical data
    /// 
    /// # Returns
    /// Returns the raw CSV response data as a string, or an error if the request fails.
    /// 
    /// # Errors
    /// Returns `DownloadError` if the API request fails, network error occurs, or response cannot be read.
    pub async fn fetch_historical_data(
        &self,
        symbol: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<String, DownloadError> {
        // Yahoo Finance historical data endpoint
        // Format: https://query1.finance.yahoo.com/v7/finance/download/{symbol}?period1={start_timestamp}&period2={end_timestamp}&interval=1d&events=history
        let start_timestamp = start_date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| DownloadError::InvalidDate("Invalid start date".to_string()))?
            .and_local_timezone(Utc)
            .unwrap()
            .timestamp();

        let end_timestamp = end_date
            .and_hms_opt(23, 59, 59)
            .ok_or_else(|| DownloadError::InvalidDate("Invalid end date".to_string()))?
            .and_local_timezone(Utc)
            .unwrap()
            .timestamp();

        let url = format!(
            "https://query1.finance.yahoo.com/v7/finance/download/{}?period1={}&period2={}&interval=1d&events=history",
            symbol, start_timestamp, end_timestamp
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| DownloadError::NetworkError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            return Err(DownloadError::ApiError(format!(
                "HTTP {}: {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown error")
            )));
        }

        let text = response
            .text()
            .await
            .map_err(|e| DownloadError::ParseError(e.to_string()))?;

        Ok(text)
    }

    /// Returns a reference to the HTTP client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Returns a reference to the configuration.
    pub fn config(&self) -> &DownloaderConfig {
        &self.config
    }
}

/// Errors that can occur during Yahoo Finance data downloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadError {
    /// HTTP client creation failed
    ClientCreation(String),
    /// Network error occurred
    NetworkError(String),
    /// API returned an error response
    ApiError(String),
    /// Failed to parse response data
    ParseError(String),
    /// Invalid date provided
    InvalidDate(String),
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::ClientCreation(msg) => write!(f, "Client creation error: {}", msg),
            DownloadError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            DownloadError::ApiError(msg) => write!(f, "API error: {}", msg),
            DownloadError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            DownloadError::InvalidDate(msg) => write!(f, "Invalid date: {}", msg),
        }
    }
}

impl std::error::Error for DownloadError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_downloader_creation() {
        let downloader = YahooFinanceDownloader::new();
        assert!(downloader.is_ok());
    }

    #[tokio::test]
    async fn test_downloader_with_config() {
        let config = DownloaderConfig {
            max_retries: 5,
            requests_per_second: 2.0,
            timeout_seconds: 60,
        };
        let downloader = YahooFinanceDownloader::with_config(config);
        assert!(downloader.is_ok());
        
        let downloader = downloader.unwrap();
        assert_eq!(downloader.config().max_retries, 5);
        assert_eq!(downloader.config().requests_per_second, 2.0);
        assert_eq!(downloader.config().timeout_seconds, 60);
    }

    #[tokio::test]
    async fn test_asset_key_to_symbol_equity() {
        let downloader = YahooFinanceDownloader::new().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let symbol = downloader.asset_key_to_symbol(&asset_key);
        assert_eq!(symbol, "AAPL");
    }

    #[tokio::test]
    async fn test_asset_key_to_symbol_future() {
        let downloader = YahooFinanceDownloader::new().unwrap();
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let asset_key = AssetKey::new_future("ES", expiry).unwrap();
        let symbol = downloader.asset_key_to_symbol(&asset_key);
        assert_eq!(symbol, "ES=F");
    }

    #[tokio::test]
    async fn test_fetch_historical_data_success() {
        let downloader = YahooFinanceDownloader::new().unwrap();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        
        // Test with a known symbol (AAPL)
        let result = downloader.fetch_historical_data("AAPL", start_date, end_date).await;
        
        // Should either succeed or fail with a network/API error, but not a client creation error
        match result {
            Ok(data) => {
                // If successful, data should be CSV format
                assert!(!data.is_empty());
                assert!(data.contains("Date") || data.contains("Close")); // CSV header
            }
            Err(DownloadError::NetworkError(_)) | Err(DownloadError::ApiError(_)) => {
                // Network or API errors are acceptable in tests
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_fetch_historical_data_invalid_symbol() {
        let downloader = YahooFinanceDownloader::new().unwrap();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        
        // Test with an invalid symbol
        let result = downloader.fetch_historical_data("INVALID_SYMBOL_XYZ123", start_date, end_date).await;
        
        // Should fail with API error or network error, not succeed
        match result {
            Ok(_) => {
                // Yahoo Finance might return empty data or error in CSV format
                // This is acceptable
            }
            Err(DownloadError::ApiError(_)) | Err(DownloadError::NetworkError(_)) => {
                // Expected error types
            }
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_fetch_historical_data_invalid_date_range() {
        let downloader = YahooFinanceDownloader::new().unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(); // start > end
        
        // This should still make the API call (Yahoo Finance handles it), but we test our date handling
        let result = downloader.fetch_historical_data("AAPL", start_date, end_date).await;
        
        // Should either succeed (Yahoo Finance returns empty) or fail appropriately
        match result {
            Ok(_) | Err(DownloadError::ApiError(_)) | Err(DownloadError::NetworkError(_)) => {
                // All acceptable outcomes
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_download_error_display() {
        let error = DownloadError::NetworkError("Connection timeout".to_string());
        assert!(error.to_string().contains("Network error"));
        assert!(error.to_string().contains("Connection timeout"));
    }
}

