# Analytics Module Architecture

This module implements the core analytics computation engine with a clear separation of concerns across four distinct layers: **definitions**, **containers**, **executors**, and **calculators**.

## Philosophy

The analytics module is designed with a four-layer architecture that separates mathematical operations, input requirements, execution orchestration, and DAG integration. This design enables:

- **Composability**: Calculators can be reused across different analytics
- **Testability**: Each layer can be tested independently
- **Maintainability**: Clear boundaries make the codebase easier to understand and modify
- **Performance**: Pure functions enable optimization and parallelization

## Architecture Layers

### 1. Definitions (`registry.rs`)

**Top-level integration that wires everything together and registers analytics in the DAG.**

Definitions are the integration layer—they tie containers and executors together and register the complete analytic in the DAG system. A definition:

- **Wires components**: Combines a container (what the analytic is) with an executor (how it runs)
- **Defines dependencies**: Specifies what parent nodes are required in the DAG
- **Registers the analytic**: Makes it available in the `AnalyticRegistry` for DAG construction
- **Provides execution**: Supplies the executor that will be used at runtime

**Examples:**
- `ReturnsDefinition` - Wires `LogReturnAnalytic` container with `MergeExecutor`
- `VolatilityDefinition` - Wires `StdDevVolatilityAnalytic` container with `WindowedAnalyticExecutor`
- `LagDefinition` - Wires lag computation with `WindowedAnalyticExecutor`

**Key Characteristics:**
- Implements `AnalyticDefinition` trait
- Specifies `AnalyticType` for DAG node identification
- Defines dependency resolution (what parent nodes to create)
- Provides the executor instance
- Registered in `AnalyticRegistry::new()` to make it available

```rust
// Example: Definition wires everything together
struct ReturnsDefinition {
    executor: Box<dyn AnalyticExecutor>,
}

impl ReturnsDefinition {
    fn new() -> Self {
        let primitive = Arc::new(LogReturnAnalytic);
        ReturnsDefinition {
            executor: Box::new(MergeExecutor::new(
                vec![AnalyticType::DataProvider, AnalyticType::Lag],
                {
                    let primitive = primitive.clone();
                    move |node, aligned_points| {
                        match (aligned_points.get(0), aligned_points.get(1)) {
                            (Some(Some(price)), Some(Some(lag))) => {
                                let asset = node.assets.first();
                                primitive.compute(asset, price.close_price, lag.close_price)
                            }
                            _ => f64::NAN,
                        }
                    }
                },
            )),
        }
    }
}

impl AnalyticDefinition for ReturnsDefinition {
    fn analytic_type(&self) -> AnalyticType {
        AnalyticType::Returns
    }
    
    fn node_type(&self) -> &'static str {
        "returns"
    }
    
    fn dependencies(&self, key: &NodeKey) -> Result<Vec<NodeKey>, DagError> {
        // Define what parent nodes are needed (DataProvider and Lag)
        Ok(vec![/* parent node keys */])
    }
    
    fn executor(&self) -> &dyn AnalyticExecutor {
        self.executor.as_ref()
    }
}
```

### 2. Containers (`containers.rs`)

**Traits and structs that define what an analytic is.**

Containers define the identity and interface of each analytic. A container represents a complete analytic with:

- **Analytic identity**: A name/identifier for the analytic
- **Input requirements**: What data the analytic needs (current + lagged, or a window)
- **Computation interface**: How to invoke the calculation with those inputs

**Examples:**
- `ReturnAnalytic` trait with `LogReturnAnalytic` and `ArithReturnAnalytic` implementations
- `VolatilityAnalytic` trait with `StdDevVolatilityAnalytic` implementation

**Key Characteristics:**
- Defines the complete analytic interface
- Wraps calculator functions with a consistent interface
- Defines the input contract for each analytic type
- May specify asset-aware or asset-agnostic behavior
- Provides the name/identifier for the analytic

```rust
// Example: Container defines the analytic
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

**Orchestration layer that runs analytics and coordinates execution.**

Executors are the execution engine that:

- **Coordinate data flow**: Pull data from parent nodes in the DAG
- **Apply analytics**: Invoke containers with the right inputs
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
- Invoke container's `compute()` method with extracted values

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

### 4. Calculators (`calculators.rs`)

**Pure stateless functions that define the mathematics.**

Calculators are the implementation layer—they contain the mathematical logic that operates on raw input data. These functions are:

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
- Called by containers to perform the actual computation

```rust
// Example: Pure mathematical function
pub fn log_return_value(current: f64, lagged: f64) -> f64 {
    if lagged <= 0.0 || current <= 0.0 || lagged.is_nan() || current.is_nan() {
        return 0.0;
    }
    (current / lagged).ln()
}
```

## Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                    DAG System                            │
│                                                          │
│  Definition registered in AnalyticRegistry:              │
│    • Provides AnalyticType                              │
│    • Defines dependencies                               │
│    • Supplies executor                                  │
│                                                          │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
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
│  Container defines the analytic:                         │
│    • Analytic identity (name)                            │
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

**1. Definition** (`registry.rs`):
```rust
struct ReturnsDefinition {
    executor: Box<dyn AnalyticExecutor>,
}

impl AnalyticDefinition for ReturnsDefinition {
    fn analytic_type(&self) -> AnalyticType {
        AnalyticType::Returns
    }
    
    fn dependencies(&self, key: &NodeKey) -> Result<Vec<NodeKey>, DagError> {
        // Returns needs DataProvider and Lag nodes
        Ok(vec![/* parent nodes */])
    }
    
    fn executor(&self) -> &dyn AnalyticExecutor {
        self.executor.as_ref()
    }
}
```
The definition wires everything together and registers the analytic in the DAG system.

**2. Container** (`containers.rs`):
```rust
pub struct LogReturnAnalytic;
impl ReturnAnalytic for LogReturnAnalytic {
    fn compute(&self, _asset: Option<&AssetKey>, current: f64, lagged: f64) -> f64 {
        log_return_value(current, lagged)  // Uses calculator
    }
}
```
The container defines what "log return" is: an analytic that takes current and lagged prices.

**3. Executor** (`registry.rs`):
- `ReturnsDefinition` uses `MergeExecutor` 
- Executor gets current price from DataProvider parent node
- Executor gets lagged price from Lag parent node
- Executor extracts `f64` values from `TimeSeriesPoint` outputs
- Executor calls `LogReturnAnalytic.compute(asset, current, lagged)` via closure
- Returns new `TimeSeriesPoint` with computed return
The executor orchestrates how the analytic runs in the DAG.

**4. Calculator** (`calculators.rs`):
```rust
pub(crate) fn log_return_value(current: f64, lagged: f64) -> f64 {
    (current / lagged).ln()
}
```
The calculator provides the mathematical implementation.

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
├── containers.rs       # Traits and structs defining analytic interfaces
├── registry.rs         # Definitions, executors, and registry
├── calculators.rs      # Pure mathematical functions
├── lag.rs             # Lag-specific analytics
├── windows.rs         # Window management utilities
├── testing.rs         # Test helpers
└── README.md          # This file
```

## Adding a New Analytic

To add a new analytic, follow these steps:

1. **Create container** in `containers.rs` that defines the analytic (trait and struct)
2. **Add calculator function** to `calculators.rs` (if needed) for the mathematical implementation
3. **Create definition** in `registry.rs` that:
   - Creates an executor (or uses an existing one like `MergeExecutor`, `WindowedAnalyticExecutor`)
   - Wires the container into the executor
   - Implements `AnalyticDefinition` trait with dependency resolution
4. **Register definition** in `AnalyticRegistry::new()` to make it available in the DAG

See existing implementations (Returns, Volatility, Lag) as examples.

