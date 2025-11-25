# Task Breakdown: SQLite Data Storage

## Overview
Total Tasks: 4 task groups

## Task List

### Database Schema and Connection

#### Task Group 1: Database Schema and Connection Management
**Dependencies:** None

- [x] 1.0 Complete database schema and connection setup
  - [x] 1.1 Write 2-8 focused tests for database schema and connection
    - Test automatic schema creation on first use
    - Test table existence checks
    - Test connection initialization with file path
    - Test in-memory database for testing
    - Test connection error handling
  - [x] 1.2 Add rusqlite dependency to Cargo.toml
    - Add rusqlite crate with appropriate features
    - Ensure compatibility with existing dependencies
  - [x] 1.3 Create database schema initialization function
    - Create assets table with asset_key (TEXT PRIMARY KEY) and asset_data (TEXT JSON)
    - Create time_series_data table with asset_key (TEXT), timestamp (TEXT), close_price (REAL)
    - Create analytics table with asset_key (TEXT), date (TEXT), analytics_name (TEXT), value (TEXT JSON)
    - Create composite primary key or unique constraint on analytics table (asset_key, date, analytics_name)
    - Create indexes on time_series_data: asset_key and timestamp for query performance
  - [x] 1.4 Implement automatic schema creation
    - Check if tables exist before creating
    - Create tables only if they don't exist
    - Handle schema creation errors gracefully
  - [x] 1.5 Implement SQLite connection management
    - Create SqliteDataProvider struct with Connection field
    - Accept database file path in constructor (default to in-memory for testing)
    - Open connection on provider creation
    - Initialize schema automatically on first database access
    - Close connection on drop
  - [x] 1.6 Ensure database schema and connection tests pass
    - Run ONLY the 2-8 tests written in 1.1
    - Verify tables are created correctly
    - Verify connection works
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 1.1 pass
- Database schema is created automatically on first use
- Tables and indexes are created correctly
- Connection management works for both file and in-memory databases

### DataProvider Trait Implementation

#### Task Group 2: SQLite DataProvider Implementation
**Dependencies:** Task Group 1

- [x] 2.0 Complete SQLite DataProvider trait implementation
  - [x] 2.1 Write 2-8 focused tests for DataProvider trait implementation
    - Test get_time_series with valid asset key and date range
    - Test get_time_series with non-existent asset key
    - Test get_time_series with invalid date range
    - Test date range filtering (inclusive boundaries)
    - Test error mapping to DataProviderError
  - [x] 2.2 Implement DataProvider trait for SqliteDataProvider
    - Implement get_time_series method matching trait signature
    - Serialize AssetKey to string for database queries
    - Query time_series_data table filtering by asset_key and date range
    - Convert database rows to TimeSeriesPoint structs
    - Handle DateTime<Utc> conversion from database TEXT format
  - [x] 2.3 Implement error handling and validation
    - Validate date range (start <= end) before querying
    - Return DataProviderError::InvalidDateRange for invalid ranges
    - Return DataProviderError::AssetNotFound when asset_key not found
    - Map SQLite errors to DataProviderError::Other with descriptive messages
  - [x] 2.4 Use prepared statements for query performance
    - Create prepared statement for time-series queries
    - Reuse prepared statements for repeated queries
    - Bind parameters (asset_key, start_date, end_date) to prepared statements
  - [x] 2.5 Ensure DataProvider implementation tests pass
    - Run ONLY the 2-8 tests written in 2.1
    - Verify queries return correct data
    - Verify error handling works correctly
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 2.1 pass
- DataProvider trait is fully implemented
- Queries return correct time-series data filtered by date range
- Error handling maps correctly to DataProviderError variants

### Write Operations

#### Task Group 3: Time-Series Write Operations
**Dependencies:** Task Group 2

- [x] 3.0 Complete time-series write operations
  - [x] 3.1 Write 2-8 focused tests for write operations
    - Test inserting single time-series point
    - Test batch insert of multiple points
    - Test upsert behavior (update on duplicate timestamp)
    - Test inserting points for different asset keys
    - Test error handling for invalid data
  - [x] 3.2 Implement insert_time_series_point method
    - Insert single TimeSeriesPoint into time_series_data table
    - Serialize AssetKey to string
    - Convert DateTime<Utc> to database format (TEXT or INTEGER)
    - Handle duplicate timestamps with upsert (INSERT OR REPLACE)
  - [x] 3.3 Implement batch insert method
    - Accept vector of TimeSeriesPoint and AssetKey
    - Use SQLite transaction for batch efficiency
    - Insert all points in single transaction
    - Handle errors and rollback transaction on failure
  - [x] 3.4 Implement update_time_series_point method
    - Update existing time-series point by asset_key and timestamp
    - Return error if point doesn't exist (or use upsert)
    - Support updating close_price for existing timestamp
  - [x] 3.5 Ensure write operations tests pass
    - Run ONLY the 2-8 tests written in 3.1
    - Verify data is correctly inserted and updated
    - Verify batch operations work efficiently
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 3.1 pass
- Single and batch insert operations work correctly
- Upsert behavior handles duplicate timestamps
- Transactions ensure data integrity

### Asset and Analytics Storage

#### Task Group 4: Asset and Analytics Storage Operations
**Dependencies:** Task Group 2

- [x] 4.0 Complete asset and analytics storage operations
  - [x] 4.1 Write 2-8 focused tests for asset and analytics storage
    - Test storing Equity struct as JSON blob
    - Test storing Future struct as JSON blob
    - Test retrieving assets by asset_key
    - Test storing analytics results
    - Test querying analytics by asset_key and date range
  - [x] 4.2 Implement asset storage methods
    - Create store_asset method accepting Equity or Future
    - Serialize asset struct to JSON using serde_json
    - Serialize AssetKey to string for database key
    - Store JSON blob in assets table with asset_key as primary key
    - Handle asset updates by replacing existing JSON blob
  - [x] 4.3 Implement asset retrieval methods
    - Create get_asset method by asset_key
    - Deserialize JSON blob back to Equity or Future struct
    - Parse AssetKey from string format
    - Return appropriate error if asset not found
  - [x] 4.4 Implement analytics storage methods
    - Create store_analytics method with asset_key, date, analytics_name, value
    - Serialize analytics value as JSON for flexibility
    - Store in analytics table with composite key (asset_key, date, analytics_name)
    - Support updating analytics for same key combination
  - [x] 4.5 Implement analytics query methods
    - Create get_analytics method by asset_key and date range
    - Create get_analytics_by_name method for cross-asset analysis
    - Deserialize JSON values back to appropriate types
    - Return empty vector if no analytics found
  - [x] 4.6 Ensure asset and analytics storage tests pass
    - Run ONLY the 2-8 tests written in 4.1
    - Verify assets are stored and retrieved correctly
    - Verify analytics storage and queries work
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 4.1 pass
- Assets (Equity and Future) can be stored and retrieved as JSON
- Analytics can be stored and queried by various criteria
- JSON serialization/deserialization works correctly

### Testing

#### Task Group 5: Test Review & Gap Analysis
**Dependencies:** Task Groups 1-4

- [x] 5.0 Review existing tests and fill critical gaps only
  - [x] 5.1 Review tests from Task Groups 1-4
    - Review the 2-8 tests written by schema-engineer (Task 1.1)
    - Review the 2-8 tests written by dataprovider-engineer (Task 2.1)
    - Review the 2-8 tests written by write-operations-engineer (Task 3.1)
    - Review the 2-8 tests written by storage-engineer (Task 4.1)
    - Total existing tests: 26 tests
  - [x] 5.2 Analyze test coverage gaps for THIS feature only
    - Identify critical user workflows that lack test coverage
    - Focus ONLY on gaps related to this spec's feature requirements
    - Do NOT assess entire application test coverage
    - Prioritize end-to-end workflows over unit test gaps
  - [x] 5.3 Write up to 10 additional strategic tests maximum
    - Add maximum of 10 new tests to fill identified critical gaps
    - Focus on integration points and end-to-end workflows
    - Do NOT write comprehensive coverage for all scenarios
    - Skip edge cases, performance tests unless business-critical
    - Added 9 strategic tests covering end-to-end workflows
  - [x] 5.4 Run feature-specific tests only
    - Run ONLY tests related to this spec's feature (tests from 1.1, 2.1, 3.1, 4.1, and 5.3)
    - Total: 35 tests (26 existing + 9 new)
    - Do NOT run the entire application test suite
    - Verify critical workflows pass

**Acceptance Criteria:**
- All feature-specific tests pass (approximately 18-42 tests total)
- Critical user workflows for this feature are covered
- No more than 10 additional tests added when filling in testing gaps
- Testing focused exclusively on this spec's feature requirements

## Execution Order

Recommended implementation sequence:
1. Database Schema and Connection (Task Group 1) - Foundation for all database operations
2. DataProvider Trait Implementation (Task Group 2) - Core read functionality
3. Write Operations (Task Group 3) - Time-series data insertion (can proceed in parallel with Task Group 4)
4. Asset and Analytics Storage (Task Group 4) - Asset and analytics operations (can proceed in parallel with Task Group 3)
5. Test Review & Gap Analysis (Task Group 5) - Final testing and gap filling

