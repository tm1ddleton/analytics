# Appendix A.1: Component Catalog (Detailed)

This document provides detailed implementation specifications for all components in the index calculation framework.

## Calculators (Pure Functions)

### Core Mathematical Functions

#### `weighted_sum(values: &[f64], weights: &[f64]) -> f64`
**Purpose**: Calculate weighted sum of values

**Location**: `src/analytics/calculators.rs`

**Formula**: `Σ w_i × value_i`

**Implementation**:
```rust
pub fn weighted_sum(values: &[f64], weights: &[f64]) -> f64 {
    if values.len() != weights.len() {
        return f64::NAN;
    }
    values.iter()
        .zip(weights.iter())
        .map(|(v, w)| v * w)
        .sum()
}
```

**Usage**: Used by `WeightedSumMergeDefinition`, `ExcessReturnDefinition`, `IndexLevelDefinition`

---

#### `compound_return(previous_level: f64, current_return: f64) -> f64`
**Purpose**: Compound returns into levels

**Location**: `src/analytics/calculators.rs`

**Formula**: `previous_level × (1 + current_return)`

**Implementation**:
```rust
pub fn compound_return(previous_level: f64, current_return: f64) -> f64 {
    previous_level * (1.0 + current_return)
}
```

**Usage**: Used by `ReturnCompounderDefinition`, `CashAssetDefinition`

---

#### `population_std_dev(values: &[f64]) -> f64`
**Purpose**: Calculate population standard deviation

**Location**: `src/analytics/calculators.rs`

**Formula**: `sqrt(Σ(x_i - μ)² / n)` where `μ = mean(values)`

**Usage**: Used by `StdDevVolatilityAnalytic` container

---

#### `root_mean_square(values: &[f64]) -> f64`
**Purpose**: Calculate root mean square (for realized volatility)

**Location**: `src/analytics/calculators.rs`

**Formula**: `sqrt(Σ(x_i²) / n)` (no mean subtraction)

**Implementation**:
```rust
pub fn root_mean_square(values: &[f64]) -> f64 {
    let valid_values: Vec<f64> = values.iter().copied().filter(|v| !v.is_nan()).collect();
    if valid_values.is_empty() {
        return f64::NAN;
    }
    let n = valid_values.len() as f64;
    let sum_of_squares = valid_values.iter().map(|&value| value.powi(2)).sum::<f64>();
    (sum_of_squares / n).sqrt()
}
```

**Usage**: Used by `RealizedVolatilityAnalytic` container

---

#### `max_value(values: &[f64]) -> f64`
**Purpose**: Maximum value combiner

**Location**: `src/analytics/calculators.rs`

**Usage**: Used by `MaxVolatilityDefinition`

---

#### `min_value(values: &[f64]) -> f64`
**Purpose**: Minimum value combiner

**Location**: `src/analytics/calculators.rs`

---

### Option Pricing Functions

#### `option_pricing(...) -> OptionPricing`
**Purpose**: Calculate option premium and all Greeks

**Location**: `src/analytics/calculators.rs`

**Returns**: `OptionPricing` struct containing:
- `premium: f64`
- `delta: f64`
- `gamma: f64`
- `vega: f64`
- `theta: f64`
- `rho: f64`

**Usage**: Used by `OptionPricingDefinition`

---

#### `implied_volatility(optimizer: OptimizationRoutine, ...) -> f64`
**Purpose**: Calculate implied volatility with pluggable optimization routine

**Location**: `src/analytics/calculators.rs`

**Optimization Routines**:
- Newton-Raphson with bisection
- Secant method
- Binary search

**Usage**: Used by `ImpliedVolatilityDefinition`

---

#### `forward_and_discount_factor(...) -> (f64, f64)`
**Purpose**: Calculate forward price and discount factor using linear regression and put/call parity

**Location**: `src/analytics/calculators.rs`

**Usage**: Used by option pricing calculations

---

#### `twap(prices: &[f64], timestamps: &[DateTime<Utc>]) -> f64`
**Purpose**: Calculate time-weighted average price

**Location**: `src/analytics/calculators.rs`

**Usage**: Used for option price averaging

---

#### `portfolio_mark_to_market(positions: &[OptionPosition], prices: &[OptionPricing]) -> f64`
**Purpose**: Calculate portfolio mark-to-market value

**Location**: `src/analytics/calculators.rs`

**Usage**: Used by `PortfolioMarkToMarketDefinition`

---

### Volatility Functions

#### `volatility_targeting_exposure(target_vol: f64, realized_vol: f64, previous_exposure: f64) -> f64`
**Purpose**: Calculate volatility targeting exposure

**Location**: `src/analytics/calculators.rs`

**Formula**: `exposure_t = exposure_{t-1} × (target_vol / realized_vol_t)`

**Usage**: Used by `VolatilityTargetingExposureDefinition`

---

## Containers (Stateless Wrappers)

### Return Analytics

#### `LogReturnAnalytic`
**Purpose**: Calculate logarithmic returns

**Location**: `src/analytics/containers.rs`

**Trait**: `ReturnAnalytic`

**Calculator**: `log_return_value(current, lagged)`

**Usage**: Used by return calculation definitions

---

#### `ArithReturnAnalytic`
**Purpose**: Calculate arithmetic returns

**Location**: `src/analytics/containers.rs`

**Trait**: `ReturnAnalytic`

**Formula**: `(current / lagged) - 1.0`

**Usage**: Used by return calculation definitions

---

### Volatility Analytics

#### `StdDevVolatilityAnalytic`
**Purpose**: Calculate standard deviation volatility

**Location**: `src/analytics/containers.rs`

**Trait**: `VolatilityAnalytic`

**Calculator**: `population_std_dev(window)`

**Usage**: Used by `VolatilityDefinition`

---

#### `RealizedVolatilityAnalytic`
**Purpose**: Calculate realized volatility (RMS)

**Location**: `src/analytics/containers.rs`

**Trait**: `VolatilityAnalytic`

**Calculator**: `root_mean_square(window)`

**Usage**: Used by `RealizedVolatilityDefinition`

---

## Executors

### `WindowedAnalyticExecutor`
**Purpose**: Rolling window calculations

**Location**: `src/analytics/registry.rs`

**Characteristics**:
- Maintains rolling window of values
- Applies container's `compute()` function to window
- Supports annualization factor (default: `sqrt(252)`)

**Usage**: Used by `VolatilityDefinition`, `RealizedVolatilityDefinition`, `LagDefinition`

---

### `RecursiveExecutor`
**Purpose**: Stateful, iterative calculations

**Location**: `src/analytics/registry.rs`

**Characteristics**:
- Maintains previous state (e.g., `Option<f64>` for levels, `Option<IndexComposition>` for indices)
- Applies calculator function with previous state and current value
- Stores new state for next iteration

**State Management**:
- State stored per-node in DAG
- Persists across push-mode iterations
- Can be serialized for hot start

**Usage**: Used by `ReturnCompounderDefinition`, `CashAssetDefinition`, `IndexLevelDefinition`, `BaseIndexDefinition`

---

### `MergeExecutor`
**Purpose**: Combines multiple parent nodes

**Location**: `src/analytics/registry.rs`

**Characteristics**:
- Takes multiple parent outputs
- Applies merge function (e.g., `weighted_sum`, `max_value`)
- No state maintained

**Usage**: Used by `ExcessReturnDefinition`, `WeightedSumMergeDefinition`, `MaxVolatilityDefinition`

---

### `DataProviderExecutor`
**Purpose**: Pulls data from data sources

**Location**: `src/analytics/registry.rs`

**Characteristics**:
- Queries `DataProvider` trait for time-series data
- No calculation performed
- Used for data input nodes

**Usage**: Used by `StartingStateDefinition`, data input nodes

---

## Definitions (Generic)

### Return Definitions

#### `ExcessReturnDefinition`
**Purpose**: Calculate excess return (asset return - cash return)

**Components**:
- **Calculator**: `weighted_sum()` with weights `[1, -1]`
- **Executor**: `MergeExecutor` (2 parents: asset return, cash return)
- **Container**: None (direct calculator usage)

**Formula**: `excess_return = asset_return - cash_return = weighted_sum([asset_return, cash_return], [1, -1])`

**Usage**: Used in SOLSTAE, AIPEX5 indices

---

#### `ReturnCompounderDefinition`
**Purpose**: Compound returns into levels

**Components**:
- **Calculator**: `compound_return(previous_level, current_return)`
- **Executor**: `RecursiveExecutor` (maintains previous level)
- **Container**: None (direct calculator usage)

**Formula**: `level_t = level_{t-1} × (1 + return_t)`

**Usage**: Used by all index level calculations

---

### Volatility Definitions

#### `VolatilityDefinition`
**Purpose**: Calculate rolling volatility

**Components**:
- **Container**: `StdDevVolatilityAnalytic`
- **Executor**: `WindowedAnalyticExecutor` (rolling window)
- **Calculator**: `population_std_dev()` (via container)

**Annualization**: Applies `sqrt(annualization_factor)` (default: `sqrt(252)`)

**Usage**: Used for volatility calculations

---

#### `RealizedVolatilityDefinition`
**Purpose**: Calculate realized volatility (RMS)

**Components**:
- **Container**: `RealizedVolatilityAnalytic`
- **Executor**: `WindowedAnalyticExecutor` (rolling window)
- **Calculator**: `root_mean_square()` (via container)

**Usage**: Used by AIPEX5 index (21-day and 63-day)

---

#### `MaxVolatilityDefinition`
**Purpose**: Take maximum of multiple volatility inputs

**Components**:
- **Calculator**: `max_value()`
- **Executor**: `MergeExecutor` (multiple volatility parents)
- **Container**: None (direct calculator usage)

**Usage**: Used by AIPEX5 index (max of 21-day and 63-day volatilities)

---

#### `VolatilityTargetingExposureDefinition`
**Purpose**: Calculate volatility targeting exposure

**Components**:
- **Calculator**: `volatility_targeting_exposure()`
- **Executor**: `RecursiveExecutor` (maintains previous exposure)
- **Container**: None (direct calculator usage)

**Usage**: Used by AIPEX5 index

---

### Asset Definitions

#### `CashAssetDefinition`
**Purpose**: Model cash/funding as an asset that accrues interest

**Components**:
- **Calculator**: Compounding function `(1 + rate × DCF/365)`
- **Executor**: `RecursiveExecutor` (maintains previous cash level)
- **Container**: None (direct calculator usage)

**Formula**: `Cash_t = Cash_{t-1} × (1 + rate × DCF/365)`

**Usage**: Used for SOFR/LIBOR cash assets, funding costs

---

#### `TotalReturnLevelDefinition`
**Purpose**: Calculate total return level (portfolio + cash)

**Components**:
- **Calculator**: `weighted_sum()` with weights `[1, 1]`
- **Executor**: `RecursiveExecutor` (maintains previous level)
- **Container**: None (direct calculator usage)

**Usage**: Used for portfolio valuation

---

#### `ExcessReturnLevelDefinition`
**Purpose**: Calculate excess return level

**Components**:
- Composed from `TotalReturnLevelDefinition` - `CashAssetDefinition` + `ExcessReturnDefinition` + `ReturnCompounderDefinition`

**Usage**: Used for excess return indices

---

### Futures Definitions

#### `RollingFuturesLevelDefinition`
**Purpose**: Calculate rolling futures level with contract transitions

**Components**:
- **Calculator**: `weighted_sum()` for active/next contract returns
- **Executor**: `RecursiveExecutor` (maintains previous level)
- **Container**: None (direct calculator usage)

**Formula**:
```
ActiveReturn_t = (Px_active_t / Px_active_{t-1} - 1)
NextReturn_t = (Px_next_t / Px_next_{t-1} - 1)
FuturesReturn_t = weighted_sum([ActiveReturn_t, NextReturn_t], [w_active, w_next]) × FXConversion_t
RFL_t = RFL_{t-1} × (1 + FuturesReturn_t)
```

**Usage**: Used by SOLSTAE index for futures components

---

### Option Definitions

#### `OptionPricingDefinition`
**Purpose**: Calculate option premium and Greeks

**Components**:
- **Calculator**: `option_pricing()`
- **Executor**: `DataProviderExecutor` (pulls option data)
- **Container**: None (direct calculator usage)

**Usage**: Used by variance replication index

---

#### `OptionDeltaDefinition`
**Purpose**: Calculate option delta

**Components**:
- Uses `OptionPricingDefinition` output (delta field)
- **Executor**: `PassthroughExecutor`
- **Container**: None

**Usage**: Used for delta hedging calculations

---

#### `PortfolioMarkToMarketDefinition`
**Purpose**: Calculate portfolio mark-to-market value

**Components**:
- **Calculator**: `portfolio_mark_to_market()`
- **Executor**: `RecursiveExecutor` (maintains portfolio state)
- **Container**: None (direct calculator usage)

**Usage**: Used by variance replication index

---

### State Definitions

#### `StartingStateDefinition`
**Purpose**: Provide initial index level and composition

**Components**:
- **Calculator**: None (data source)
- **Executor**: `DataProviderExecutor` (loads from database or config)
- **Container**: `StartingStateAnalytic`

**Output**: `IndexComposition` struct

**Logic**:
1. Query database for latest checkpoint before requested start date
2. If checkpoint found: return checkpoint level and composition (hot start)
3. If no checkpoint: return config values (cold start)

**Usage**: Used by all index calculations

---

## Definitions (Index-Specific)

### Variance Replication Index

#### `VarianceStrikeCalculator`
**Purpose**: Calculate variance strike from option prices

**Location**: Factory-generated from rulebook

**Dependencies**: Option prices (TWAP)

**Usage**: Variance replication index only

---

#### `VarianceReplicationPortfolioBuilder`
**Purpose**: Build option portfolio based on variance strike

**Location**: Factory-generated from rulebook

**Dependencies**: Variance strike, option prices

**Usage**: Variance replication index only

---

### AIPEX5 Index

#### `RealizedVolatility21DayDefinition`
**Purpose**: 21-day realized volatility

**Components**:
- **Container**: `RealizedVolatilityAnalytic`
- **Executor**: `WindowedAnalyticExecutor` (window: 21)
- **Calculator**: `root_mean_square()` (via container)

**Usage**: AIPEX5 index only

---

#### `RealizedVolatility63DayDefinition`
**Purpose**: 63-day realized volatility

**Components**:
- **Container**: `RealizedVolatilityAnalytic`
- **Executor**: `WindowedAnalyticExecutor` (window: 63)
- **Calculator**: `root_mean_square()` (via container)

**Usage**: AIPEX5 index only

---

## Component Relationships

### Calculator → Container → Executor → Definition

**Example Flow**:
1. **Calculator**: `population_std_dev()` - pure math function
2. **Container**: `StdDevVolatilityAnalytic` - implements `VolatilityAnalytic` trait
3. **Executor**: `WindowedAnalyticExecutor` - applies container to rolling window
4. **Definition**: `VolatilityDefinition` - wires everything together, registers in DAG

### Direct Calculator Usage

Some definitions use calculators directly without containers:
- `ExcessReturnDefinition`: Uses `weighted_sum()` calculator directly with `MergeExecutor`
- `ReturnCompounderDefinition`: Uses `compound_return()` calculator directly with `RecursiveExecutor`

### Container Usage

Containers are used when:
- Multiple implementations of the same trait are needed (e.g., `LogReturnAnalytic` vs `ArithReturnAnalytic`)
- Executors need a trait interface to work with different calculation types
- Code organization and reusability benefits from trait abstraction

