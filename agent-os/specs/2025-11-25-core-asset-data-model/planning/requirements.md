# Spec Requirements: Core Asset Data Model

## Initial Description
Core Asset Data Model — Implement asset objects as first-class entities with key-based identification, metadata storage, and time-series data attachment capabilities

## Requirements Discussion

### First Round Questions

**Q1:** I'm assuming asset keys will be string-based identifiers (like ticker symbols such as "AAPL", "MSFT") that uniquely identify each asset. Is that correct, or should we support UUIDs or composite keys?
**Answer:** This is a reasonable assumption. Asset keys will be string-based identifiers.

**Q2:** I'm assuming we'll primarily support equities (stocks) for the POC, with the data model extensible to support other asset types (bonds, futures, etc.) later. Should we include an asset type field now, or keep it simple for POC?
**Answer:** We should support equities and futures. Futures should contain relevant expiry details, there should be a notion of the expiry calendar. It should be possible to generate a rolling futures price specifying the days before to switch over. Futures should be denoted by a series (i.e. their underlying) and an expiry date as the key. Equities should have rudimentary corporate action treatment as part of the object.

**Q3:** I'm thinking metadata should include fields like asset name, exchange, currency, and sector. Should metadata be stored as a flexible key-value map (HashMap) for extensibility, or a fixed struct with specific fields?
**Answer:** These metadata fields should be properties of the struct given we are taking an object-based approach.

**Q4:** I'm assuming time-series data will be OHLCV (Open, High, Low, Close, Volume) bars with timestamps. Should we store this as a Vec of structs in memory, or should the asset object just reference/query the SQLite storage?
**Answer:** Just close prices for now.

**Q5:** I'm thinking the Asset struct should own its metadata but reference time-series data (either via ID or lazy loading from SQLite). Should assets own their time-series data in memory, or should they query it on-demand from storage?
**Answer:** The asset should be agnostic as to where it gets its data and should certainly not be coupled to any db.

**Q6:** I'm assuming assets need to be serializable (via serde) for JSON API responses and potentially for caching. Should we implement Serialize/Deserialize traits, or is this out of scope for the initial data model?
**Answer:** Implement serde.

**Q7:** I'm assuming the asset data model will be used in a multi-threaded context (tokio async runtime). Should Asset objects be Send + Sync, or will they be accessed from a single thread with message passing?
**Answer:** Elaborate on this please.

**Q8:** I'm thinking the initial implementation should support: creating assets with key and metadata, attaching time-series data, and querying by key. Should we also include update operations for metadata, or keep it immutable for the POC?
**Answer:** Immutable.

### Existing Code to Reference
No similar existing features identified for reference. This is a new codebase with no existing Rust code patterns to reuse.

### Follow-up Questions

**Follow-up 1:** Since we're using tokio for async runtime and the system will handle multiple assets concurrently, I need to understand the concurrency model:
1. Asset access pattern: Will multiple async tasks need to read the same Asset object simultaneously (requiring Arc<Asset> or similar), or will each task work with its own Asset instance?
2. Data updates: When time-series data updates arrive in push mode, will updates happen from a single thread/task (making Arc<Mutex<Asset>> or Arc<RwLock<Asset>> unnecessary), or could multiple tasks update the same asset concurrently?
3. Send + Sync requirements: Should Asset implement Send + Sync so it can be moved between threads, or will all asset operations stay within a single tokio runtime on one thread?

**Answer:** Assume the simplest threading mode possible for now.

## Visual Assets

### Files Provided:
No visual assets provided.

### Visual Insights:
No visual assets to analyze.

## Requirements Summary

### Functional Requirements
- **Asset Key Format**: String-based identifiers (ticker symbols like "AAPL", "MSFT") that uniquely identify each asset
- **Asset Types**: Support both equities and futures
  - **Equities**: Must include rudimentary corporate action treatment as part of the object
  - **Futures**: 
    - Denoted by a series (underlying) and an expiry date as the key
    - Must contain relevant expiry details
    - Must support expiry calendar notion
    - Must support generating rolling futures price with configurable days-before-expiry switchover
- **Metadata Structure**: Fixed struct with specific fields as properties (not HashMap), following object-based approach
- **Time-Series Data**: Store just close prices for now (not full OHLCV)
- **Data Source Agnostic**: Asset objects must be agnostic about where they get their data and must not be coupled to any database
- **Serialization**: Implement serde Serialize/Deserialize traits for JSON API responses and caching
- **Immutability**: Assets are immutable - no update operations for metadata
- **Core Operations**: Create assets with key and metadata, attach time-series data, query by key

### Reusability Opportunities
No existing code patterns to reference. This is a greenfield implementation.

### Scope Boundaries
**In Scope:**
- Asset struct with key-based identification
- Support for equities and futures asset types
- Fixed metadata struct with specific fields
- Close price time-series data attachment
- Serde serialization support
- Immutable asset objects
- Data source agnostic design (no DB coupling)
- Futures expiry calendar and rolling price generation
- Corporate action treatment for equities

**Out of Scope:**
- Full OHLCV time-series data (only close prices)
- Mutable/update operations
- Database coupling in asset objects
- Complex threading/concurrency (simplest mode)
- Other asset types beyond equities and futures (for now)

### Technical Considerations
- **Threading Model**: Simplest threading mode possible - owned Asset structs, no shared ownership (Arc) or interior mutability (Mutex/RwLock) needed
- **Language**: Rust with serde for serialization
- **Asset Key**: String-based (ticker symbols for equities, series+expiry for futures)
- **Futures Key Format**: Combination of series (underlying) and expiry date
- **Data Access**: Asset objects query data on-demand from any source (not owned in memory, not coupled to storage)
- **Immutability**: All asset operations are immutable - create new instances rather than modify existing ones
- **Corporate Actions**: Equities must handle basic corporate actions (splits, dividends, etc.) as part of the object model
- **Rolling Futures**: Futures must support generating continuous price series with configurable rollover logic

