# Product Tech Stack

## Core Language & Runtime
- **Language:** Rust
- **Package Manager:** Cargo
- **Minimum Rust Version:** TBD (target latest stable)
- **Build System:** Cargo workspaces for library organization

## Computation & Data Structures
- **DAG Library:** petgraph (or similar open-source Rust DAG library) for explicit analytics dependency graph construction and execution
- **Data Processing:** Polars for dataframe operations and data export to Python (post-POC)
- **Concurrency:** tokio for async runtime and concurrent computation
- **Serialization:** serde with appropriate formats (JSON, MessagePack, etc.) for API communication
- **HTTP Client:** reqwest or ureq for downloading data from Yahoo Finance or other APIs
- **SQLite:** rusqlite or sqlx for SQLite database operations

## Frontend
- **JavaScript Framework:** React
- **CSS Framework:** Tailwind CSS
- **UI Components:** Material UI (or shadcn/ui)
- **Real-Time Updates:** WebSocket client or EventSource (for SSE) to receive analytics updates from backend
- **Data Visualization:** TBD (Chart.js, Recharts, or similar for displaying analytics and asset data)

## Python Integration (Post-POC)
- **Python Bindings:** PyO3 for direct Rust-to-Python bindings
- **Python Dataframes:** Polars Python library for dataframe export and manipulation
- **Python Package:** Build Python wheel distribution for PyPI or local installation

## Web Server & API
- **HTTP Framework:** axum (or actix-web/warp) for REST API server implementation
- **Real-Time Communication:** WebSocket (via tokio-tungstenite) or Server-Sent Events (SSE) for pushing analytics updates to React UI
- **API Format:** JSON for request/response payloads
- **Authentication:** TBD (JWT, API keys, or OAuth2 based on requirements) - not required for POC
- **API Documentation:** OpenAPI/Swagger specification

## Data Storage
- **POC Storage:** SQLite for simple, file-based storage of asset data and computed analytics
- **Data Source:** Yahoo Finance API (or alternative free financial data APIs like Alpha Vantage, Polygon.io free tier)
- **Query Interface:** Key-based and date-range queries for asset data and analytics via SQLite
- **Future Storage:** TBD (time-series database or object storage for production scale)
- **Caching:** In-memory caching layer for frequently accessed analytics (optional Redis for distributed scenarios)

## Testing & Quality
- **Test Framework:** Built-in Rust testing (cargo test)
- **Benchmarking:** criterion for performance benchmarks
- **Linting:** clippy for Rust code quality
- **Formatting:** rustfmt for code formatting
- **Python Testing:** pytest for Python bindings and client testing

## Deployment & Infrastructure
- **Containerization:** Docker for containerized deployment
- **Distribution:** 
  - Rust crate published to crates.io
  - Python wheel for PyPI or private package repository
  - Docker image for REST API server
- **CI/CD:** GitHub Actions (or similar) for automated testing and releases
- **Hosting:** TBD (cloud-agnostic design, deployable to AWS, GCP, Azure, or on-premises)

## Development Tools
- **Version Control:** Git
- **Documentation:** 
  - rustdoc for Rust API documentation
  - Sphinx or mkdocs for Python documentation
- **Dependency Management:** Cargo.lock for reproducible builds

## Architecture Patterns
- **Design Pattern:** Asset-centric object model (assets as first-class objects, not dataframe rows)
- **Computation Model:** Dual-mode engine (push-mode for real-time, pull-mode for historical)
- **Access Patterns:** Multi-modal (embedded Rust, Python PyO3, REST API, Python REST client)
- **Scalability:** Designed for horizontal scaling with distributed computation capabilities

## Third-Party Services (Optional)
- **Monitoring:** TBD (Prometheus, Datadog, or similar for production observability)
- **Logging:** tracing crate with appropriate backends (e.g., tracing-subscriber)
- **Error Tracking:** TBD (Sentry or similar for error monitoring)

