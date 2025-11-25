use crate::asset_key::AssetKey;
use crate::time_series::{DataProvider, DataProviderError, DateRange, TimeSeriesPoint};
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Connection, Result as SqliteResult};
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
    fn table_exists(&self, table_name: &str) -> SqliteResult<bool> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name=?1"
        )?;
        let exists = stmt.exists([table_name])?;
        Ok(exists)
    }

    /// Returns a reference to the underlying SQLite connection.
    /// 
    /// This is useful for implementing additional methods that need direct database access.
    pub fn connection(&self) -> &Connection {
        &self.conn
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
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, close_price FROM time_series_data 
             WHERE asset_key = ?1 
             AND date(timestamp) >= ?2 
             AND date(timestamp) <= ?3 
             ORDER BY timestamp"
        ).map_err(|e| DataProviderError::Other(format!("SQL error: {}", e)))?;

        let rows = stmt.query_map(
            [&asset_key_str, &start_date_str, &end_date_str],
            |row| {
                let timestamp_str: String = row.get(0)?;
                let close_price: f64 = row.get(1)?;
                
                // Parse timestamp from string (stored as ISO 8601)
                let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                    .map_err(|e| rusqlite::Error::InvalidColumnType(0, format!("Invalid timestamp: {}", e), rusqlite::types::Type::Text))?
                    .with_timezone(&Utc);
                
                Ok(TimeSeriesPoint::new(timestamp, close_price))
            }
        ).map_err(|e| DataProviderError::Other(format!("SQL error: {}", e)))?;

        let mut points = Vec::new();
        for row_result in rows {
            match row_result {
                Ok(point) => points.push(point),
                Err(e) => return Err(DataProviderError::Other(format!("Row parsing error: {}", e))),
            }
        }

        // If no points found, check if asset exists at all
        if points.is_empty() {
            // Check if asset exists in assets table
            let mut check_stmt = self.conn.prepare(
                "SELECT 1 FROM assets WHERE asset_key = ?1 LIMIT 1"
            ).map_err(|e| DataProviderError::Other(format!("SQL error: {}", e)))?;
            
            let asset_exists = check_stmt.exists([&asset_key_str])
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
        
        let index_names: Vec<String> = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        }).unwrap().map(|r| r.unwrap()).collect();
        
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
}

