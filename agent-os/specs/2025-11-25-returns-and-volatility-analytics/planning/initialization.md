# Returns and Volatility Analytics - Feature Initialization

## Initial Description

Returns and Volatility Analytics - Stateless calculation functions for returns and volatility that work with windowed data and integrate with the DAG computation framework. 

Calculations should be fed data externally in windowing containers, with returns using 2-point slices and volatility using N-day windows. 

Should support queries like '10-day volatility for AAPL between dates' with automatic dependency resolution.

## Key Concepts

1. **Stateless Functions**: Calculations should be pure functions without internal state
2. **Windowing Container**: Data is passed externally in window slices
3. **Returns Calculation**: Uses 2-point data slices (current, previous) - NaN/None for first point
4. **Volatility Calculation**: Uses N-day windows (e.g., 10-day, 30-day)
5. **DAG Integration**: Should work with the DAG computation framework
6. **Dependency Resolution**: Automatically resolve and create required nodes
7. **Date Range Queries**: Support queries with arbitrary start/end dates

## Date
2025-11-25

