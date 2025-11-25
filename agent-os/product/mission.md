# Product Mission

## Pitch
Financial Analytics Engine is a Rust library and calculation engine that helps quantitative traders and researchers build systematic trading strategies by providing real-time and historical financial analytics through a flexible, scalable, and performant computation platform.

## Users

### Primary Customers
- **Quantitative Traders**: Professionals building systematic trading strategies who need real-time analytics that update as market data streams in
- **Research Analysts**: Quantitative researchers who need to download precomputed analytics via Python to explore and backtest new strategy ideas

### User Personas

**Trading Strategist** (25-45)
- **Role:** Quantitative trader or portfolio manager
- **Context:** Building and deploying systematic trading strategies that require real-time analytics updates
- **Pain Points:** Existing analytics libraries are too slow, don't support real-time updates, or require complex data pipeline management. Need analytics that update automatically when new market data arrives.
- **Goals:** Deploy strategies with analytics that update in real-time, scale to handle multiple assets and strategies simultaneously, and integrate seamlessly into trading systems

**Research Analyst** (25-50)
- **Role:** Quantitative researcher or data scientist
- **Context:** Exploring new trading strategies by analyzing historical analytics and testing hypotheses
- **Pain Points:** Difficult to access precomputed analytics, slow iteration cycles when testing new ideas, analytics libraries don't integrate well with Python data science workflows
- **Goals:** Quickly download precomputed analytics for specific assets and date ranges, work with familiar Python tools (Polars dataframes), and iterate rapidly on strategy research

## The Problem

### Fragmented Analytics Infrastructure
Quantitative trading teams struggle with analytics libraries that are either too slow for real-time use, don't support incremental updates, or require complex data pipeline management. Existing solutions often force a choice between performance and flexibility, and don't provide unified access patterns for both real-time trading and historical research workflows.

**Our Solution:** A unified Rust-based analytics engine that supports both push-mode (real-time incremental updates) and pull-mode (historical time series) computation, with explicit DAG-based wiring for transparent dependency management and optimal performance.

### Limited Integration Options
Traders and researchers need analytics accessible from multiple environments—embedded in Rust applications, via Python for research, and through web APIs for distributed systems. Current solutions typically support only one access pattern, forcing teams to build custom integration layers.

**Our Solution:** Multi-modal access including embedded Rust library, Python bindings via PyO3, REST API for distributed queries, and seamless Polars dataframe integration for Python workflows.

## Differentiators

### Asset-Centric Architecture
Unlike analytics libraries built on dataframe-first designs, we model assets as first-class objects with explicit relationships. This results in clearer code, better performance through optimized data structures, and more intuitive API design that matches how traders think about financial instruments.

### Dual-Mode Computation Engine
Unlike single-mode analytics libraries, we support both push-mode (incremental updates) and pull-mode (time series generation) in the same unified engine. This results in one codebase that serves both real-time trading and historical research needs, reducing maintenance overhead and ensuring consistency.

### Explicit DAG-Based Wiring
Unlike black-box analytics pipelines, we provide explicit DAG construction for analytics dependencies using proven Rust libraries. This results in transparent computation graphs, easier debugging, automatic optimization opportunities, and confidence in calculation correctness.

### Multi-Modal Access Patterns
Unlike single-access-mode libraries, we provide embedded Rust, Python PyO3 bindings, REST API, and Polars integration from day one. This results in teams using the same analytics engine across all their workflows—from real-time trading to research notebooks—without custom integration work.

## Key Features

### Core Features
- **Push-Mode Analytics**: Analytics automatically update when new market data arrives, enabling real-time strategy execution
- **Pull-Mode Analytics**: Generate complete time series on-demand for historical analysis and backtesting
- **DAG-Based Computation**: Explicitly wire analytics together as a directed acyclic graph using open-source Rust libraries for transparent dependencies
- **Asset-Centric Data Model**: Assets modeled as objects rather than dataframe rows, providing intuitive API and optimized performance

### Integration Features
- **Embedded Rust Library**: Use the engine directly in Rust applications with zero-copy performance
- **Python PyO3 Bindings**: Direct Python access to the Rust engine for maximum performance in research workflows
- **REST API Server**: Query analytics by asset key and date range from distributed systems and web applications
- **Polars Dataframe Export**: Analytics returned as Polars dataframes in Python for seamless integration with data science workflows

### Advanced Features
- **Distributed & Scalable**: Architecture designed for horizontal scaling across multiple nodes
- **Real-Time Strategy Updates**: Strategy outputs update automatically as new analytics are computed in push mode
- **Historical Research Access**: Researchers can query precomputed analytics for specific assets and date ranges via Python API

