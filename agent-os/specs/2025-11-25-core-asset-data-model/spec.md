# Specification: Core Asset Data Model

## Goal
Implement asset objects as first-class entities with key-based identification, metadata storage, and time-series data attachment capabilities to support both equities and futures in an immutable, data-source-agnostic design.

## User Stories
- As a quantitative trader, I want to create asset objects with unique keys and metadata so that I can identify and track financial instruments in my trading strategies
- As a research analyst, I want to work with asset objects that support both equities and futures so that I can analyze different asset types in my strategy research
- As a system developer, I want assets to be data-source agnostic so that the analytics engine can work with data from any storage backend

## Specific Requirements

**Asset Key Identification**
- String-based keys uniquely identify each asset (e.g., "AAPL", "MSFT" for equities)
- Futures use composite key format: series (underlying) + expiry date
- Keys must be immutable and used for asset lookup and identification
- Key format validation should reject empty strings and invalid characters

**Equity Asset Type**
- Struct includes fields for equity-specific metadata (name, exchange, currency, sector)
- Must support rudimentary corporate action treatment as part of the object
- Corporate actions include splits, dividends, and other adjustments
- Metadata stored as fixed struct properties following object-based approach

**Futures Asset Type**
- Struct includes series (underlying identifier) and expiry date as key components
- Must contain relevant expiry details (expiry date, contract month, etc.)
- Must support expiry calendar notion for determining contract rollover dates
- Must support generating rolling futures price with configurable days-before-expiry switchover
- Rolling price generation creates continuous price series by switching between contracts

**Metadata Structure**
- Fixed struct with specific fields as properties (not HashMap)
- Common metadata fields: name, exchange, currency, sector, asset type
- All metadata fields are part of the struct definition
- Metadata is immutable once asset is created

**Time-Series Data Attachment**
- Assets support attaching close price time-series data
- Time-series data structure: timestamp + close price pairs
- Asset objects are agnostic about data source (not coupled to database)
- Data can be queried on-demand from any source via abstraction layer
- Only close prices supported initially (not full OHLCV)

**Serialization Support**
- Implement serde Serialize and Deserialize traits for all asset types
- Support JSON serialization for REST API responses
- Support serialization for caching and persistence if needed
- Ensure all asset metadata and key information is serializable

**Immutability Design**
- All asset objects are immutable after creation
- No update or mutation operations on existing assets
- Create new asset instances rather than modify existing ones
- Immutability ensures thread safety in simplest threading mode

**Threading Model**
- Simplest threading mode: owned Asset structs, no shared ownership
- No Arc, Mutex, or RwLock required for basic asset access
- Each task/thread works with its own Asset instance
- Assets can be cloned if needed for concurrent access

**Asset Creation and Query**
- Support creating assets with key and metadata
- Support attaching time-series data to assets
- Support querying assets by key
- Asset factory/constructor methods for different asset types

**Data Source Abstraction**
- Asset objects must not be coupled to any database
- Use trait-based abstraction for data access (e.g., DataProvider trait)
- Assets query data on-demand through abstraction layer
- Support multiple data sources (SQLite, in-memory, API, etc.) via same interface

## Visual Design
No visual assets provided.

## Existing Code to Leverage
No existing code patterns to reference. This is a greenfield Rust implementation.

## Out of Scope
- Full OHLCV time-series data (only close prices supported)
- Mutable/update operations on existing assets
- Database coupling in asset object definitions
- Complex threading/concurrency patterns (Arc, Mutex, RwLock)
- Other asset types beyond equities and futures (bonds, options, etc.)
- Asset versioning or historical metadata tracking
- Built-in data persistence in asset objects
- Asset relationship modeling (parent/child, derivatives, etc.)
- Real-time data streaming integration in asset model
- Asset validation against external data sources

