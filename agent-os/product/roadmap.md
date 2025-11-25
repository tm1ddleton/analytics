# Product Roadmap

## POC Phase (Immediate Focus)

1. [x] Core Asset Data Model — Implement asset objects as first-class entities with key-based identification, metadata storage, and time-series data attachment capabilities `M`
2. [x] SQLite Data Storage — Implement simple SQLite-based storage for asset data and computed analytics with key-based and date-range query capabilities `S`
3. [x] Yahoo Finance Data Downloader — Create data ingestion module that downloads historical market data from Yahoo Finance (or alternative free APIs) and stores in SQLite `S`
4. [x] DAG Computation Framework — Build DAG construction and execution engine using appropriate Rust libraries (e.g., petgraph) to wire analytics dependencies explicitly with cycle detection and topological sorting `L`
5. [x] Push-Mode Analytics Engine — Implement incremental computation system where analytics update automatically when new data arrives, with dependency propagation through the DAG `XL`
6. [x] Basic Analytics Library — Create foundational analytics calculations (e.g., moving averages, returns, volatility) that work in push mode `M`
7. [x] High-Speed Data Replay System — Implement replay engine that reads historical data from SQLite and feeds it into push-mode analytics at configurable high speed (faster than real-time) `M`
8. [x] Pull-Mode Analytics Engine — Implement time-series generation system that computes complete historical analytics on-demand for specified date ranges `L`
9. [x] REST API Server with WebSocket/SSE — Build HTTP server with REST endpoints for querying analytics and WebSocket or Server-Sent Events for real-time push updates to UI `M`
10. [x] React UI Dashboard — Create React frontend that connects to REST API, displays real-time analytics updates via WebSocket/SSE, shows asset data visualization, and includes controls for replay speed and asset selection `L`

## Post-POC Development

11. [ ] Embedded Rust API — Design and implement clean, ergonomic Rust API for using the engine directly in Rust applications with comprehensive documentation `M`
12. [ ] Python PyO3 Bindings — Create Python bindings using PyO3 that expose core Rust API functions, asset management, and analytics computation to Python `L`
13. [ ] Polars Dataframe Integration — Implement conversion layer that exports analytics results as Polars dataframes in Python bindings and REST API responses `S`
14. [ ] Python REST Client — Create Python client library that wraps REST API calls and returns Polars dataframes, providing alternative to PyO3 for distributed access `S`
15. [ ] Real-Time Data Ingestion — Implement streaming data input system that accepts live market data updates and triggers push-mode analytics computation `M`
16. [ ] Strategy Output System — Build mechanism for strategies to subscribe to analytics updates and receive real-time notifications when outputs change `M`
17. [ ] Distributed Architecture Foundation — Design and implement distributed computation capabilities with node coordination, data partitioning, and result aggregation `XL`
18. [ ] Performance Optimization — Optimize computation engine for high-throughput scenarios including parallel DAG execution, caching, and memory management `L`
19. [ ] Comprehensive Testing Suite — Create unit tests, integration tests, and performance benchmarks covering all access modes and computation patterns `M`

> Notes
> - POC Phase (1-10) focuses on demonstrating both push-mode and pull-mode analytics with high-speed replay in a React UI
> - Order items by technical dependencies and product architecture
> - Each item should represent an end-to-end (library + API + integration) functional and testable feature
> - POC uses SQLite for simplicity; production will require more sophisticated storage
> - React UI demonstrates real-time analytics updates during high-speed replay and on-demand batch computation

