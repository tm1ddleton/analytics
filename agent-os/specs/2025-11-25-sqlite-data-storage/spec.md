# Specification: SQLite Data Storage

## Goal
Implement simple SQLite-based storage for asset data and computed analytics with key-based and date-range query capabilities, providing a persistent DataProvider implementation that integrates seamlessly with the existing asset-centric architecture.

## User Stories
- As a quantitative trader, I want to store asset data and time-series prices in SQLite so that I can persist market data and query it efficiently for strategy development
- As a research analyst, I want to query historical time-series data by asset key and date range from SQLite so that I can analyze price patterns and compute analytics
- As a system developer, I want a SQLite DataProvider implementation so that the analytics engine can use persistent storage while maintaining the data-source-agnostic design

## Specific Requirements

**Database Schema Design**
- Create three separate tables: assets, time_series_data, and analytics
- Assets table stores full Equity and Future structs as JSON blobs with asset_key as primary key
- Time_series_data table stores individual rows per timestamp with columns: asset_key, timestamp, close_price
- Analytics table stores computed results with columns: asset_key, date, analytics_name, value (flexible JSON structure)
- Create indexes on asset_key and timestamp in time_series_data table for query performance
- Use SQLite TEXT type for asset_key (serialized from AssetKey enum)
- Use SQLite REAL type for close_price and numeric analytics values

**SQLite DataProvider Implementation**
- Implement DataProvider trait for SQLite backend
- Manage single database connection (sufficient for POC)
- Automatically create tables and schema on first use (check if tables exist, create if missing)
- Map all database errors to DataProviderError::Other with descriptive error messages
- Support key-based queries using asset_key column
- Support date-range queries filtering by timestamp between start and end dates (inclusive)

**Write Operations for Time-Series Data**
- Support inserting individual time-series points via insert method
- Support batch insert operations for efficient bulk data loading
- Support update operations for existing time-series points (upsert behavior)
- Handle duplicate timestamps by updating existing rows rather than failing
- Ensure data integrity with appropriate constraints and error handling

**Asset Storage Operations**
- Support storing Equity and Future structs as JSON blobs in assets table
- Serialize asset_key to string format for database storage
- Deserialize JSON blobs back to Equity or Future structs when retrieving
- Support querying assets by asset_key
- Handle asset updates by replacing existing JSON blob

**Analytics Storage Operations**
- Support storing analytics results keyed by asset_key + date + analytics_name
- Use flexible JSON structure for analytics values to support future analytics types
- Support querying analytics by asset_key and date range
- Support querying analytics by analytics_name for cross-asset analysis
- Allow updating analytics values for same key combination

**Connection and Initialization**
- Open SQLite database connection on provider creation
- Accept database file path as constructor parameter (default to in-memory for testing)
- Automatically initialize schema on first database access
- Close connection gracefully when provider is dropped
- Handle connection errors and retry logic if needed (simple approach for POC)

**Error Handling and Validation**
- Validate date ranges before executing queries (start <= end)
- Return DataProviderError::AssetNotFound when asset_key not found
- Return DataProviderError::InvalidDateRange for invalid date ranges
- Map all SQLite errors (connection failures, SQL errors, constraint violations) to DataProviderError::Other
- Include descriptive error messages in DataProviderError::Other variant

**Performance Considerations**
- Use prepared statements for repeated queries to improve performance
- Leverage SQLite indexes on asset_key and timestamp for fast date-range queries
- Support batch inserts using SQLite transactions for efficiency
- Keep implementation simple for POC (avoid premature optimization)

## Visual Design
No visual assets provided.

## Existing Code to Leverage

**DataProvider Trait**
- Implement the existing DataProvider trait from time_series module
- Follow the same method signature: get_time_series(asset_key, date_range) -> Result<Vec<TimeSeriesPoint>>
- Use existing DataProviderError enum for error handling
- Maintain compatibility with existing InMemoryDataProvider for testing

**AssetKey Serialization**
- Use existing AssetKey enum and its serialization capabilities
- Leverage AssetKey::as_string() method or serde serialization for database storage
- Parse serialized keys back to AssetKey enum when reading from database

**TimeSeriesPoint Structure**
- Use existing TimeSeriesPoint struct with timestamp and close_price fields
- Convert between database rows and TimeSeriesPoint structs
- Maintain DateTime<Utc> format for timestamps in database

## Out of Scope
- Connection pooling (single connection sufficient for POC)
- Migration system for schema changes (automatic creation only)
- Complex error types beyond DataProviderError::Other
- Normalized database schema with foreign keys (use JSON blobs for simplicity)
- Advanced query optimization beyond basic indexes
- Transaction management beyond basic SQLite transactions
- Multi-database support (SQLite only for POC)
- Backup and recovery mechanisms
- Database replication or synchronization
- Query result caching (implement at higher layer if needed)

