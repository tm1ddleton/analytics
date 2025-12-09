# Index Calculation Framework: Architecture Proposal

## Executive Summary

This document outlines the architecture for implementing index calculations on the analytics platform. The framework is designed to support multiple index types through **generic, reusable components** that can be composed to build complex indices.

**Key Value Propositions**:
- **Reusability**: Generic analytics components reduce development time for new indices
- **Scalability**: Single persistence model supports unlimited indices without migrations
- **Maintainability**: Clear separation between generic and index-specific logic
- **Extensibility**: New indices can be built by composing existing components

**Supported Indices**:
1. **SOLSTAE Index**: Systematic Trend Alpha Replicator (ETF/futures-based)
2. **Variance Replication Index**: Options-based variance replication
3. **AIPEX5 Index**: AI Powered US Equity Index (volatility-targeted)

---

## Part 1: Framework Architecture

### 1.1 Core Architecture Principles

**Three-Layer Architecture**:
1. **Calculators**: Pure mathematical functions (no state, no dependencies)
2. **Containers**: Stateless wrappers that provide trait-based interfaces for calculators
3. **Executors**: Execution strategies (how calculations run - push/pull, windowing, recursion)
4. **Definitions**: DAG nodes that wire calculators + containers + executors together

**Key Design Principles**:
- **Decomposition via Transformation**: Sometimes transforming to returns makes calculations decomposable (e.g., excess returns, AIPEX5), but not always (e.g., variance index works in quantities/level space)
- **Composition Over Specialization**: Build complex indices from simple, composable nodes
- **Generic Before Specific**: Maximize reuse of generic components before creating index-specific logic

### 1.2 Component Responsibilities

#### Calculators (Pure Functions)
**Location**: `src/analytics/calculators.rs`

**Responsibility**: Pure mathematical functions with no side effects
- Input: Values and parameters
- Output: Calculated result
- No state, no dependencies, no I/O

**What Calculators Don't Care About**:
- **Domain**: A mean is a mean whether calculated over time, cross-sectionally, or across assets
- **Dimensions**: The same mathematical operation applies regardless of whether data is time series, cross-sectional, or multi-dimensional
- **Business Context**: Calculators are domain-agnostic pure math functions

**Examples**:
- `weighted_sum(values, weights) -> f64` - Works for any weighted aggregation
- `compound_return(previous_level, return) -> f64` - Works for any compounding
- `root_mean_square(values) -> f64` - Works for any RMS calculation
- `option_pricing(...) -> OptionPricing` - Mathematical pricing formula

#### Containers (Stateless Wrappers)
**Location**: `src/analytics/containers.rs`

**Responsibility**: Provide trait-based interfaces that wrap calculator functions
- Implement traits like `ReturnAnalytic`, `VolatilityAnalytic`
- Bridge calculators and executors with a common interface
- No state, no dependencies - just trait implementations

**What Containers Don't Care About**:
- **Execution Strategy**: Containers don't know how calculations are executed (push/pull, windowing, etc.)
- **DAG Structure**: Containers don't know about dependencies or DAG wiring
- **Business Context**: Containers are domain-agnostic trait implementations

**Key Characteristics**:
- Containers are stateless trait implementations
- They wrap calculator functions with a common interface
- Multiple containers can use the same calculator
- They enable executors to work with different calculation types through traits

**Examples**:
- `LogReturnAnalytic`: Implements `ReturnAnalytic` trait, wraps `log_return_value()` calculator
- `StdDevVolatilityAnalytic`: Implements `VolatilityAnalytic` trait, wraps `population_std_dev()` calculator
- `RealizedVolatilityAnalytic`: Implements `VolatilityAnalytic` trait, wraps `root_mean_square()` calculator

#### Executors (Execution Strategies)
**Location**: `src/analytics/registry.rs`

**Responsibility**: Define how calculations execute
- **WindowedAnalyticExecutor**: Rolling window calculations
- **RecursiveExecutor**: Stateful, iterative calculations (maintains previous state)
- **MergeExecutor**: Combines multiple parent nodes
- **DataProviderExecutor**: Pulls data from data sources

**What Executors Don't Care About**:
- **The Calculation**: Executors don't know or care what calculation is being performed
- **Business Logic**: They only care about dimensional orientation of data and caching strategies
- **Domain**: Execution patterns are domain-agnostic

**Key Characteristics**:
- Executors are generic and reusable
- They orchestrate execution patterns, not business logic
- Multiple definitions can use the same executor
- They handle data flow, windowing, state management, and caching

#### Definitions (DAG Nodes)
**Location**: `src/analytics/registry.rs`

**Responsibility**: Wire components together to create DAG nodes
- Combine calculator + container + executor
- Define dependencies (parent nodes)
- Register in `AnalyticRegistry`

**What Definitions Care About**:
- **Domain**: Definitions understand the business context (e.g., "excess return", "volatility", "option pricing")
- **DAG Structure**: How components are wired together for a specific use case
- **Data Semantics**: What the inputs and outputs represent in business terms

**Examples**:
- `ExcessReturnDefinition`: Uses `WeightedSum` calculator + `MergeExecutor` - understands "excess return" domain
- `VolatilityDefinition`: Uses `StdDevVolatilityAnalytic` container + `WindowedAnalyticExecutor` - understands volatility domain
- `ReturnCompounderDefinition`: Uses `compound_return` calculator + `RecursiveExecutor` - understands return compounding domain

**Note**: Some definitions use containers (when a trait interface is needed), while others use calculators directly (when no trait abstraction is needed).

### 1.3 Generic Analytics Components

#### Core Mathematical Functions
- **Weighted Sum**: `weighted_sum()` - Weighted aggregation
- **Compound Return**: `compound_return()` - Compound returns into levels
- **Max/Min Combiners**: `max_value()`, `min_value()` - Combine multiple values
- **Root Mean Square**: `root_mean_square()` - For realized volatility

#### Return Calculations
- **Arithmetic Return**: `ArithReturnAnalytic` - Standard return calculation
- **Log Return**: `LogReturnAnalytic` - Logarithmic return calculation
- **Excess Return**: `ExcessReturnDefinition` - Asset return minus cash return (WeightedSum [1, -1])

#### Volatility Calculations
- **Standard Deviation**: `population_std_dev()` - Population standard deviation
- **Realized Volatility**: `root_mean_square()` - Realized volatility (no mean subtraction)
- **Volatility Targeting**: `volatility_targeting_exposure()` - Dynamic exposure calculation

#### Asset Management
- **Cash Asset**: `CashAssetDefinition` - Interest accrual (treats cash as asset)
- **Return Compounder**: `ReturnCompounderDefinition` - Compound returns to levels
- **Total Return Level**: `TotalReturnLevelDefinition` - Portfolio + Cash
- **Excess Return Level**: Composed from Total Return - Cash Return, then compounded

#### Options Infrastructure
- **Option Pricing**: `option_pricing()` - Premium + all Greeks
- **Implied Volatility**: `implied_volatility()` - With pluggable optimization routines
- **Forward/Discount Factor**: `forward_and_discount_factor()` - Put/call parity + linear regression
- **TWAP**: `twap()` - Time-weighted average price
- **Portfolio Mark-to-Market**: `portfolio_mark_to_market()` - Value option portfolio

#### Futures Infrastructure
- **Rolling Futures Level**: `RollingFuturesLevelDefinition` - Contract transitions
- **FX Conversion**: Generic FX conversion support

### 1.4 Persistence Strategy

**Single Table Approach**: All index data stored in `analytics` table
- **Scalability**: Supports unlimited indices without migrations
- **Flexibility**: JSON storage for composition data
- **Performance**: Key-based queries with date ranges

**Storage Format**:
- Level: Stored as scalar value
- Composition: Stored as JSON (weights or quantities)
- Checkpoints: Stored as regular analytics entries

### 1.5 Initial State and Hot Start

**Unified Approach**: `StartingState` node provides initial values
- Loads from database checkpoint (if available)
- Falls back to index configuration (if no checkpoint)
- Index calculation node is a regular DAG node (no special logic)

### 1.6 Registry Strategy

**Hybrid Approach**: Separate registries for generic and index-specific components

**Main Registry** (`AnalyticRegistry`):
- Contains all generic, reusable components
- Available to all indices
- Examples: `WeightedSum`, `ExcessReturnDefinition`, `VolatilityDefinition`

**Index Registry** (`IndexNodeRegistry`):
- Factory-generated from rulebook configuration
- Contains index-specific nodes and DAG structure
- Examples: `VarianceStrikeCalculator`, `VarianceReplicationPortfolioBuilder`
- Can support multiple indices with different rulebooks

**Benefits**:
- Clear separation of concerns
- Easy to add/remove indices without affecting main registry
- Supports multiple indices simultaneously

---

## Part 2: Index Implementations

### 2.1 SOLSTAE Index

**Type**: Systematic Trend Alpha Replicator Excess Return Index

**Key Components**:
- ETF Total Return calculation
- Futures rolling with contract transitions
- Transaction and replication costs (as assets)
- Base Index composition
- Excess return calculation

**DAG Structure**:
```
StartingState ──> BaseIndex ──> IndexLevel (WeightedSum of components)
                                ├── ETF Total Return
                                ├── Futures Rolling
                                ├── Transaction Costs (asset)
                                ├── Replication Costs (asset)
                                └── Cash Asset
```

**Generic Components Used**:
- `WeightedSum` calculator
- `RollingFuturesLevelDefinition`
- `ExcessReturnDefinition`
- `ReturnCompounderDefinition`
- `CashAssetDefinition`

### 2.2 Variance Replication Index

**Type**: Options-based variance replication

**Key Components**:
- Variance strike calculation (from option prices)
- Option portfolio management
- Delta hedging (treated as regular asset)
- Option cash flows
- Portfolio mark-to-market

**Note**: This index works in quantities/level space (not return space) because portfolio mark-to-market and position management are naturally expressed in terms of quantities and prices.

**Index-Specific Components**:
- **Variance Strike Calculator**: Calculates variance strike from option prices (index-specific)
- **Variance Replication Portfolio Builder**: Builds option portfolio based on variance strike (index-specific)

**Generic Components Used**:
- `OptionPricingDefinition`
- `OptionDeltaDefinition`
- `DeltaHedgePositionDefinition` (as regular asset)
- `OptionCashFlowsDefinition`
- `PortfolioMarkToMarketDefinition`
- `TotalReturnLevelDefinition`
- `ExcessReturnLevelDefinition`

**DAG Structure**:
```
Option Prices ──> Variance Strike Calculator ──> Portfolio Builder ──> Option Portfolio
                                                                      ├──> Option Delta ──> Delta Hedge Position (asset)
                                                                      ├──> Option Cash Flows
                                                                      └──> Portfolio Mark-to-Market
                                                                      
Total Return Level ──> Total Return ──┐
                                      ├──> Excess Return (WeightedSum [1, -1]) ──> Excess Return Level (ReturnCompounder)
Cash Asset ──> Cash Return ───────────┘
```

### 2.3 AIPEX5 Index

**Type**: Volatility-targeted dynamic exposure index

**Key Components**:
- Realized volatility calculation (21-day and 63-day)
- Volatility targeting exposure
- Dynamic exposure index level (all generic components)

**Index-Specific Components**:
- **Realized Volatility Nodes**: 21-day and 63-day (use generic `root_mean_square` calculator)
- **Max Volatility Combiner**: Max of 21-day and 63-day (uses generic `max_value` calculator)
- **Volatility Targeting Exposure**: Uses generic `volatility_targeting_exposure` calculator

**Generic Components Used**:
- `RealizedVolatilityAnalytic` (uses `root_mean_square`)
- `MaxVolatilityDefinition` (uses `max_value` + `MergeExecutor`)
- `VolatilityTargetingExposureDefinition`
- `WeightedSum` calculator (for excess returns)
- `ReturnCompounderDefinition` (for index level)

**DAG Structure** (All Generic):
```
Base Index Level ──> Base Index Return ──┐
                                         ├──> Base Excess Return (WeightedSum [1, -1]) ──┐
LIBOR Rate Asset ──> LIBOR Return ───────┘                                                │
                                                                                          ├──> Scaled Excess Return (Exposure × Base Excess Return) ──┐
Exposure ───────────────────────────────────────────────────────────────────────────────┘                                                          │
                                                                                                                                                  ├──> Net Return (WeightedSum [1, -1]) ──> Index Level (ReturnCompounder)
Fee Asset ──> Fee Return ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

**Note**: No index-specific calculator needed - all calculations use generic components. The index level calculation uses return space decomposition (excess returns, then compounding) because it makes the calculation decomposable.

---

## Part 3: Implementation Roadmap

### Phase 1: Core Generic Components
1. Index composition data structures
2. RecursiveExecutor
3. WeightedSum calculator and definitions
4. Cash asset and return calculations
5. Return compounder

### Phase 2: SOLSTAE Index
1. ETF total return
2. Rolling futures level
3. Transaction/replication costs
4. Base index composition
5. Index level calculation

### Phase 3: Options Infrastructure
1. Option data structures
2. Option pricing calculator
3. Implied volatility calculator
4. Forward and discount factor calculators
5. TWAP calculator
6. Portfolio mark-to-market

### Phase 4: Variance Replication Index
1. Variance strike calculator (index-specific)
2. Variance replication portfolio builder (index-specific)
3. Option portfolio management
4. Delta hedging (as regular asset)
5. Integration with generic options infrastructure

### Phase 4b: AIPEX5 Index
1. Realized volatility calculator (`root_mean_square`)
2. Max volatility combiner
3. Volatility targeting exposure calculator
4. Generic multiply combiner (for exposure scaling)
5. Wire DAG using all generic components

### Phase 5: Persistence and StartingState
1. StartingState node implementation
2. Checkpoint persistence
3. Hot start capability

---

## Part 4: Technical Specifications

### 4.1 Component Catalog

#### Calculators (Pure Functions)
- `weighted_sum()` - Weighted aggregation
- `compound_return()` - Compound returns into levels
- `population_std_dev()` - Population standard deviation
- `root_mean_square()` - Root mean square (realized volatility)
- `max_value()` - Maximum value combiner
- `min_value()` - Minimum value combiner
- `option_pricing()` - Option pricing (premium + Greeks)
- `implied_volatility()` - Implied volatility (pluggable optimizer)
- `forward_and_discount_factor()` - Forward and discount factor
- `twap()` - Time-weighted average price
- `portfolio_mark_to_market()` - Portfolio valuation
- `volatility_targeting_exposure()` - Volatility targeting

#### Containers (Stateless Wrappers)
- **ReturnAnalytic** trait: Common interface for return calculations
  - `LogReturnAnalytic`: Wraps `log_return_value()` calculator
  - `ArithReturnAnalytic`: Wraps arithmetic return calculator
- **VolatilityAnalytic** trait: Common interface for volatility calculations
  - `StdDevVolatilityAnalytic`: Wraps `population_std_dev()` calculator
  - `RealizedVolatilityAnalytic`: Wraps `root_mean_square()` calculator

#### Executors
- **WindowedAnalyticExecutor**: Rolling window calculations
- **RecursiveExecutor**: Stateful, iterative calculations
- **MergeExecutor**: Combines multiple parent nodes
- **DataProviderExecutor**: Pulls data from sources

#### Definitions (Generic)
- `ExcessReturnDefinition` - Excess return calculation
- `WeightedSumMergeDefinition` - Weighted sum merge
- `CashAssetDefinition` - Cash asset compounding
- `ReturnCompounderDefinition` - Compound returns into levels
- `TotalReturnLevelDefinition` - Total return level
- `ExcessReturnLevelDefinition` - Excess return level
- `RollingFuturesLevelDefinition` - Rolling futures
- `VolatilityDefinition` - Volatility calculation
- `RealizedVolatilityDefinition` - Realized volatility
- `MaxVolatilityDefinition` - Max volatility combiner
- `VolatilityTargetingExposureDefinition` - Volatility targeting exposure
- `OptionPricingDefinition` - Option pricing
- `OptionDeltaDefinition` - Option delta calculation
- `PortfolioMarkToMarketDefinition` - Portfolio valuation
- `StartingStateDefinition` - Initial state provider

#### Definitions (Index-Specific)
- **Variance Replication**:
  - `VarianceStrikeCalculator` - Variance strike calculation
  - `VarianceReplicationPortfolioBuilder` - Portfolio builder
  
- **AIPEX5**:
  - `RealizedVolatility21DayDefinition` - 21-day realized volatility
  - `RealizedVolatility63DayDefinition` - 63-day realized volatility

### 4.2 Data Structures

- `IndexComposition` - Index composition output
- `OptionDefinition` - Option asset definition
- `OptionPosition` - Option position (option + units)
- `OptionPricing` - Option pricing result (premium + Greeks)
- `ContinuingOptionPortfolio` - Option portfolio tracking

---

## Part 5: Key Design Decisions

### 5.1 Generic vs Index-Specific

**Generic Components** (Main Registry):
- All mathematical functions
- All return calculations
- All volatility calculations
- All asset management (cash, fees, etc.)
- All options infrastructure (pricing, Greeks, etc.)
- All futures infrastructure

**Index-Specific Components** (Factory-Generated):
- Variance strike calculator (variance-specific formula)
- Variance replication portfolio builder (variance-specific logic)
- DAG structure (how components are wired together)

### 5.2 Decomposition via Transformation

**Principle**: Sometimes transforming to returns makes calculations decomposable
- **When to use returns**: Excess return calculations (AIPEX5, SOLSTAE) benefit from return space decomposition
  - Excess return: `WeightedSum [1, -1]` of asset return and cash return
  - Then compound: `ReturnCompounder` to get levels
- **When to use levels/quantities**: Some calculations work better in level/quantity space
  - Variance index: Works directly with quantities and levels
  - Portfolio mark-to-market: Works with position quantities and prices
- **Key Insight**: The transformation (returns vs levels) is chosen based on what makes the calculation decomposable, not a universal rule

### 5.3 Asset-First Approach

**Principle**: Treat everything as an asset
- Cash is an asset (compounds interest)
- Fees are assets (negative returns)
- Delta hedge positions are assets
- Enables consistent calculation patterns

### 5.4 Persistence Strategy

**Single Table**: All indices use `analytics` table
- No migrations needed for new indices
- Scalable to unlimited indices
- JSON storage for flexible composition data

---

## Part 6: Registry and Configuration

### 6.1 Registry Architecture

**Two-Tier Registry System**:

1. **Main Registry** (`AnalyticRegistry`):
   - Generic, reusable components
   - Available to all indices
   - Registered at application startup

2. **Index Registry** (`IndexNodeRegistry`):
   - Factory-generated from rulebook configuration
   - Index-specific nodes and DAG structure
   - Created per index instance

**Registration Flow**:
```
Application Start ──> Main Registry (Generic Components)
                              │
                              └──> Index Registry Factory ──> Index-Specific Nodes
                                                              (from Rulebook)
```

### 6.2 Rulebook Configuration

**Format**: YAML configuration (converted from PDF guidelines)

**Structure**:
- Index metadata (name, currency, start date, initial level)
- Component definitions (ETFs, futures, options)
- Calculation parameters (costs, rates, schedules)
- DAG structure (how components are wired)

**Example** (SOLSTAE):
```yaml
index:
  name: "Solactive Systematic Trend Alpha Replicator Excess Return Index"
  currency: "USD"
  start_date: "2006-07-13"
  initial_level: 100.0

components:
  - id: "EEM.P"
    type: "ETF"
  - id: "0#ES:"
    type: "EquityFutures"
    roll_schedule: {...}

calculation:
  transaction_cost_rate: 0.0002
  replication_cost_rates:
    futures: 0.0015
```

---

## Part 7: Data Requirements

### 7.1 Data Sources

**Required Data Types**:
- **ETF Data**: Prices, dividends, corporate actions
- **Futures Data**: Settlement prices, expiration dates, roll schedules
- **Options Data**: Prices (bid/ask/mid), strikes, expirations, TWAP
- **FX Rates**: WM/Refinitiv rates (04:00 p.m. London time)
- **Reference Rates**: SOFR, LIBOR, overnight rates
- **Index Levels**: Underlying index levels (for variance replication)

### 7.2 Data Provider Interface

**Extended `DataProvider` Trait**:
- Support for all asset types (ETF, futures, options)
- Support for reference rates
- Support for target weights (for SOLSTAE)
- Support for underlying index levels

---

## Appendix: Implementation Details

Detailed implementation specifications are provided in separate appendix documents:

- **A.1 Component Catalog (Detailed)**: See `appendix-a1-component-catalog.md`
  - Complete calculator function signatures
  - Container trait implementations
  - Executor patterns and usage
  - Definition wiring examples

- **A.2 DAG Structures (Detailed)**: See `appendix-a2-dag-structures.md`
  - Complete DAG diagrams for all three indices
  - Component-by-component breakdown
  - Common patterns and execution flows
  - State management details

- **A.3 Data Structures (Detailed)**: See `appendix-a3-data-structures.md`
  - Complete struct definitions with code examples
  - Conversion function implementations
  - Persistence storage formats
  - Data provider extensions

