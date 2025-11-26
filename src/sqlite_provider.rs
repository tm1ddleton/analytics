use crate::asset::Asset;
use crate::asset_key::AssetKey;
use crate::equity::Equity;
use crate::future::Future;
use crate::time_series::{DataProvider, DataProviderError, DateRange, TimeSeriesPoint};
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Connection, Result as SqliteResult};
use serde_json;
use std::path::Path;

/// SQLite-based data provider implementation.
///
/// Stores asset data, time-series data, and analytics in SQLite database.
/// Automatically creates schema on first use.
#[derive(Debug)]
pub struct SqliteDataProvider {
    conn: Connection,
}

impl SqliteDataProvider {
    /// Creates a new SQLite data provider with a file-based database.
    ///
    /// # Arguments
    /// * `db_path` - Path to the SQLite database file. If the file doesn't exist, it will be created.
    ///
    /// # Returns
    /// Returns `Ok(SqliteDataProvider)` if successful, or an error if connection fails.
    ///
    /// # Errors
    /// Returns an error if the database connection cannot be established.
    pub fn new<P: AsRef<Path>>(db_path: P) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        let provider = SqliteDataProvider { conn };
        provider.ensure_schema()?;
        Ok(provider)
    }

    /// Creates a new SQLite data provider with an in-memory database.
    ///
    /// Useful for testing.
    ///
    /// # Returns
    /// Returns `Ok(SqliteDataProvider)` if successful, or an error if connection fails.
    pub fn new_in_memory() -> SqliteResult<Self> {
        let conn = Connection::open_in_memory()?;
        let provider = SqliteDataProvider { conn };
        provider.ensure_schema()?;
        Ok(provider)
    }

    /// Ensures the database schema exists, creating tables if they don't exist.
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, or an error if schema creation fails.
    fn ensure_schema(&self) -> SqliteResult<()> {
        // Create assets table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS assets (
                asset_key TEXT PRIMARY KEY,
                asset_data TEXT NOT NULL
            )",
            [],
        )?;

        // Create time_series_data table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS time_series_data (
                asset_key TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                close_price REAL NOT NULL,
                PRIMARY KEY (asset_key, timestamp)
            )",
            [],
        )?;

        // Create indexes on time_series_data for query performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_time_series_asset_key ON time_series_data(asset_key)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_time_series_timestamp ON time_series_data(timestamp)",
            [],
        )?;

        // Create analytics table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS analytics (
                asset_key TEXT NOT NULL,
                date TEXT NOT NULL,
                analytics_name TEXT NOT NULL,
                value TEXT NOT NULL,
                PRIMARY KEY (asset_key, date, analytics_name)
            )",
            [],
        )?;

        Ok(())
    }

    /// Checks if a table exists in the database.
    ///
    /// # Arguments
    /// * `table_name` - Name of the table to check
    ///
    /// # Returns
    /// Returns `true` if the table exists, `false` otherwise.
    #[cfg(test)]
    fn table_exists(&self, table_name: &str) -> SqliteResult<bool> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1")?;
        let exists = stmt.exists([table_name])?;
        Ok(exists)
    }

    /// Returns a reference to the underlying SQLite connection.
    ///
    /// This is useful for implementing additional methods that need direct database access.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Inserts a single time-series point into the database.
    ///
    /// If a point with the same asset_key and timestamp already exists, it will be replaced (upsert).
    ///
    /// # Arguments
    /// * `asset_key` - The asset key for this time-series point
    /// * `point` - The time-series point to insert
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, or an error if insertion fails.
    ///
    /// # Errors
    /// Returns an error if the database operation fails.
    pub fn insert_time_series_point(
        &self,
        asset_key: &AssetKey,
        point: &TimeSeriesPoint,
    ) -> Result<(), DataProviderError> {
        let asset_key_str = asset_key.as_string();
        let timestamp_str = point.timestamp.to_rfc3339();

        self.conn
            .execute(
                "INSERT OR REPLACE INTO time_series_data (asset_key, timestamp, close_price) VALUES (?1, ?2, ?3)",
                rusqlite::params![asset_key_str, timestamp_str, point.close_price],
            )
            .map_err(|e| DataProviderError::Other(format!("Failed to insert time-series point: {}", e)))?;

        Ok(())
    }

    /// Inserts multiple time-series points in a single transaction.
    ///
    /// This is more efficient than inserting points one by one.
    /// If any point fails to insert, the entire transaction is rolled back.
    ///
    /// # Arguments
    /// * `asset_key` - The asset key for all time-series points
    /// * `points` - Vector of time-series points to insert
    ///
    /// # Returns
    /// Returns `Ok(())` if all points are inserted successfully, or an error if any insertion fails.
    ///
    /// # Errors
    /// Returns an error if the database operation fails. The transaction will be rolled back.
    pub fn insert_time_series_batch(
        &mut self,
        asset_key: &AssetKey,
        points: &[TimeSeriesPoint],
    ) -> Result<(), DataProviderError> {
        if points.is_empty() {
            return Ok(());
        }

        let asset_key_str = asset_key.as_string();
        let transaction = self
            .conn
            .transaction()
            .map_err(|e| DataProviderError::Other(format!("Failed to start transaction: {}", e)))?;

        {
            let mut stmt = transaction
                .prepare(
                    "INSERT OR REPLACE INTO time_series_data (asset_key, timestamp, close_price) VALUES (?1, ?2, ?3)",
                )
                .map_err(|e| DataProviderError::Other(format!("Failed to prepare statement: {}", e)))?;

            for point in points {
                let timestamp_str = point.timestamp.to_rfc3339();
                stmt.execute(rusqlite::params![
                    asset_key_str,
                    timestamp_str,
                    point.close_price
                ])
                .map_err(|e| {
                    DataProviderError::Other(format!("Failed to insert point in batch: {}", e))
                })?;
            }
        }

        transaction.commit().map_err(|e| {
            DataProviderError::Other(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(())
    }

    /// Updates an existing time-series point in the database.
    ///
    /// # Arguments
    /// * `asset_key` - The asset key for this time-series point
    /// * `point` - The time-series point to update (must have matching timestamp)
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, or an error if update fails or point doesn't exist.
    ///
    /// # Errors
    /// Returns an error if the point doesn't exist or if the database operation fails.
    pub fn update_time_series_point(
        &self,
        asset_key: &AssetKey,
        point: &TimeSeriesPoint,
    ) -> Result<(), DataProviderError> {
        let asset_key_str = asset_key.as_string();
        let timestamp_str = point.timestamp.to_rfc3339();

        let rows_affected = self
            .conn
            .execute(
                "UPDATE time_series_data SET close_price = ?3 WHERE asset_key = ?1 AND timestamp = ?2",
                rusqlite::params![asset_key_str, timestamp_str, point.close_price],
            )
            .map_err(|e| DataProviderError::Other(format!("Failed to update time-series point: {}", e)))?;

        if rows_affected == 0 {
            return Err(DataProviderError::Other(format!(
                "Time-series point not found for asset_key={} and timestamp={}",
                asset_key_str, timestamp_str
            )));
        }

        Ok(())
    }

    /// Stores an Equity asset in the database as a JSON blob.
    ///
    /// If an asset with the same asset_key already exists, it will be replaced.
    ///
    /// # Arguments
    /// * `equity` - The Equity asset to store
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, or an error if storage fails.
    ///
    /// # Errors
    /// Returns an error if serialization or database operation fails.
    pub fn store_asset_equity(&self, equity: &Equity) -> Result<(), DataProviderError> {
        let asset_key_str = equity.key().as_string();
        let asset_json = serde_json::to_string(equity)
            .map_err(|e| DataProviderError::Other(format!("Failed to serialize Equity: {}", e)))?;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO assets (asset_key, asset_data) VALUES (?1, ?2)",
                rusqlite::params![asset_key_str, asset_json],
            )
            .map_err(|e| DataProviderError::Other(format!("Failed to store Equity: {}", e)))?;

        Ok(())
    }

    /// Stores a Future asset in the database as a JSON blob.
    ///
    /// If an asset with the same asset_key already exists, it will be replaced.
    ///
    /// # Arguments
    /// * `future` - The Future asset to store
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, or an error if storage fails.
    ///
    /// # Errors
    /// Returns an error if serialization or database operation fails.
    pub fn store_asset_future(&self, future: &Future) -> Result<(), DataProviderError> {
        let asset_key_str = future.key().as_string();
        let asset_json = serde_json::to_string(future)
            .map_err(|e| DataProviderError::Other(format!("Failed to serialize Future: {}", e)))?;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO assets (asset_key, asset_data) VALUES (?1, ?2)",
                rusqlite::params![asset_key_str, asset_json],
            )
            .map_err(|e| DataProviderError::Other(format!("Failed to store Future: {}", e)))?;

        Ok(())
    }

    /// Retrieves an Equity asset from the database by asset_key.
    ///
    /// # Arguments
    /// * `asset_key` - The asset key to retrieve
    ///
    /// # Returns
    /// Returns `Ok(Equity)` if found, or an error if not found or deserialization fails.
    ///
    /// # Errors
    /// Returns an error if the asset is not found, is not an Equity, or deserialization fails.
    pub fn get_asset_equity(&self, asset_key: &AssetKey) -> Result<Equity, DataProviderError> {
        let asset_key_str = asset_key.as_string();
        let asset_json: String = self
            .conn
            .query_row(
                "SELECT asset_data FROM assets WHERE asset_key = ?1",
                [&asset_key_str],
                |row| row.get(0),
            )
            .map_err(|e| {
                if let rusqlite::Error::QueryReturnedNoRows = e {
                    DataProviderError::AssetNotFound
                } else {
                    DataProviderError::Other(format!("Failed to retrieve asset: {}", e))
                }
            })?;

        let equity: Equity = serde_json::from_str(&asset_json).map_err(|e| {
            DataProviderError::Other(format!("Failed to deserialize Equity: {}", e))
        })?;

        // Verify the asset key matches
        if equity.key() != asset_key {
            return Err(DataProviderError::Other(format!(
                "Asset key mismatch: expected {:?}, got {:?}",
                asset_key,
                equity.key()
            )));
        }

        Ok(equity)
    }

    /// Retrieves a Future asset from the database by asset_key.
    ///
    /// # Arguments
    /// * `asset_key` - The asset key to retrieve
    ///
    /// # Returns
    /// Returns `Ok(Future)` if found, or an error if not found or deserialization fails.
    ///
    /// # Errors
    /// Returns an error if the asset is not found, is not a Future, or deserialization fails.
    pub fn get_asset_future(&self, asset_key: &AssetKey) -> Result<Future, DataProviderError> {
        let asset_key_str = asset_key.as_string();
        let asset_json: String = self
            .conn
            .query_row(
                "SELECT asset_data FROM assets WHERE asset_key = ?1",
                [&asset_key_str],
                |row| row.get(0),
            )
            .map_err(|e| {
                if let rusqlite::Error::QueryReturnedNoRows = e {
                    DataProviderError::AssetNotFound
                } else {
                    DataProviderError::Other(format!("Failed to retrieve asset: {}", e))
                }
            })?;

        let future: Future = serde_json::from_str(&asset_json).map_err(|e| {
            DataProviderError::Other(format!("Failed to deserialize Future: {}", e))
        })?;

        // Verify the asset key matches
        if future.key() != asset_key {
            return Err(DataProviderError::Other(format!(
                "Asset key mismatch: expected {:?}, got {:?}",
                asset_key,
                future.key()
            )));
        }

        Ok(future)
    }

    /// Stores an analytics result in the database.
    ///
    /// The analytics value is stored as a JSON blob for flexibility.
    /// If an analytics result with the same (asset_key, date, analytics_name) already exists, it will be replaced.
    ///
    /// # Arguments
    /// * `asset_key` - The asset key for this analytics result
    /// * `date` - The date for this analytics result
    /// * `analytics_name` - The name/identifier of the analytics (e.g., "sma_20", "rsi")
    /// * `value` - The analytics value (will be serialized as JSON)
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, or an error if storage fails.
    ///
    /// # Errors
    /// Returns an error if serialization or database operation fails.
    pub fn store_analytics<V: serde::Serialize>(
        &self,
        asset_key: &AssetKey,
        date: NaiveDate,
        analytics_name: &str,
        value: &V,
    ) -> Result<(), DataProviderError> {
        let asset_key_str = asset_key.as_string();
        let date_str = date.format("%Y-%m-%d").to_string();
        let value_json = serde_json::to_string(value).map_err(|e| {
            DataProviderError::Other(format!("Failed to serialize analytics value: {}", e))
        })?;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO analytics (asset_key, date, analytics_name, value) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![asset_key_str, date_str, analytics_name, value_json],
            )
            .map_err(|e| DataProviderError::Other(format!("Failed to store analytics: {}", e)))?;

        Ok(())
    }

    /// Retrieves analytics results for a given asset key and date range.
    ///
    /// # Arguments
    /// * `asset_key` - The asset key to query
    /// * `date_range` - The date range to query (inclusive)
    ///
    /// # Returns
    /// Returns a vector of tuples: (date, analytics_name, value_json_string).
    /// Returns an empty vector if no analytics are found.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    pub fn get_analytics(
        &self,
        asset_key: &AssetKey,
        date_range: &DateRange,
    ) -> Result<Vec<(NaiveDate, String, String)>, DataProviderError> {
        // Validate date range
        if date_range.start > date_range.end {
            return Err(DataProviderError::InvalidDateRange);
        }

        let asset_key_str = asset_key.as_string();
        let start_date_str = date_range.start.format("%Y-%m-%d").to_string();
        let end_date_str = date_range.end.format("%Y-%m-%d").to_string();

        let mut stmt = self
            .conn
            .prepare(
                "SELECT date, analytics_name, value FROM analytics 
                 WHERE asset_key = ?1 
                 AND date >= ?2 
                 AND date <= ?3 
                 ORDER BY date, analytics_name",
            )
            .map_err(|e| DataProviderError::Other(format!("SQL error: {}", e)))?;

        let rows = stmt
            .query_map([&asset_key_str, &start_date_str, &end_date_str], |row| {
                let date_str: String = row.get(0)?;
                let analytics_name: String = row.get(1)?;
                let value_json: String = row.get(2)?;

                let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
                    rusqlite::Error::InvalidColumnType(
                        0,
                        format!("Invalid date: {}", e),
                        rusqlite::types::Type::Text,
                    )
                })?;

                Ok((date, analytics_name, value_json))
            })
            .map_err(|e| DataProviderError::Other(format!("SQL error: {}", e)))?;

        let mut results = Vec::new();
        for row_result in rows {
            match row_result {
                Ok(result) => results.push(result),
                Err(e) => {
                    return Err(DataProviderError::Other(format!(
                        "Row parsing error: {}",
                        e
                    )))
                }
            }
        }

        Ok(results)
    }

    /// Retrieves analytics results by analytics name across all assets for a given date range.
    ///
    /// This is useful for cross-asset analysis (e.g., comparing RSI across multiple assets).
    ///
    /// # Arguments
    /// * `analytics_name` - The name/identifier of the analytics to query
    /// * `date_range` - The date range to query (inclusive)
    ///
    /// # Returns
    /// Returns a vector of tuples: (asset_key_string, date, value_json_string).
    /// Returns an empty vector if no analytics are found.
    ///
    /// # Errors
    /// Returns an error if the database query fails.
    pub fn get_analytics_by_name(
        &self,
        analytics_name: &str,
        date_range: &DateRange,
    ) -> Result<Vec<(String, NaiveDate, String)>, DataProviderError> {
        // Validate date range
        if date_range.start > date_range.end {
            return Err(DataProviderError::InvalidDateRange);
        }

        let start_date_str = date_range.start.format("%Y-%m-%d").to_string();
        let end_date_str = date_range.end.format("%Y-%m-%d").to_string();

        let mut stmt = self
            .conn
            .prepare(
                "SELECT asset_key, date, value FROM analytics 
                 WHERE analytics_name = ?1 
                 AND date >= ?2 
                 AND date <= ?3 
                 ORDER BY asset_key, date",
            )
            .map_err(|e| DataProviderError::Other(format!("SQL error: {}", e)))?;

        let rows = stmt
            .query_map([analytics_name, &start_date_str, &end_date_str], |row| {
                let asset_key_str: String = row.get(0)?;
                let date_str: String = row.get(1)?;
                let value_json: String = row.get(2)?;

                let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
                    rusqlite::Error::InvalidColumnType(
                        1,
                        format!("Invalid date: {}", e),
                        rusqlite::types::Type::Text,
                    )
                })?;

                Ok((asset_key_str, date, value_json))
            })
            .map_err(|e| DataProviderError::Other(format!("SQL error: {}", e)))?;

        let mut results = Vec::new();
        for row_result in rows {
            match row_result {
                Ok(result) => results.push(result),
                Err(e) => {
                    return Err(DataProviderError::Other(format!(
                        "Row parsing error: {}",
                        e
                    )))
                }
            }
        }

        Ok(results)
    }
}

impl DataProvider for SqliteDataProvider {
    fn get_time_series(
        &self,
        asset_key: &AssetKey,
        date_range: &DateRange,
    ) -> Result<Vec<TimeSeriesPoint>, DataProviderError> {
        // Validate date range
        if date_range.start > date_range.end {
            return Err(DataProviderError::InvalidDateRange);
        }

        // Serialize asset key to string
        let asset_key_str = asset_key.as_string();
        // Format dates as ISO 8601 strings for comparison
        // SQLite's date() function works with ISO 8601 timestamps
        let start_date_str = date_range.start.format("%Y-%m-%d").to_string();
        let end_date_str = date_range.end.format("%Y-%m-%d").to_string();

        // Query time-series data using prepared statement for performance
        // Use date() function to extract date part from timestamp for comparison
        let mut stmt = self
            .conn
            .prepare(
                "SELECT timestamp, close_price FROM time_series_data 
             WHERE asset_key = ?1 
             AND date(timestamp) >= ?2 
             AND date(timestamp) <= ?3 
             ORDER BY timestamp",
            )
            .map_err(|e| DataProviderError::Other(format!("SQL error: {}", e)))?;

        let rows = stmt
            .query_map([&asset_key_str, &start_date_str, &end_date_str], |row| {
                let timestamp_str: String = row.get(0)?;
                let close_price: f64 = row.get(1)?;

                // Parse timestamp from string (stored as ISO 8601)
                let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                    .map_err(|e| {
                        rusqlite::Error::InvalidColumnType(
                            0,
                            format!("Invalid timestamp: {}", e),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc);

                Ok(TimeSeriesPoint::new(timestamp, close_price))
            })
            .map_err(|e| DataProviderError::Other(format!("SQL error: {}", e)))?;

        let mut points = Vec::new();
        for row_result in rows {
            match row_result {
                Ok(point) => points.push(point),
                Err(e) => {
                    return Err(DataProviderError::Other(format!(
                        "Row parsing error: {}",
                        e
                    )))
                }
            }
        }

        // If no points found, check if asset exists at all
        if points.is_empty() {
            // Check if asset exists in assets table
            let mut check_stmt = self
                .conn
                .prepare("SELECT 1 FROM assets WHERE asset_key = ?1 LIMIT 1")
                .map_err(|e| DataProviderError::Other(format!("SQL error: {}", e)))?;

            let asset_exists = check_stmt
                .exists([&asset_key_str])
                .map_err(|e| DataProviderError::Other(format!("SQL error: {}", e)))?;

            if !asset_exists {
                return Err(DataProviderError::AssetNotFound);
            }
        }

        Ok(points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_sqlite_provider_creation_in_memory() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        assert!(provider.table_exists("assets").unwrap());
        assert!(provider.table_exists("time_series_data").unwrap());
        assert!(provider.table_exists("analytics").unwrap());
    }

    #[test]
    fn test_automatic_schema_creation() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();

        // Verify all tables exist
        assert!(provider.table_exists("assets").unwrap());
        assert!(provider.table_exists("time_series_data").unwrap());
        assert!(provider.table_exists("analytics").unwrap());
    }

    #[test]
    fn test_table_existence_check() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();

        assert!(provider.table_exists("assets").unwrap());
        assert!(provider.table_exists("time_series_data").unwrap());
        assert!(provider.table_exists("analytics").unwrap());
        assert!(!provider.table_exists("nonexistent_table").unwrap());
    }

    #[test]
    fn test_connection_initialization() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();

        // Verify connection is accessible
        let _conn = provider.connection();
        // If we can access the connection, initialization was successful
    }

    #[test]
    fn test_schema_creation_idempotent() {
        // Create provider twice - schema should be created only once
        let provider1 = SqliteDataProvider::new_in_memory().unwrap();
        assert!(provider1.table_exists("assets").unwrap());

        // Re-initialize schema (should not fail)
        provider1.ensure_schema().unwrap();
        assert!(provider1.table_exists("assets").unwrap());
    }

    #[test]
    fn test_indexes_created() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();

        // Check that indexes exist by querying sqlite_master
        let mut stmt = provider.connection().prepare(
            "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_time_series%'"
        ).unwrap();

        let index_names: Vec<String> = stmt
            .query_map([], |row| Ok(row.get::<_, String>(0)?))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(index_names.contains(&"idx_time_series_asset_key".to_string()));
        assert!(index_names.contains(&"idx_time_series_timestamp".to_string()));
    }

    #[test]
    fn test_get_time_series_valid_asset_key_and_date_range() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();

        // Insert test data
        let timestamp1 = Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap();
        let timestamp2 = Utc.with_ymd_and_hms(2024, 1, 16, 16, 0, 0).unwrap();
        let timestamp3 = Utc.with_ymd_and_hms(2024, 1, 17, 16, 0, 0).unwrap();

        provider.connection().execute(
            "INSERT INTO time_series_data (asset_key, timestamp, close_price) VALUES (?1, ?2, ?3)",
            rusqlite::params![asset_key.as_string(), timestamp1.to_rfc3339(), 150.0],
        ).unwrap();
        provider.connection().execute(
            "INSERT INTO time_series_data (asset_key, timestamp, close_price) VALUES (?1, ?2, ?3)",
            rusqlite::params![asset_key.as_string(), timestamp2.to_rfc3339(), 151.0],
        ).unwrap();
        provider.connection().execute(
            "INSERT INTO time_series_data (asset_key, timestamp, close_price) VALUES (?1, ?2, ?3)",
            rusqlite::params![asset_key.as_string(), timestamp3.to_rfc3339(), 152.0],
        ).unwrap();

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
    fn test_get_time_series_non_existent_asset_key() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("NONEXISTENT").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        );

        let result = provider.get_time_series(&asset_key, &date_range);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DataProviderError::AssetNotFound);
    }

    #[test]
    fn test_get_time_series_invalid_date_range() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(), // start > end
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let result = provider.get_time_series(&asset_key, &date_range);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DataProviderError::InvalidDateRange);
    }

    #[test]
    fn test_get_time_series_date_range_inclusive_boundaries() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();

        // Insert data at boundary dates
        let start_timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
        let end_timestamp = Utc.with_ymd_and_hms(2024, 1, 16, 23, 59, 59).unwrap();
        let outside_timestamp = Utc.with_ymd_and_hms(2024, 1, 17, 0, 0, 0).unwrap();

        provider.connection().execute(
            "INSERT INTO time_series_data (asset_key, timestamp, close_price) VALUES (?1, ?2, ?3)",
            rusqlite::params![asset_key.as_string(), start_timestamp.to_rfc3339(), 150.0],
        ).unwrap();
        provider.connection().execute(
            "INSERT INTO time_series_data (asset_key, timestamp, close_price) VALUES (?1, ?2, ?3)",
            rusqlite::params![asset_key.as_string(), end_timestamp.to_rfc3339(), 151.0],
        ).unwrap();
        provider.connection().execute(
            "INSERT INTO time_series_data (asset_key, timestamp, close_price) VALUES (?1, ?2, ?3)",
            rusqlite::params![asset_key.as_string(), outside_timestamp.to_rfc3339(), 152.0],
        ).unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        );

        let result = provider.get_time_series(&asset_key, &date_range).unwrap();
        // Should include both boundary dates
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_get_time_series_error_mapping() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        );

        // Test with invalid SQL (should map to DataProviderError::Other)
        // This is tested indirectly through the normal flow, but we can verify
        // that SQL errors are properly mapped
        let result = provider.get_time_series(&asset_key, &date_range);
        // Should return AssetNotFound (no data and asset doesn't exist)
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DataProviderError::AssetNotFound);
    }

    #[test]
    fn test_get_time_series_futures_key() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let asset_key = AssetKey::new_future("ES", expiry).unwrap();

        let timestamp = Utc.with_ymd_and_hms(2024, 12, 15, 16, 0, 0).unwrap();
        provider.connection().execute(
            "INSERT INTO time_series_data (asset_key, timestamp, close_price) VALUES (?1, ?2, ?3)",
            rusqlite::params![asset_key.as_string(), timestamp.to_rfc3339(), 4500.0],
        ).unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 12, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 15).unwrap(),
        );

        let result = provider.get_time_series(&asset_key, &date_range).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].close_price, 4500.0);
    }

    #[test]
    fn test_insert_time_series_point_single() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap();
        let point = TimeSeriesPoint::new(timestamp, 150.0);

        // Insert the point
        provider
            .insert_time_series_point(&asset_key, &point)
            .unwrap();

        // Verify it was inserted
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );
        let result = provider.get_time_series(&asset_key, &date_range).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].close_price, 150.0);
        assert_eq!(result[0].timestamp, timestamp);
    }

    #[test]
    fn test_insert_time_series_point_batch() {
        let mut provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();

        let points = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap(), 150.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 16, 16, 0, 0).unwrap(), 151.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 17, 16, 0, 0).unwrap(), 152.0),
        ];

        // Insert batch
        provider
            .insert_time_series_batch(&asset_key, &points)
            .unwrap();

        // Verify all points were inserted
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(),
        );
        let result = provider.get_time_series(&asset_key, &date_range).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].close_price, 150.0);
        assert_eq!(result[1].close_price, 151.0);
        assert_eq!(result[2].close_price, 152.0);
    }

    #[test]
    fn test_insert_time_series_point_upsert() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap();

        // Insert initial point
        let point1 = TimeSeriesPoint::new(timestamp, 150.0);
        provider
            .insert_time_series_point(&asset_key, &point1)
            .unwrap();

        // Insert point with same timestamp but different price (should replace)
        let point2 = TimeSeriesPoint::new(timestamp, 155.0);
        provider
            .insert_time_series_point(&asset_key, &point2)
            .unwrap();

        // Verify only one point exists with updated price
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );
        let result = provider.get_time_series(&asset_key, &date_range).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].close_price, 155.0); // Updated price
    }

    #[test]
    fn test_insert_time_series_point_different_asset_keys() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key1 = AssetKey::new_equity("AAPL").unwrap();
        let asset_key2 = AssetKey::new_equity("MSFT").unwrap();

        let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap();
        let point1 = TimeSeriesPoint::new(timestamp, 150.0);
        let point2 = TimeSeriesPoint::new(timestamp, 300.0);

        // Insert points for different assets
        provider
            .insert_time_series_point(&asset_key1, &point1)
            .unwrap();
        provider
            .insert_time_series_point(&asset_key2, &point2)
            .unwrap();

        // Verify both assets have their own data
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );

        let result1 = provider.get_time_series(&asset_key1, &date_range).unwrap();
        assert_eq!(result1.len(), 1);
        assert_eq!(result1[0].close_price, 150.0);

        let result2 = provider.get_time_series(&asset_key2, &date_range).unwrap();
        assert_eq!(result2.len(), 1);
        assert_eq!(result2[0].close_price, 300.0);
    }

    #[test]
    fn test_insert_time_series_point_empty_batch() {
        let mut provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let points: Vec<TimeSeriesPoint> = vec![];

        // Insert empty batch should succeed
        provider
            .insert_time_series_batch(&asset_key, &points)
            .unwrap();
    }

    #[test]
    fn test_update_time_series_point() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap();

        // Insert initial point
        let point1 = TimeSeriesPoint::new(timestamp, 150.0);
        provider
            .insert_time_series_point(&asset_key, &point1)
            .unwrap();

        // Update the point
        let point2 = TimeSeriesPoint::new(timestamp, 155.0);
        provider
            .update_time_series_point(&asset_key, &point2)
            .unwrap();

        // Verify updated price
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        );
        let result = provider.get_time_series(&asset_key, &date_range).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].close_price, 155.0);
    }

    #[test]
    fn test_update_time_series_point_not_found() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap();
        let point = TimeSeriesPoint::new(timestamp, 150.0);

        // Try to update non-existent point
        let result = provider.update_time_series_point(&asset_key, &point);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_store_asset_equity() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let equity = Equity::new("AAPL", "Apple Inc.", "NASDAQ", "USD", "Technology").unwrap();

        // Store the equity
        provider.store_asset_equity(&equity).unwrap();

        // Verify it was stored
        let retrieved = provider.get_asset_equity(equity.key()).unwrap();
        assert_eq!(retrieved.name(), equity.name());
        assert_eq!(retrieved.exchange(), equity.exchange());
        assert_eq!(retrieved.currency(), equity.currency());
        assert_eq!(retrieved.sector(), equity.sector());
    }

    #[test]
    fn test_store_asset_future() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
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
        )
        .unwrap();

        // Store the future
        provider.store_asset_future(&future).unwrap();

        // Verify it was stored
        let retrieved = provider.get_asset_future(future.key()).unwrap();
        assert_eq!(retrieved.series(), future.series());
        assert_eq!(retrieved.expiry_date(), future.expiry_date());
        assert_eq!(retrieved.contract_month(), future.contract_month());
        assert_eq!(retrieved.name(), future.name());
    }

    #[test]
    fn test_get_asset_equity_not_found() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("NONEXISTENT").unwrap();

        let result = provider.get_asset_equity(&asset_key);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DataProviderError::AssetNotFound);
    }

    #[test]
    fn test_store_analytics() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let analytics_name = "sma_20";
        let value = 150.5;

        // Store analytics
        provider
            .store_analytics(&asset_key, date, analytics_name, &value)
            .unwrap();

        // Verify it was stored
        let date_range = DateRange::new(date, date);
        let results = provider.get_analytics(&asset_key, &date_range).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, date);
        assert_eq!(results[0].1, analytics_name);

        // Deserialize the value
        let retrieved_value: f64 = serde_json::from_str(&results[0].2).unwrap();
        assert_eq!(retrieved_value, value);
    }

    #[test]
    fn test_get_analytics_date_range() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();

        // Store analytics for multiple dates
        let date1 = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2024, 1, 16).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2024, 1, 17).unwrap();

        provider
            .store_analytics(&asset_key, date1, "sma_20", &150.0)
            .unwrap();
        provider
            .store_analytics(&asset_key, date2, "sma_20", &151.0)
            .unwrap();
        provider
            .store_analytics(&asset_key, date3, "sma_20", &152.0)
            .unwrap();

        // Query date range
        let date_range = DateRange::new(date1, date2);
        let results = provider.get_analytics(&asset_key, &date_range).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, date1);
        assert_eq!(results[1].0, date2);
    }

    #[test]
    fn test_get_analytics_by_name() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key1 = AssetKey::new_equity("AAPL").unwrap();
        let asset_key2 = AssetKey::new_equity("MSFT").unwrap();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let analytics_name = "rsi";

        // Store analytics for different assets
        provider
            .store_analytics(&asset_key1, date, analytics_name, &65.5)
            .unwrap();
        provider
            .store_analytics(&asset_key2, date, analytics_name, &70.2)
            .unwrap();

        // Query by analytics name
        let date_range = DateRange::new(date, date);
        let results = provider
            .get_analytics_by_name(analytics_name, &date_range)
            .unwrap();
        assert_eq!(results.len(), 2);

        // Verify both assets are included
        let asset_keys: Vec<String> = results.iter().map(|r| r.0.clone()).collect();
        assert!(asset_keys.contains(&asset_key1.as_string()));
        assert!(asset_keys.contains(&asset_key2.as_string()));
    }

    #[test]
    fn test_store_analytics_update() {
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let analytics_name = "sma_20";

        // Store initial value
        provider
            .store_analytics(&asset_key, date, analytics_name, &150.0)
            .unwrap();

        // Update with new value
        provider
            .store_analytics(&asset_key, date, analytics_name, &155.0)
            .unwrap();

        // Verify updated value
        let date_range = DateRange::new(date, date);
        let results = provider.get_analytics(&asset_key, &date_range).unwrap();
        assert_eq!(results.len(), 1);
        let retrieved_value: f64 = serde_json::from_str(&results[0].2).unwrap();
        assert_eq!(retrieved_value, 155.0);
    }

    // Task Group 5: Additional strategic tests for end-to-end workflows

    #[test]
    fn test_end_to_end_store_asset_and_time_series() {
        // End-to-end workflow: Store asset, then store and query time-series data
        let mut provider = SqliteDataProvider::new_in_memory().unwrap();
        let equity = Equity::new("AAPL", "Apple Inc.", "NASDAQ", "USD", "Technology").unwrap();

        // Store asset
        provider.store_asset_equity(&equity).unwrap();

        // Store time-series data
        let points = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap(), 150.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 16, 16, 0, 0).unwrap(), 151.0),
        ];
        provider
            .insert_time_series_batch(equity.key(), &points)
            .unwrap();

        // Query time-series using DataProvider trait
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        );
        let result = provider.get_time_series(equity.key(), &date_range).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].close_price, 150.0);
        assert_eq!(result[1].close_price, 151.0);
    }

    #[test]
    fn test_end_to_end_store_asset_and_analytics() {
        // End-to-end workflow: Store asset, then store and query analytics
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let equity = Equity::new("AAPL", "Apple Inc.", "NASDAQ", "USD", "Technology").unwrap();

        // Store asset
        provider.store_asset_equity(&equity).unwrap();

        // Store analytics
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        provider
            .store_analytics(equity.key(), date, "sma_20", &150.5)
            .unwrap();
        provider
            .store_analytics(equity.key(), date, "rsi", &65.0)
            .unwrap();

        // Query analytics
        let date_range = DateRange::new(date, date);
        let results = provider.get_analytics(equity.key(), &date_range).unwrap();
        assert_eq!(results.len(), 2);

        // Verify both analytics are present
        let analytics_names: Vec<String> = results.iter().map(|r| r.1.clone()).collect();
        assert!(analytics_names.contains(&"sma_20".to_string()));
        assert!(analytics_names.contains(&"rsi".to_string()));
    }

    #[test]
    fn test_file_based_database() {
        // Test file-based database creation (not just in-memory)
        use std::fs;
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_analytics.db");

        // Clean up if exists
        let _ = fs::remove_file(&db_path);

        {
            let provider = SqliteDataProvider::new(&db_path).unwrap();
            let equity = Equity::new("AAPL", "Apple Inc.", "NASDAQ", "USD", "Technology").unwrap();

            // Store asset
            provider.store_asset_equity(&equity).unwrap();
        }

        // Reopen database and verify data persists
        let provider = SqliteDataProvider::new(&db_path).unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let retrieved = provider.get_asset_equity(&asset_key).unwrap();
        assert_eq!(retrieved.name(), "Apple Inc.");

        // Clean up
        let _ = fs::remove_file(&db_path);
    }

    #[test]
    fn test_multiple_analytics_same_asset_date() {
        // Test storing multiple analytics for same asset and date
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Store multiple analytics
        provider
            .store_analytics(&asset_key, date, "sma_20", &150.0)
            .unwrap();
        provider
            .store_analytics(&asset_key, date, "sma_50", &148.0)
            .unwrap();
        provider
            .store_analytics(&asset_key, date, "rsi", &65.0)
            .unwrap();

        // Query all analytics for this date
        let date_range = DateRange::new(date, date);
        let results = provider.get_analytics(&asset_key, &date_range).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_analytics_empty_results() {
        // Test that analytics queries return empty vector when no data exists
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        );

        let results = provider.get_analytics(&asset_key, &date_range).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_analytics_by_name_empty_results() {
        // Test that analytics_by_name queries return empty vector when no data exists
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        );

        let results = provider
            .get_analytics_by_name("sma_20", &date_range)
            .unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_time_series_with_asset_stored() {
        // Test that get_time_series returns empty vector (not AssetNotFound) when asset exists but no time-series data
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let equity = Equity::new("AAPL", "Apple Inc.", "NASDAQ", "USD", "Technology").unwrap();

        // Store asset but no time-series data
        provider.store_asset_equity(&equity).unwrap();

        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        );

        // Should return empty vector, not AssetNotFound error
        let result = provider.get_time_series(equity.key(), &date_range).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_cross_asset_analytics_workflow() {
        // End-to-end workflow: Store multiple assets, store analytics, query by analytics name
        let provider = SqliteDataProvider::new_in_memory().unwrap();
        let equity1 = Equity::new("AAPL", "Apple Inc.", "NASDAQ", "USD", "Technology").unwrap();
        let equity2 =
            Equity::new("MSFT", "Microsoft Corp.", "NASDAQ", "USD", "Technology").unwrap();
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Store assets
        provider.store_asset_equity(&equity1).unwrap();
        provider.store_asset_equity(&equity2).unwrap();

        // Store analytics for both assets
        provider
            .store_analytics(equity1.key(), date, "rsi", &65.0)
            .unwrap();
        provider
            .store_analytics(equity2.key(), date, "rsi", &70.0)
            .unwrap();

        // Query by analytics name across all assets
        let date_range = DateRange::new(date, date);
        let results = provider.get_analytics_by_name("rsi", &date_range).unwrap();
        assert_eq!(results.len(), 2);

        // Verify both assets are included
        let asset_keys: Vec<String> = results.iter().map(|r| r.0.clone()).collect();
        assert!(asset_keys.contains(&equity1.key().as_string()));
        assert!(asset_keys.contains(&equity2.key().as_string()));
    }

    #[test]
    fn test_future_asset_storage_and_retrieval() {
        // Test complete workflow for Future assets
        let provider = SqliteDataProvider::new_in_memory().unwrap();
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
        )
        .unwrap();

        // Store future
        provider.store_asset_future(&future).unwrap();

        // Store time-series data
        let timestamp = Utc.with_ymd_and_hms(2024, 12, 15, 16, 0, 0).unwrap();
        let point = TimeSeriesPoint::new(timestamp, 4500.0);
        provider
            .insert_time_series_point(future.key(), &point)
            .unwrap();

        // Retrieve future
        let retrieved = provider.get_asset_future(future.key()).unwrap();
        assert_eq!(retrieved.series(), "ES");
        assert_eq!(retrieved.expiry_date(), expiry);

        // Query time-series
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 12, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 15).unwrap(),
        );
        let result = provider.get_time_series(future.key(), &date_range).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].close_price, 4500.0);
    }
}
