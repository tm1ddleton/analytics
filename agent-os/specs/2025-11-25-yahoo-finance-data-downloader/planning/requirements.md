# Spec Requirements: Yahoo Finance Data Downloader

## Initial Description
Yahoo Finance Data Downloader — Create data ingestion module that downloads historical market data from Yahoo Finance (or alternative free APIs) and stores in SQLite

## Requirements Discussion

### First Round Questions

**Q1:** Which asset types should the downloader support?
**Answer:** Equities and futures (if supported by Yahoo Finance).

**Q2:** Which data source(s) should we use?
**Answer:** Just Yahoo Finance. No alternative APIs or fallbacks needed.

**Q3:** How should date ranges be specified?
**Answer:** Start and end dates. Forward fill missing data points (weekends, holidays).

**Q4:** How should this integrate with existing SqliteDataProvider?
**Answer:** Accept a SqliteDataProvider instance and write directly to it.

**Q5:** Batch vs incremental downloads?
**Answer:** Batch downloads are fine.

**Q6:** Error handling and retries?
**Answer:** Retry logic with rate limiting. For partial failures, just retry the missing assets. Should continue up to a limited number of retries.

**Q7:** Data transformation?
**Answer:** No transformation needed, just drop OHL (only keep close prices). Extract corporate action data if possible from Yahoo Finance.

**Q8:** Progress tracking?
**Answer:** Just logging.

**Q9:** Asset metadata?
**Answer:** Download and store asset metadata if possible from Yahoo Finance. Otherwise, just infer/guess it.

**Q10:** Futures handling?
**Answer:** Sounds sensible (referring to the proposed approach: contract identification by series + expiry, handling rolling contracts, expiry date mapping to Yahoo Finance symbols).

## Key Requirements Summary

### Asset Support
- Support equities (stocks) via ticker symbols
- Support futures contracts if Yahoo Finance supports them
- Auto-detect asset type based on asset key format

### Data Source
- Use Yahoo Finance API exclusively
- No fallback APIs required

### Date Range Handling
- Accept start and end date parameters
- Forward fill missing data points (weekends, holidays)
- Download all available data within the specified range

### Integration
- Accept SqliteDataProvider instance as parameter
- Write time-series data directly to SQLite using existing provider methods
- Automatically create/update Equity/Future asset records when downloading

### Download Strategy
- Batch downloads (download all requested data)
- Check existing data in SQLite before downloading (avoid duplicates)
- Use batch insert operations for efficiency

### Error Handling
- Implement retry logic with configurable max attempts
- Implement rate limiting (requests per second/minute)
- Handle partial failures gracefully (retry only failed assets)
- Continue downloading other assets if one fails
- Limit total retry attempts per asset

### Data Processing
- Yahoo Finance returns OHLCV data
- Drop Open, High, Low, Volume - only store close prices
- Extract corporate actions (splits, dividends) from Yahoo Finance if available
- Convert timestamps to UTC
- No additional data transformation required

### Progress Tracking
- Logging only (no callbacks or events)
- Log download progress, errors, and completion status

### Asset Metadata
- Download metadata from Yahoo Finance if available (name, exchange, currency, sector)
- If metadata not available, infer/guess from available information
- Store metadata in Equity/Future structs when creating assets

### Futures Handling
- Identify contracts by series + expiry date
- Map expiry dates to Yahoo Finance symbol format
- Handle rolling contracts appropriately
- Download multiple contracts for a series if needed

