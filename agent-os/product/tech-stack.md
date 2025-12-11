# Product Tech Stack

## Core Language & Runtime
- **Language:** Rust
- **Package Manager:** Cargo
- **Minimum Rust Version:** Latest stable
- **Build System:** Cargo workspaces for library organization

## Core Domain Libraries
- **Decimal Arithmetic:** rust_decimal for financial calculations
- **Date/Time:** chrono for temporal operations
- **Serialization:** serde with JSON for API communication, protobuf for quant library integration
- **Error Handling:** thiserror for library errors, anyhow for application errors

## Computation & Data Structures
- **DAG Library:** petgraph for analytics dependency graph construction and execution
- **Data Processing:** Polars for dataframe operations and Python export (Phase 7)
- **Concurrency:** tokio for async runtime
- **Collections:** hashbrown for high-performance hash maps

## Event Processing
- **Trigger System:** Custom implementation supporting 8 trigger types
- **Callback Handling:** Transport-agnostic design with JSON/protobuf schema contracts
- **Move Rules Engine:** Multi-dimensional context matching with configurable booking patterns

## Ledger Implementation
- **In-Memory Mode:** Native Rust data structures for maximum speed (research/simulation)
- **Abstraction:** Trait-based design allowing pluggable storage backends
- **Output:** Moves returned as Rust objects, serialized (JSON/protobuf) for downstream consumption
- **Persistence:** OUT OF SCOPE - how moves get posted to persistent storage is handled externally

## Frontend
- **JavaScript Framework:** React
- **CSS Framework:** Tailwind CSS
- **UI Components:** Material UI (or shadcn/ui)
- **Real-Time Updates:** WebSocket or SSE for live analytics
- **Data Visualization:** TBD (Chart.js, Recharts, or similar)

## Python Integration (Phase 7)
- **Python Bindings:** PyO3 for direct Rust-to-Python bindings
- **Python Dataframes:** Polars Python library for dataframe export
- **Python Package:** Wheel distribution for PyPI or local installation

## Web Server & API
- **HTTP Framework:** axum for REST API (stub for demo/testing)
- **Real-Time Communication:** WebSocket (tokio-tungstenite) or SSE
- **API Format:** JSON or protobuf for request/response
- **API Documentation:** OpenAPI/Swagger specification

## Data Storage
- **Research/Simulation:** In-memory (no persistence)
- **POC/Demo:** SQLite for simple file-based storage (existing foundation)
- **Market Data Source:** Yahoo Finance (POC), production feeds TBD

## Testing & Quality
- **Test Framework:** Built-in Rust testing (cargo test)
- **Benchmarking:** criterion for performance benchmarks
- **Linting:** clippy for code quality
- **Formatting:** rustfmt for code formatting
- **Python Testing:** pytest for Python bindings

## Deployment & Infrastructure
- **Containerization:** Docker
- **Distribution:**
  - Rust crate (crates.io)
  - Python wheel (PyPI)
  - Docker image (REST API server)
- **CI/CD:** GitHub Actions
- **Hosting:** Cloud-agnostic (AWS, GCP, Azure, on-premises)

## Development Tools
- **Version Control:** Git
- **Documentation:** rustdoc for Rust API, Sphinx/mkdocs for Python
- **Dependency Management:** Cargo.lock for reproducible builds

## Architecture Patterns
- **Core Abstraction:** Ledger as the unifying data structure (Position Ledger + Index Ledger)
- **Processing Model:** Event-driven with two-call pattern (ProductAction -> Moves)
- **Computation Modes:** In-memory (research), Streaming (real-time)
- **Design Pattern:** Stateless Smart Contract Adapter
- **Move Rules:** Multi-dimensional context matching with pluggable booking patterns
- **Provisional Calculations:** Pre-fixing values propagate through DAG but don't persist in stateful operations
- **Output Model:** Moves as Rust objects, serialized for downstream systems
- **Scalability:** Horizontal scaling via stateless design

## Observability (Production - OUT OF SCOPE)
- **Logging:** tracing crate with structured output
- **Metrics:** Prometheus-compatible metrics
- **Error Tracking:** TBD (Sentry or similar)
