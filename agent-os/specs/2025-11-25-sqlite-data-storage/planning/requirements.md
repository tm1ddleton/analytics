# Spec Requirements: SQLite Data Storage

## Initial Description
SQLite Data Storage — Implement simple SQLite-based storage for asset data and computed analytics with key-based and date-range query capabilities

## Requirements Discussion

### First Round Questions

**Q1:** I'm assuming separate tables for assets (equities/futures metadata), time_series_data (timestamp + close_price per asset), and analytics (computed analytics results). Is that correct, or should we use a different schema?
**Answer:** Correct. Separate tables for assets, time_series_data, and analytics.

**Q2:** Should we store full Equity and Future structs (including corporate actions, expiry calendars) as JSON blobs, or normalize into separate tables with relationships?
**Answer:** JSON blobs fine. Store full Equity and Future structs as JSON.

**Q3:** Should time-series data be stored as individual rows per timestamp, or in a more compact format? Should we index by asset_key and timestamp for query performance?
**Answer:** Individual rows per timestamp. Index by asset_key and timestamp for query performance.

**Q4:** For "computed analytics," should we store analytics results keyed by asset_key + date + analytics_name, or use a different structure? Should this be flexible for future analytics types?
**Answer:** That is fine. Store analytics keyed by asset_key + date + analytics_name, flexible for future types.

**Q5:** Should the SQLite provider manage a single connection, a connection pool, or create connections on-demand? For POC, is a single connection sufficient?
**Answer:** Single connection fine. Single connection sufficient for POC.

**Q6:** Should the provider automatically create tables/schema on first use, or require explicit initialization? Should we include migration support for schema changes?
**Answer:** Automatic. Automatically create tables/schema on first use.

**Q7:** Should the provider support inserting/updating time-series data, or is it read-only for now? Should we support batch inserts for efficiency?
**Answer:** Write is fine. Support inserting/updating time-series data. Support batch inserts for efficiency.

**Q8:** Should database errors (connection failures, SQL errors) map to DataProviderError::Other, or should we add more specific error types?
**Answer:** DataProviderError::Other is fine. Map database errors to DataProviderError::Other.

### Existing Code to Reference
No existing code patterns to reference. This is a greenfield implementation.

### Follow-up Questions
No follow-up questions needed.

## Visual Assets

### Files Provided:
No visual assets provided.

### Visual Insights:
No visual assets to analyze.

## Requirements Summary

### Functional Requirements
- **Database Schema**: Separate tables for assets, time_series_data, and analytics
- **Asset Storage**: Store full Equity and Future structs as JSON blobs
- **Time-Series Storage**: Individual rows per timestamp with indexes on asset_key and timestamp
- **Analytics Storage**: Store analytics keyed by asset_key + date + analytics_name, flexible for future types
- **Connection Management**: Single connection for POC simplicity
- **Schema Initialization**: Automatically create tables/schema on first use
- **Write Operations**: Support inserting/updating time-series data with batch insert support
- **Error Handling**: Map database errors to DataProviderError::Other
- **DataProvider Trait**: Implement DataProvider trait for SQLite backend
- **Key-Based Queries**: Support querying by asset key
- **Date-Range Queries**: Support querying time-series data by date range

### Reusability Opportunities
No existing code patterns to reference. This is a greenfield implementation.

### Scope Boundaries
**In Scope:**
- SQLite database implementation
- DataProvider trait implementation for SQLite
- Tables for assets, time_series_data, and analytics
- Automatic schema creation
- Key-based and date-range queries
- Write operations (insert/update) for time-series data
- Batch insert support
- Simple single-connection architecture

**Out of Scope:**
- Connection pooling (single connection for POC)
- Migration system (automatic schema creation only)
- Complex error types (use DataProviderError::Other)
- Normalized database schema (use JSON blobs for assets)
- Advanced query optimization beyond basic indexes
- Transaction management beyond basic SQLite transactions
- Multi-database support (SQLite only)

### Technical Considerations
- **Keep it simple for POC**: Prioritize simplicity over advanced features
- **SQLite Library**: Use rusqlite or sqlx as specified in tech stack
- **JSON Serialization**: Use serde_json to serialize/deserialize asset structs
- **Indexing**: Create indexes on asset_key and timestamp for query performance
- **Batch Operations**: Support efficient batch inserts for time-series data
- **Error Mapping**: Map all database errors to DataProviderError::Other with descriptive messages
- **Schema Design**: Simple table structure with JSON columns for flexibility

