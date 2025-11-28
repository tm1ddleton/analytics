# Analytics Module Architecture

This module implements the core analytics computation engine with a clear separation of concerns across three distinct layers: **calculators**, **containers**, and **executors**.

## Philosophy

The analytics module is designed with a three-layer architecture that separates mathematical operations, input requirements, and execution orchestration. This design enables:

- **Composability**: Calculators can be reused across different analytics
- **Testability**: Each layer can be tested independently
- **Maintainability**: Clear boundaries make the codebase easier to understand and modify
- **Performance**: Pure functions enable optimization and parallelization

## Architecture Layers

### 1. Calculators (`calculators.rs`)

**Pure stateless functions that define the mathematics.**

Calculators are the foundation layer—they contain the mathematical logic that operates on raw input data. These functions are:

- **Stateless**: No internal state, pure transformations
- **Side-effect free**: Deterministic outputs for given inputs
- **Mathematical primitives**: Focus on the computation itself

**Examples:**
- `log_return_value(current, lagged)` - Computes log return between two prices
- `population_std_dev(values)` - Calculates population standard deviation
- `ema_step(previous, value, lambda)` - Single step of exponential moving average
- `log_return_window(window)` - Log return over a window of prices

**Key Characteristics:**
- Operate on `f64` primitives or slices of `f64`
- Handle edge cases (NaN, zero, empty inputs)
- No knowledge of assets, dates, or DAG structure
- Highly testable and reusable

```rust
// Example: Pure mathematical function
pub fn log_return_value(current: f64, lagged: f64) -> f64 {
    if lagged <= 0.0 || current <= 0.0 || lagged.is_nan() || current.is_nan() {
        return 0.0;
    }
    (current / lagged).ln()
}
```

### 2. Containers (`containers.rs`)

**Traits and structs that define what inputs are required for a given analytic.**

Containers bridge calculators with the analytics system by defining:

- **Input requirements**: What data the analytic needs (current + lagged, or a window)
- **Computation interface**: How to invoke the calculator with those inputs
- **Analytic identity**: A name/identifier for the analytic

**Examples:**
- `ReturnAnalytic` trait with `LogReturnAnalytic` and `ArithReturnAnalytic` implementations
- `VolatilityAnalytic` trait with `StdDevVolatilityAnalytic` implementation

**Key Characteristics:**
- Wraps calculator functions with a consistent interface
- Defines the input contract for each analytic type
- May specify asset-aware or asset-agnostic behavior
- Provides the name/identifier for the analytic

```rust
// Example: Container defines input requirements
pub trait ReturnAnalytic: Send + Sync {
    fn name(&self) -> &'static str;
    fn compute(&self, asset: Option<&AssetKey>, current: f64, lagged: f64) -> f64;
}

pub struct LogReturnAnalytic;

impl ReturnAnalytic for LogReturnAnalytic {
    fn name(&self) -> &'static str { "log_return" }
    fn compute(&self, _asset: Option<&AssetKey>, current: f64, lagged: f64) -> f64 {
        log_return_value(current, lagged)  // Delegates to calculator
    }
}
```

### 3. Executors (`registry.rs`)

**Orchestration layer that stitches nodes together and runs analytics on inputs and outputs.**

Executors are the execution engine that:

- **Coordinate data flow**: Pull data from parent nodes in the DAG
- **Apply analytics**: Invoke containers/calculators with the right inputs
- **Handle modes**: Support both push-mode (incremental) and pull-mode (batch) execution
- **Manage state**: Maintain windows, buffers, and intermediate results

**Examples:**
- `WindowedAnalyticExecutor` - Manages sliding windows for volatility/lag calculations
- `MergeExecutor` - Combines outputs from multiple parent nodes
- `DataProviderExecutor` - Queries data sources and provides time-series to the DAG

**Key Characteristics:**
- Know about DAG structure and node dependencies
- Handle push-mode (point-by-point) and pull-mode (batch) execution
- Manage stateful operations (windows, buffers, aggregations)
- Coordinate between multiple parent nodes

```rust
// Example: Executor orchestrates computation
pub trait AnalyticExecutor: Send + Sync {
    fn execute_push(
        &self,
        node: &Node,
        parent_outputs: &[ParentOutput],
        timestamp: DateTime<Utc>,
        value: f64,
    ) -> Result<NodeOutput, DagError>;
    
    fn execute_pull(
        &self,
        node: &Node,
        parent_outputs: &[ParentOutput],
        date_range: &DateRange,
        provider: &dyn DataProvider,
    ) -> Result<Vec<TimeSeriesPoint>, DagError>;
}
```

## Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                    DAG Execution                         │
│                                                          │
│  Executor coordinates:                                   │
│    • Gathers inputs from parent nodes                    │
│    • Extracts values (f64) from time-series points       │
│    • Invokes Container.compute()                         │
│                                                          │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│                  Container Layer                         │
│                                                          │
│  Container defines:                                      │
│    • Required inputs (current+lagged or window)          │
│    • Calls Calculator function                           │
│    • Returns computed value                              │
│                                                          │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│                  Calculator Layer                        │
│                                                          │
│  Calculator performs:                                    │
│    • Pure mathematical transformation                    │
│    • Operates on f64 values                             │
│    • Returns computed result                            │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Example: Computing Returns

**1. Calculator** (`calculators.rs`):
```rust
pub(crate) fn log_return_value(current: f64, lagged: f64) -> f64 {
    (current / lagged).ln()
}
```

**2. Container** (`containers.rs`):
```rust
pub struct LogReturnAnalytic;
impl ReturnAnalytic for LogReturnAnalytic {
    fn compute(&self, _asset: Option<&AssetKey>, current: f64, lagged: f64) -> f64 {
        log_return_value(current, lagged)  // Uses calculator
    }
}
```

**3. Executor** (`registry.rs`):
- `ReturnsDefinition` uses `MergeExecutor` 
- Executor gets current price from one parent node
- Executor gets lagged price from another parent node (Lag node)
- Executor calls `LogReturnAnalytic.compute(current, lagged)`
- Returns new `TimeSeriesPoint` with computed return

## Benefits of This Architecture

1. **Separation of Concerns**: Each layer has a single, clear responsibility
2. **Reusability**: Calculators can be shared across multiple analytics
3. **Testability**: Each layer can be unit tested independently
4. **Flexibility**: New analytics can be added by combining existing calculators
5. **Performance**: Pure functions enable compiler optimizations and parallelization
6. **Maintainability**: Changes to math logic don't affect execution logic and vice versa

## File Organization

```
src/analytics/
├── calculators.rs      # Pure mathematical functions
├── containers.rs       # Traits and structs defining analytic interfaces
├── registry.rs         # Executors and analytic definitions
├── lag.rs             # Lag-specific analytics
├── windows.rs         # Window management utilities
├── testing.rs         # Test helpers
└── README.md          # This file
```

## Adding a New Analytic

To add a new analytic, follow these steps:

1. **Add calculator function** to `calculators.rs` (if needed)
2. **Create container** in `containers.rs` that wraps the calculator
3. **Implement executor** in `registry.rs` that orchestrates the computation
4. **Register definition** in `AnalyticRegistry` to wire it into the DAG

See existing implementations (Returns, Volatility, Lag) as examples.

