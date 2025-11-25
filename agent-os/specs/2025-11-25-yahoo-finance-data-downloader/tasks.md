# Task Breakdown: Yahoo Finance Data Downloader

## Overview
Total Tasks: 6 task groups

## Task List

### HTTP Client and API Integration

#### Task Group 1: HTTP Client Setup and Yahoo Finance API Integration
**Dependencies:** None

- [x] 1.0 Complete HTTP client setup and Yahoo Finance API integration
  - [x] 1.1 Add HTTP client dependency to Cargo.toml
    - Add reqwest or ureq crate with appropriate features (async support if using reqwest)
    - Ensure compatibility with existing dependencies
  - [x] 1.2 Write 2-8 focused tests for HTTP client setup
    - Test HTTP client creation
    - Test basic API request to Yahoo Finance
    - Test error handling for network failures
    - Test response parsing
  - [x] 1.3 Create YahooFinanceDownloader struct
    - Create struct with HTTP client field
    - Accept optional configuration (rate limit, retry settings)
    - Initialize HTTP client with appropriate settings
  - [x] 1.4 Implement Yahoo Finance API symbol lookup
    - Create method to convert AssetKey to Yahoo Finance symbol format
    - Handle equity ticker symbols (e.g., "AAPL" -> "AAPL")
    - Handle futures symbols (e.g., "ES" + expiry -> "ES=F" or contract-specific format)
    - Support both asset types through unified interface
  - [x] 1.5 Implement basic API request method
    - Create method to fetch historical data from Yahoo Finance
    - Handle API endpoint construction with symbol and date range
    - Parse HTTP response and handle errors
    - Return raw response data for processing
  - [x] 1.6 Ensure HTTP client and API integration tests pass
    - Run ONLY the 2-8 tests written in 1.2
    - Verify API requests work correctly
    - Verify error handling works
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 1.2 pass
- HTTP client successfully connects to Yahoo Finance API
- AssetKey can be converted to Yahoo Finance symbol format
- Basic API requests return data or appropriate errors

### Data Parsing and Conversion

#### Task Group 2: Yahoo Finance Data Parsing and Conversion
**Dependencies:** Task Group 1

- [x] 2.0 Complete Yahoo Finance data parsing and conversion
  - [x] 2.1 Write 2-8 focused tests for data parsing
    - Test parsing OHLCV data from Yahoo Finance response
    - Test extracting close prices only
    - Test timestamp conversion to DateTime<Utc>
    - Test handling missing or invalid data
  - [x] 2.2 Implement response data parsing
    - Parse Yahoo Finance API response format (CSV, JSON, or other format)
    - Extract OHLCV data points
    - Handle different response formats if Yahoo Finance uses multiple
  - [x] 2.3 Implement data conversion to TimeSeriesPoint
    - Extract close prices from OHLCV data
    - Drop Open, High, Low, Volume fields
    - Convert timestamps from Yahoo Finance format to DateTime<Utc>
    - Create TimeSeriesPoint structs for each data point
  - [x] 2.4 Implement date range filtering
    - Filter downloaded data to match requested date range
    - Handle timezone conversions correctly
    - Ensure inclusive date boundaries
  - [x] 2.5 Ensure data parsing and conversion tests pass
    - Run ONLY the 2-8 tests written in 2.1
    - Verify data is correctly parsed and converted
    - Verify date filtering works correctly
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 2.1 pass
- Yahoo Finance response data is correctly parsed
- Close prices are extracted and converted to TimeSeriesPoint structs
- Timestamps are correctly converted to DateTime<Utc>

### SqliteDataProvider Integration

#### Task Group 3: Integration with SqliteDataProvider
**Dependencies:** Task Group 2

- [ ] 3.0 Complete SqliteDataProvider integration
  - [ ] 3.1 Write 2-8 focused tests for SqliteDataProvider integration
    - Test downloading and storing data for single asset
    - Test downloading and storing data for multiple assets
    - Test duplicate data handling (skip existing dates)
    - Test batch insert operations
  - [ ] 3.2 Implement download method accepting SqliteDataProvider
    - Create download method that accepts &mut SqliteDataProvider
    - Accept AssetKey and DateRange parameters
    - Download data from Yahoo Finance
    - Store data using insert_time_series_batch method
  - [ ] 3.3 Implement duplicate data checking
    - Check existing data in SQLite before downloading
    - Query existing dates for the asset
    - Filter out dates that already exist in database
    - Only download missing dates (incremental behavior)
  - [ ] 3.4 Implement batch data storage
    - Use insert_time_series_batch for efficient storage
    - Handle large date ranges by batching if needed
    - Ensure data integrity with proper error handling
  - [ ] 3.5 Ensure SqliteDataProvider integration tests pass
    - Run ONLY the 2-8 tests written in 3.1
    - Verify data is correctly stored in SQLite
    - Verify duplicate checking works correctly
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 3.1 pass
- Data is successfully downloaded and stored in SQLite
- Duplicate dates are skipped (incremental downloads work)
- Batch insert operations are used for efficiency

### Error Handling and Retry Logic

#### Task Group 4: Error Handling and Retry Logic
**Dependencies:** Task Group 1

- [ ] 4.0 Complete error handling and retry logic
  - [ ] 4.1 Write 2-8 focused tests for error handling and retries
    - Test retry logic with configurable max attempts
    - Test handling of API failures
    - Test partial failure recovery (some assets succeed, others fail)
    - Test retry limit enforcement
  - [ ] 4.2 Create download error types
    - Define error enum for download-specific errors
    - Include API failures, network errors, parsing errors
    - Map Yahoo Finance API errors to appropriate error types
  - [ ] 4.3 Implement retry logic
    - Add configurable max retry attempts (default: 3)
    - Implement exponential backoff for retries
    - Track retry attempts per asset
    - Limit total retry attempts to prevent infinite retries
  - [ ] 4.4 Implement partial failure handling
    - Continue downloading other assets if one fails
    - Retry only failed assets (not successful ones)
    - Track which assets succeeded and which failed
    - Return clear error messages indicating failed assets
  - [ ] 4.5 Implement logging for errors and retries
    - Log all API errors and retry attempts
    - Log failed assets with error reasons
    - Log successful downloads and skipped assets
  - [ ] 4.6 Ensure error handling and retry tests pass
    - Run ONLY the 2-8 tests written in 4.1
    - Verify retry logic works correctly
    - Verify partial failure handling works
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 4.1 pass
- Retry logic works with configurable max attempts
- Partial failures are handled gracefully
- Clear error messages are returned for failed assets

### Rate Limiting

#### Task Group 5: Rate Limiting Implementation
**Dependencies:** Task Group 1

- [ ] 5.0 Complete rate limiting implementation
  - [ ] 5.1 Write 2-8 focused tests for rate limiting
    - Test rate limiting with configurable requests per second
    - Test handling of 429 (Too Many Requests) responses
    - Test delays between requests
    - Test exponential backoff on rate limit errors
  - [ ] 5.2 Implement rate limiting mechanism
    - Add configurable rate limit (default: 1 request per second)
    - Add delays between API requests
    - Track request timestamps to enforce rate limits
  - [ ] 5.3 Implement 429 response handling
    - Detect 429 (Too Many Requests) responses
    - Implement exponential backoff on rate limit errors
    - Retry requests after appropriate delay
  - [ ] 5.4 Integrate rate limiting with download methods
    - Apply rate limiting to all Yahoo Finance API requests
    - Ensure rate limits are respected across multiple asset downloads
    - Log rate limit events for debugging
  - [ ] 5.5 Ensure rate limiting tests pass
    - Run ONLY the 2-8 tests written in 5.1
    - Verify rate limiting works correctly
    - Verify 429 handling works
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 5.1 pass
- Rate limiting is enforced (1 request per second default)
- 429 responses are handled with exponential backoff
- Delays are added between requests

### Forward Filling and Advanced Features

#### Task Group 6: Forward Filling, Asset Metadata, and Corporate Actions
**Dependencies:** Task Groups 2, 3

- [ ] 6.0 Complete forward filling, asset metadata, and corporate actions
  - [ ] 6.1 Write 2-8 focused tests for forward filling
    - Test forward filling missing dates (weekends, holidays)
    - Test forward filling within date range only
    - Test forward filling with no data available
    - Test logging of forward-filled dates
  - [ ] 6.2 Implement forward filling logic
    - Identify missing dates in downloaded data (weekends, holidays, market closures)
    - Copy last available close price to missing dates
    - Only forward fill within requested date range
    - Do not forward fill beyond end date
  - [ ] 6.3 Implement asset metadata extraction
    - Extract metadata from Yahoo Finance if available (name, exchange, currency, sector)
    - Infer/guess metadata if not available (use ticker as name, default currency, etc.)
    - Create Equity or Future structs with metadata
    - Store assets using store_asset_equity or store_asset_future methods
  - [ ] 6.4 Implement corporate actions extraction
    - Extract stock splits from Yahoo Finance if available
    - Extract dividend information from Yahoo Finance if available
    - Store corporate actions in Equity struct's corporate_actions field
    - Handle cases where corporate actions are not available (empty vector)
  - [ ] 6.5 Implement futures-specific handling
    - Map futures contracts to Yahoo Finance symbol format
    - Handle contract month identification
    - Store futures-specific metadata (series, expiry_date, contract_month, expiry_calendar)
    - Support downloading multiple contracts for a series if needed
  - [ ] 6.6 Implement logging for forward filling and metadata
    - Log forward-filled dates for transparency
    - Log asset metadata extraction (successful or inferred)
    - Log corporate actions extraction
  - [ ] 6.7 Ensure forward filling and advanced features tests pass
    - Run ONLY the 2-8 tests written in 6.1
    - Verify forward filling works correctly
    - Verify asset metadata is extracted and stored
    - Verify corporate actions are extracted if available
    - Do NOT run the entire test suite at this stage

**Acceptance Criteria:**
- The 2-8 tests written in 6.1 pass
- Missing dates are forward filled with last available close price
- Asset metadata is extracted from Yahoo Finance or inferred
- Corporate actions are extracted and stored if available
- Futures contracts are handled correctly

## Execution Order

Recommended implementation sequence:
1. HTTP Client Setup and Yahoo Finance API Integration (Task Group 1) - Foundation for all API interactions
2. Data Parsing and Conversion (Task Group 2) - Core data processing
3. Error Handling and Retry Logic (Task Group 4) - Can proceed in parallel with Task Group 2
4. Rate Limiting Implementation (Task Group 5) - Can proceed in parallel with Task Group 2
5. Integration with SqliteDataProvider (Task Group 3) - Requires Task Group 2
6. Forward Filling, Asset Metadata, and Corporate Actions (Task Group 6) - Requires Task Groups 2 and 3

