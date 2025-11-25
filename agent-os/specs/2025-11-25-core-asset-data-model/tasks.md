# Task Breakdown: Core Asset Data Model

## Overview
Total Tasks: 4 task groups

## Task List

### Core Data Structures

#### Task Group 1: Asset Base Types and Key Identification
**Dependencies:** None

- [x] 1.0 Complete core asset data structures
  - [x] 1.1 Write 2-8 focused tests for asset key identification
    - Test key validation (empty strings, invalid characters)
    - Test key immutability
    - Test key-based asset lookup
    - Test equity key format (ticker symbols)
    - Test futures composite key format (series + expiry)
  - [x] 1.2 Create AssetKey enum or struct for key identification
    - Support string-based keys for equities (e.g., "AAPL", "MSFT")
    - Support composite keys for futures (series + expiry date)
    - Implement key validation (reject empty strings, invalid characters)
    - Make keys immutable and hashable for use in collections
  - [x] 1.3 Create base Asset trait or enum for asset type discrimination
    - Define common asset interface/behavior
    - Support asset type identification (Equity vs Future)
    - Ensure immutability design
  - [x] 1.4 Implement AssetKey Display and Debug traits
    - Format equity keys as ticker symbols
    - Format futures keys as "SERIES-YYYY-MM-DD" or similar
  - [x] 1.5 Ensure asset key tests pass
    - Run ONLY the 2-8 tests written in 1.1
    - Verify key validation works correctly
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 1.1 pass
- Asset keys support both equity and futures formats
- Key validation rejects invalid inputs
- Keys are immutable and hashable

### Asset Type Implementations

#### Task Group 2: Equity and Futures Asset Types
**Dependencies:** Task Group 1

- [ ] 2.0 Complete equity and futures asset implementations
  - [ ] 2.1 Write 2-8 focused tests for equity and futures structs
    - Test equity creation with metadata
    - Test futures creation with series and expiry
    - Test immutability (no mutation after creation)
    - Test asset type discrimination
    - Test metadata field access
  - [ ] 2.2 Create Equity struct with metadata fields
    - Fields: key, name, exchange, currency, sector
    - Include corporate action data structure (splits, dividends)
    - Implement constructor/factory methods
    - Ensure struct is immutable (no mutable fields)
  - [ ] 2.3 Create Future struct with expiry details
    - Fields: key (series + expiry), series (underlying), expiry_date, contract_month
    - Include expiry calendar reference/notion
    - Implement constructor/factory methods
    - Ensure struct is immutable
  - [ ] 2.4 Create common metadata struct for shared fields
    - Extract common fields (name, exchange, currency) if applicable
    - Use composition in Equity and Future structs
  - [ ] 2.5 Ensure equity and futures tests pass
    - Run ONLY the 2-8 tests written in 2.1
    - Verify struct creation and field access work
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 2.1 pass
- Equity struct supports all required metadata fields
- Future struct supports series, expiry date, and expiry details
- Both asset types are immutable
- Corporate action structure included in Equity

### Time-Series and Data Access

#### Task Group 3: Time-Series Data and Data Source Abstraction
**Dependencies:** Task Group 2

- [ ] 3.0 Complete time-series data structures and data source abstraction
  - [ ] 3.1 Write 2-8 focused tests for time-series and data access
    - Test time-series data structure (timestamp + close price)
    - Test data source trait implementation
    - Test asset querying data on-demand
    - Test data source agnostic design (no DB coupling)
  - [ ] 3.2 Create TimeSeriesPoint struct for close price data
    - Fields: timestamp (DateTime or similar), close_price (f64 or Decimal)
    - Support Vec<TimeSeriesPoint> for series data
    - Ensure immutability
  - [ ] 3.3 Create DataProvider trait for data source abstraction
    - Define trait methods: get_time_series(asset_key, date_range) -> Result<Vec<TimeSeriesPoint>>
    - Ensure trait is generic and not coupled to any database
    - Support multiple implementations (SQLite, in-memory, API, etc.)
  - [ ] 3.4 Integrate data provider into Asset types
    - Add method to query time-series data via DataProvider trait
    - Ensure assets remain agnostic about data source
    - Do not store data provider in asset struct (pass as parameter)
  - [ ] 3.5 Create in-memory DataProvider implementation for testing
    - Simple HashMap-based storage for test data
    - Implement DataProvider trait
    - Support date range queries
  - [ ] 3.6 Ensure time-series and data access tests pass
    - Run ONLY the 2-8 tests written in 3.1
    - Verify data source abstraction works
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 3.1 pass
- Time-series data structure supports timestamp + close price
- DataProvider trait enables data source agnostic design
- Assets can query data without database coupling
- In-memory provider works for testing

### Serialization and Advanced Features

#### Task Group 4: Serialization, Corporate Actions, and Futures Features
**Dependencies:** Task Group 3

- [ ] 4.0 Complete serialization and advanced asset features
  - [ ] 4.1 Write 2-8 focused tests for serialization and advanced features
    - Test serde Serialize/Deserialize for all asset types
    - Test JSON serialization format
    - Test corporate action handling in equities
    - Test futures expiry calendar functionality
    - Test rolling futures price generation
  - [ ] 4.2 Implement serde Serialize/Deserialize for all asset types
    - Add serde derives to AssetKey, Equity, Future structs
    - Ensure all metadata fields are serializable
    - Support JSON format for REST API responses
    - Test round-trip serialization (serialize then deserialize)
  - [ ] 4.3 Implement corporate action handling for equities
    - Create CorporateAction enum (Split, Dividend, etc.)
    - Add corporate_actions field to Equity struct
    - Implement methods to apply corporate actions to price data
    - Keep implementation rudimentary for POC
  - [ ] 4.4 Implement futures expiry calendar
    - Create ExpiryCalendar trait or struct
    - Support determining contract rollover dates
    - Support configurable days-before-expiry for rollover
  - [ ] 4.5 Implement rolling futures price generation
    - Create method to generate continuous price series
    - Support configurable days-before-expiry switchover
    - Switch between contracts at specified rollover points
    - Return continuous price series
  - [ ] 4.6 Ensure serialization and advanced feature tests pass
    - Run ONLY the 2-8 tests written in 4.1
    - Verify JSON serialization works correctly
    - Verify corporate actions and rolling futures functionality
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 4.1 pass
- All asset types serialize/deserialize correctly with serde
- Corporate actions structure exists in Equity
- Futures expiry calendar supports rollover date determination
- Rolling futures price generation works with configurable switchover

### Testing

#### Task Group 5: Test Review & Gap Analysis
**Dependencies:** Task Groups 1-4

- [ ] 5.0 Review existing tests and fill critical gaps only
  - [ ] 5.1 Review tests from Task Groups 1-4
    - Review the 2-8 tests written by core-data-engineer (Task 1.1)
    - Review the 2-8 tests written by asset-types-engineer (Task 2.1)
    - Review the 2-8 tests written by time-series-engineer (Task 3.1)
    - Review the 2-8 tests written by serialization-engineer (Task 4.1)
    - Total existing tests: approximately 8-32 tests
  - [ ] 5.2 Analyze test coverage gaps for THIS feature only
    - Identify critical user workflows that lack test coverage
    - Focus ONLY on gaps related to this spec's feature requirements
    - Do NOT assess entire application test coverage
    - Prioritize end-to-end workflows over unit test gaps
  - [ ] 5.3 Write up to 10 additional strategic tests maximum
    - Add maximum of 10 new tests to fill identified critical gaps
    - Focus on integration points and end-to-end workflows
    - Do NOT write comprehensive coverage for all scenarios
    - Skip edge cases, performance tests unless business-critical
  - [ ] 5.4 Run feature-specific tests only
    - Run ONLY tests related to this spec's feature (tests from 1.1, 2.1, 3.1, 4.1, and 5.3)
    - Expected total: approximately 18-42 tests maximum
    - Do NOT run the entire application test suite
    - Verify critical workflows pass

**Acceptance Criteria:**
- All feature-specific tests pass (approximately 18-42 tests total)
- Critical user workflows for this feature are covered
- No more than 10 additional tests added when filling in testing gaps
- Testing focused exclusively on this spec's feature requirements

## Execution Order

Recommended implementation sequence:
1. Core Data Structures (Task Group 1) - Foundation for asset key identification
2. Asset Type Implementations (Task Group 2) - Build equity and futures on top of keys
3. Time-Series and Data Access (Task Group 3) - Add data querying capabilities
4. Serialization and Advanced Features (Task Group 4) - Complete serialization and advanced features
5. Test Review & Gap Analysis (Task Group 5) - Final testing and gap filling

