# Specification: Yahoo Finance Data Downloader

## Goal
Create a data ingestion module that downloads historical market data from Yahoo Finance and stores it in SQLite, providing seamless integration with the existing asset-centric architecture and SqliteDataProvider.

## User Stories
- As a quantitative trader, I want to download historical market data from Yahoo Finance and store it in SQLite so that I can build and backtest trading strategies with real market data
- As a research analyst, I want to download historical price data for multiple assets over date ranges so that I can analyze price patterns and compute analytics
- As a system developer, I want a Yahoo Finance downloader that integrates with SqliteDataProvider so that downloaded data is immediately available for analytics computation

## Specific Requirements

**Asset Type Support**
- Support equities (stocks) via ticker symbols (e.g., "AAPL", "MSFT")
- Support futures contracts if Yahoo Finance supports them
- Auto-detect asset type based on AssetKey format (Equity vs Future)
- Handle both asset types through a unified download interface

**Data Source**
- Use Yahoo Finance API exclusively (no alternative APIs or fallbacks)
- Use appropriate Rust HTTP client library (reqwest or ureq) for API requests
- Handle Yahoo Finance API rate limits and response formats
- Support both historical and current data downloads

**Date Range Handling**
- Accept start and end date parameters (NaiveDate format)
- Download all available data within the specified date range
- Forward fill missing data points (weekends, holidays) by copying the last available close price
- Handle edge cases: single date requests, date ranges spanning multiple years, future dates

**Integration with SqliteDataProvider**
- Accept SqliteDataProvider instance (mutable reference) as parameter
- Write time-series data directly to SQLite using existing provider methods (insert_time_series_batch)
- Automatically create/update Equity/Future asset records when downloading
- Store asset metadata (name, exchange, currency, sector) if available from Yahoo Finance
- Use batch insert operations for efficient data storage

**Download Strategy**
- Batch downloads: download all requested data for an asset in a single API call when possible
- Check existing data in SQLite before downloading to avoid duplicates
- Skip downloading dates that already exist in the database (incremental behavior)
- Support downloading multiple assets in sequence or parallel (simple sequential for POC)

**Error Handling and Retries**
- Implement retry logic with configurable max attempts (default: 3 attempts)
- Implement rate limiting (requests per second/minute) to respect Yahoo Finance limits
- Handle partial failures gracefully: retry only failed assets, continue with other assets
- Limit total retry attempts per asset to prevent infinite retries
- Log all errors and retry attempts for debugging
- Return clear error messages indicating which assets failed and why

**Data Processing**
- Yahoo Finance returns OHLCV (Open, High, Low, Close, Volume) data
- Extract only close prices, drop Open, High, Low, Volume fields
- Convert timestamps from Yahoo Finance format to DateTime<Utc>
- Extract corporate actions (splits, dividends) from Yahoo Finance if available
- Store corporate actions in Equity struct when creating/updating assets
- No additional data transformation or calculation required

**Progress Tracking**
- Logging only (no callbacks, events, or progress bars)
- Log download progress: asset being downloaded, date range, number of data points
- Log errors: failed assets, retry attempts, API errors
- Log completion status: successful downloads, skipped assets, failed assets

**Asset Metadata**
- Download metadata from Yahoo Finance if available:
  - Company/asset name
  - Exchange (e.g., "NASDAQ", "NYSE")
  - Currency (e.g., "USD", "EUR")
  - Sector (for equities)
- If metadata not available from Yahoo Finance, infer/guess from available information:
  - Use ticker symbol as name if name unavailable
  - Default to "USD" for currency if not specified
  - Default to "Unknown" for sector if not available
- Store metadata in Equity/Future structs when creating assets
- Update existing asset metadata if asset already exists in database

**Futures Handling**
- Identify futures contracts by series (underlying) + expiry date
- Map expiry dates to Yahoo Finance symbol format (e.g., "ES=F" for E-mini S&P 500)
- Handle rolling contracts: download multiple contracts for a series if requested
- Support contract month identification and expiry calendar integration
- Store futures-specific metadata (series, expiry_date, contract_month, expiry_calendar)

**Corporate Actions Extraction**
- Extract stock splits from Yahoo Finance if available
- Extract dividend information from Yahoo Finance if available
- Store corporate actions in Equity struct's corporate_actions field
- Handle corporate action dates and apply them to time-series data if needed
- If corporate actions not available, leave corporate_actions vector empty

**API Rate Limiting**
- Implement rate limiting to respect Yahoo Finance API limits
- Default: 1 request per second (configurable)
- Add delays between requests to avoid hitting rate limits
- Handle 429 (Too Many Requests) responses with exponential backoff

**Forward Filling Missing Data**
- Identify missing dates in downloaded data (weekends, holidays, market closures)
- Forward fill by copying the last available close price to missing dates
- Only forward fill within the requested date range
- Do not forward fill beyond the end date
- Log forward-filled dates for transparency

## Visual Design
No visual assets provided.

## Existing Code to Leverage

**SqliteDataProvider Integration**
- Use existing SqliteDataProvider struct and its methods
- Leverage insert_time_series_batch for efficient batch data storage
- Use store_asset_equity and store_asset_future for asset metadata storage
- Integrate with existing database schema (time_series_data and assets tables)

**AssetKey and Asset Types**
- Use existing AssetKey enum for asset identification
- Use AssetKey::new_equity() and AssetKey::new_future() for key creation
- Use AssetKey::as_string() for Yahoo Finance symbol lookup
- Leverage existing Equity and Future structs for asset storage

**TimeSeriesPoint Structure**
- Convert Yahoo Finance data to TimeSeriesPoint structs
- Use TimeSeriesPoint::new(timestamp, close_price) for data points
- Maintain DateTime<Utc> format for timestamps

**DateRange Structure**
- Use existing DateRange struct for date range specification
- Leverage DateRange::new(start, end) for date range creation

**Error Handling**
- Use existing DataProviderError enum where appropriate
- Create new error types for download-specific errors (API failures, rate limiting, etc.)
- Map Yahoo Finance API errors to appropriate error types

## Out of Scope
- Alternative data sources (Alpha Vantage, Polygon.io, etc.)
- Real-time streaming data (only historical downloads)
- Data validation beyond basic format checking
- Complex data transformations or calculations
- Progress callbacks or events (logging only)
- Parallel downloads of multiple assets (sequential for POC)
- Caching of downloaded data beyond SQLite storage
- Data quality checks or outlier detection
- Automatic data updates or scheduled downloads
- Support for options, bonds, or other asset types beyond equities and futures
- Corporate action adjustment of historical prices (store actions but don't adjust prices)
- Advanced retry strategies beyond simple exponential backoff
- API authentication or API keys (use public Yahoo Finance endpoints)
- Data compression or optimization beyond SQLite's built-in features

