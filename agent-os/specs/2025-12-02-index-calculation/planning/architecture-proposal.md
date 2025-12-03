# Architecture Proposal: Index Calculation Framework

## Overview

This document proposes the architecture for implementing the Solactive Systematic Trend Alpha Replicator Excess Return Index calculation using the existing analytics framework. The proposal addresses node design, registry strategy, persistence, and hot start capabilities.

## Node Architecture

### General-Purpose Nodes (Add to Main Registry)

These nodes are reusable across different index calculations and should be added to the main `AnalyticRegistry`:

#### 0. Index Composition Types (Generalized for Nested Indices)
**Purpose**: Define data structures for index composition output, supporting nested indices

**Location**: `src/analytics/index_composition.rs` (new module)

```rust
/// Index composition output structure
pub struct IndexComposition {
    pub level: f64,
    pub representation: CompositionRepresentation,
    pub timestamp: DateTime<Utc>,
}

/// Composition representation (weights or quantities)
pub enum CompositionRepresentation {
    Weights {
        components: Vec<IndexComponent>,
    },
    Quantities {
        components: Vec<IndexComponent>,
        divisor: f64,
    },
}

/// Index component (can be asset or nested index)
pub enum IndexComponent {
    /// Direct asset component (e.g., ETF, futures contract)
    Asset {
        component_id: String,
        exposure: f64,   // weight or quantity depending on representation
        price: f64,
    },
    /// Nested index component
    Index {
        component_id: String,
        exposure: f64,   // weight or quantity depending on representation
        composition: Box<IndexComposition>,  // Recursive - supports nested indices
    },
}
```

**Key Features**:
- **Nested Support**: Components can be assets or other indices (recursive structure)
- **Risk Representation**: Quantities accurately represent position risk (e.g., offsetting cash in FX-hedged)
- **Flattening**: `flatten()` method collapses nested indices to flat asset-based composition

**Conversion Functions** (Non-Recursive):
- `weights_to_quantities(composition: &IndexComposition) -> IndexComposition`
  - Formula: `q_i = (w_i × level) / P_i`, `Divisor = 1`
  - **Converts current level only** - preserves nested index structure (nested indices keep their original representation)
- `quantities_to_weights(composition: &IndexComposition) -> IndexComposition`
  - Formula: `level = (Σ q_i × P_i) / divisor`, `w_i = (q_i × P_i) / level`
  - **Converts current level only** - preserves nested index structure (nested indices keep their original representation)

**Recursive Conversion** (if needed):
- Apply conversion node recursively to nested indices at a higher level
- Or create a separate "RecursiveConversion" node that applies conversion to all levels

**Flatten Function**:
- `flatten(&self) -> IndexComposition`: Recursively resolves nested indices to underlying assets
- Multiplies weights/quantities through the hierarchy
- Returns flat composition with only asset components
- Preserves risk representation (quantities accurately model position risk)

#### 0. RecursiveExecutor (General-Purpose Executor)
**Purpose**: Maintain previous state for recursive/iterative calculations

**Location**: `src/analytics/registry.rs` (new executor, reusable across multiple analytics)

**Key Characteristics**:
- **Maintains State**: Stores previous calculation result (e.g., `Option<f64>` for levels, `Option<IndexComposition>` for indices)
- **Reusable**: Can be used for any calculation that depends on previous value
- **Generic**: Works with any state type that can be serialized/stored
- **Push-Mode**: Designed for incremental computation (point-by-point updates)

**Use Cases**:
1. **Exponential Moving Average (EMA)**: Maintains previous EMA value
   - Calculator: `ema_step(previous: Option<f64>, value: f64, lambda: f64) -> f64`
   - State: `Option<f64>` (previous EMA value)
   - Formula: `EMA_t = lambda × value_t + (1 - lambda) × EMA_{t-1}`
   
2. **Index Level Calculation**: Maintains previous index level and composition
   - Calculator: Index level update function
   - State: `Option<IndexComposition>` (previous level + composition)
   - Formula: `Level_t = Level_{t-1} × (1 + return_t)`
   
3. **Cash Asset**: Maintains previous cash level
   - Calculator: Compounding function
   - State: `Option<f64>` (previous cash level)
   - Formula: `Cash_t = Cash_{t-1} × (1 + rate × DCF/365)`
   
4. **Rolling Futures Level**: Maintains previous futures level
   - Calculator: Weighted return calculation
   - State: `Option<f64>` (previous futures level)
   - Formula: `RFL_t = RFL_{t-1} × (1 + FuturesReturn_t)`

**Implementation Pattern**:
```rust
struct RecursiveExecutor {
    compute_fn: Arc<dyn Fn(&Node, Option<&dyn Any>, DateTime<Utc>, f64) -> Result<(f64, Box<dyn Any>), String> + Send + Sync>,
}

impl AnalyticExecutor for RecursiveExecutor {
    fn execute_push(&self, node: &Node, parent_outputs: &[ParentOutput], timestamp: DateTime<Utc>) -> Result<TimeSeriesPoint, String> {
        // 1. Get current value from parent outputs
        let current_value = extract_value(parent_outputs)?;
        
        // 2. Load previous state from node state storage
        let previous_state = node.get_state();
        
        // 3. Call compute function with previous state and current value
        let (result, new_state) = (self.compute_fn)(node, previous_state, timestamp, current_value)?;
        
        // 4. Store new state for next iteration
        node.set_state(new_state);
        
        // 5. Return result
        Ok(TimeSeriesPoint { timestamp, value: result })
    }
}
```

**State Management**:
- State is stored per-node in the DAG (node-specific state storage)
- State persists across push-mode iterations
- For pull-mode: State is reset and rebuilt by replaying historical data
- State can be serialized for persistence (hot start capability)

**Integration with Existing Analytics**:
- **EMA**: Currently uses `ExponentialWindow` - can be refactored to use `RecursiveExecutor` for consistency
- **Index Calculations**: New use case - maintains index level and composition
- **Cash Assets**: New use case - maintains cash level

#### 1. WeightedSum Merge Node (Uses Existing MergeExecutor)
**Purpose**: Calculate weighted sum of component returns

**Approach**: Compose existing nodes - no new calculator needed
- Use existing `ArithReturnAnalytic` to calculate returns for each component
- Use `MergeExecutor` to combine multiple return inputs with weights
- Reuse existing arithmetic returns infrastructure

**DAG Structure**:
```
Component1 Level ──> ArithReturn ──┐
Component2 Level ──> ArithReturn ──┤
Component3 Level ──> ArithReturn ──┼──> WeightedSumMerge (with weights)
...                                 │
ComponentN Level ──> ArithReturn ──┘
```

**Implementation**: 
- New Definition: `WeightedSumDefinition` 
- Uses `MergeExecutor` with N return nodes as parents
- Merge function closure: `Σ w_i × return_i` where weights come from target weights data
- Location: `src/analytics/registry.rs` (new definition, uses existing MergeExecutor)

#### 2. Cash Asset Node (Interest Accrual)
**Purpose**: Model cash/funding as an asset that accrues interest

**Approach**: Create a cash asset that compounds interest over time
- Cash asset level: `Cash_t = Cash_{t-1} × (1 + rate × DCF/365)`
- Can be used for any funding/financing calculation
- Reusable across different indices and strategies

**Implementation**:
- New Definition: `CashAssetDefinition`
- Uses `RecursiveExecutor` (general-purpose) to maintain cash level
- Calculator: Pure compounding function `(1 + rate × DCF/365)`
- State: `Option<f64>` (previous cash level)

#### 3. Cash Asset Returns (Uses Existing Arithmetic Return)
**Purpose**: Calculate returns on cash asset

**Approach**: Use existing `ArithReturnAnalytic`
- Cash asset level compounds: `Cash_t = Cash_{t-1} × (1 + rate × DCF/365)`
- Cash return = `(Cash_t / Cash_{t-1} - 1)` = `rate × DCF/365`
- This is just an arithmetic return calculation - no new calculator needed!

**Excess Return Pattern** (using WeightedSum with negative weights):
```
Asset Total Return ──┐
                     ├──> WeightedSum([1, -1]) ──> Excess Return
Cash Asset Return ──┘
```

**Note**: WeightedSum can handle differences using negative weights: `1 × asset_return + (-1) × cash_return = asset_return - cash_return`
- No separate Difference calculator needed - WeightedSum is sufficient!

#### 5. Weighted Sum Calculator (Pure Math)
**Purpose**: Calculate weighted sum of values

**Approach**: Domain-agnostic pure function
- Input: Values and weights
- Output: Weighted sum
- Formula: `Σ w_i × value_i`

**Implementation**:
- New Calculator: `weighted_sum(values: &[f64], weights: &[f64]) -> f64`
- Can be used for any weighted aggregation
- Location: `src/analytics/calculators.rs` (pure math function)

#### 6. Transaction Cost Return Node
**Purpose**: Calculate transaction costs as a return stream

**Approach**: Calculate transaction cost as a return (not a level)
- Transaction cost return: Based on weight changes
- Returns negative return to be subtracted from base return
- Uses WeightedSum to subtract from base return

**Formula**:
- First day: `TTC_return = -Σ ftc × ABS(w_i,t)`
- Subsequent days: `TTC_return = -Σ ftc × ABS(w_i,t - w_i,t-1)`
- Where `ftc = 0.02% = 0.0002`

**Implementation**:
- **Calculator**: `transaction_cost_return()` in `src/analytics/calculators.rs`
- **Container**: `TransactionCostAnalytic` in `src/analytics/containers.rs`
- **Executor**: `RecursiveExecutor` to maintain previous weights state
- **Definition**: `TransactionCostDefinition` in `src/analytics/registry.rs`
- **Data Dependency**: Target weights from DataProvider (database query)

**State Management**:
- Previous weights: `Vec<f64>` stored in executor state
- First day flag: Determined by absence of previous weights

**See**: `cost-implementation-detail.md` for full implementation details

#### 7. Replication Cost Return Node
**Purpose**: Calculate replication costs as a return stream

**Approach**: Calculate replication cost as a return (not a level)
- Replication cost return: Based on component weights, types, and day count
- Returns negative return to be subtracted from base return
- Uses WeightedSum to subtract from base return

**Formula**:
- `TRC_return = -Σ RC × ABS(w_i,t) × DCF/365`
- Where `RC = 0.15%` for futures, `0.0%` for ETFs
- `DCF = calendar days / 365`

**Implementation**:
- **Calculator**: `replication_cost_return()` in `src/analytics/calculators.rs`
- **Day Count Calculator**: `day_count_fraction()` in `src/analytics/calculators.rs`
- **Container**: `ReplicationCostAnalytic` in `src/analytics/containers.rs`
- **Executor**: `RecursiveExecutor` to maintain previous date for DCF calculation
- **Definition**: `ReplicationCostDefinition` in `src/analytics/registry.rs`
- **Data Dependency**: Target weights from DataProvider, component types from rulebook config

**State Management**:
- Previous date: `NaiveDate` stored in executor state
- Component types: `Vec<AssetType>` stored in definition (static, from rulebook)

**See**: `cost-implementation-detail.md` for full implementation details

#### 4. Day Count / Funding Asset Node
**Purpose**: Model day count effects as an asset

**Approach**: Treat day count/funding as an asset that compounds
- Any compounding effect (funding costs, day count adjustments) becomes an asset
- Can be differenced with other assets to get net effect
- More composable than embedding day count logic everywhere

**Implementation**:
- Similar to Cash Asset Node
- Creates a "funding asset" that compounds at a rate
- Can be differenced with other assets to get net return

**Example**: Transaction cost funding
- Create funding asset that compounds at transaction cost rate
- Difference with base return to get net return after costs

### Conversion Nodes (General-Purpose)

#### WeightsToQuantities Node
**Purpose**: Convert weights-based composition to quantities-based (preserves nested structure)

**Components**:
- **Calculator**: `weights_to_quantities()` function (non-recursive, converts current level only)
- **Container**: `WeightsToQuantitiesAnalytic`
- **Executor**: `PassthroughExecutor` (no state needed)
- **Definition**: `WeightsToQuantitiesDefinition` - registers in main registry

**Input**: `IndexComposition` with weights representation (can contain nested indices)
**Output**: `IndexComposition` with quantities representation (preserves nested structure, nested indices remain in their original representation)

**Formula** (current level only):
- For assets: `q_i = (w_i × Level) / P_i,t`
- For nested indices: `q_i = (w_i × Level) / nested_index_level` (nested index composition unchanged)
- `Divisor = 1` (when converting from weights)

**Key Point**: Conversion does NOT recurse into nested indices - it only converts the current level. Nested indices keep their original representation. To convert nested indices, apply conversion recursively at a higher level or use a separate recursive conversion node.

#### QuantitiesToWeights Node
**Purpose**: Convert quantities-based composition to weights-based (preserves nested structure)

**Components**:
- **Calculator**: `quantities_to_weights()` function (non-recursive, converts current level only)
- **Container**: `QuantitiesToWeightsAnalytic`
- **Executor**: `PassthroughExecutor` (no state needed)
- **Definition**: `QuantitiesToWeightsDefinition` - registers in main registry

**Input**: `IndexComposition` with quantities representation (can contain nested indices)
**Output**: `IndexComposition` with weights representation (preserves nested structure, nested indices remain in their original representation)

**Formula** (current level only):
- For assets: `Level = (Σ q_i × P_i,t) / Divisor`, `w_i = (q_i × P_i,t) / Level`
- For nested indices: `w_i = (q_i × nested_index_level) / Level` (nested index composition unchanged)

**Key Point**: Conversion does NOT recurse into nested indices - it only converts the current level. Nested indices keep their original representation. To convert nested indices, apply conversion recursively at a higher level or use a separate recursive conversion node.

#### Flatten Node
**Purpose**: Flatten nested index composition to flat asset-based composition

**Components**:
- **Calculator**: `flatten()` method on `IndexComposition`
- **Container**: `FlattenAnalytic`
- **Executor**: `PassthroughExecutor` (no state needed)
- **Definition**: `FlattenDefinition` - registers in main registry

**Input**: `IndexComposition` (can be nested)
**Output**: `IndexComposition` (flat, only asset components)

**Algorithm**:
- Recursively resolves all nested indices to underlying assets
- Multiplies weights/quantities through the hierarchy
- Aggregates exposures for assets that appear in multiple nested indices
- Returns flat composition with only `IndexComponent::Asset` entries

**Use Cases**:
- Risk analysis: See all underlying asset exposures
- Portfolio construction: Build portfolios from flattened composition
- Reporting: Present flat view of nested index structure

### Index-Specific Nodes (Factory-Generated)

These nodes are specific to the SOLSTAE index and should be generated from rulebook configuration:

#### 1. RollingFuturesLevel Node (General-Purpose, Not SOLSTAE-Specific)
**Purpose**: Calculate rolling futures level with contract transitions

**Note**: This should be implemented as a general-purpose feature, not SOLSTAE-specific.

**Configuration Required**:
- Roll schedule (Table 4: Active Contract, Table 5: Next Active Contract)
- Roll anchor type (First Notice vs Expiry)
- Roll offset and roll days
- Futures currency
- FX conversion requirements

**Components**:
- **Calculator**: Pure function for weighted return calculation (uses WeightedSum)
- **Container**: `RollingFuturesLevelAnalytic` - stores rolling level state
- **Executor**: `RecursiveExecutor` - maintains previous level for incremental calculation
- **Definition**: `RollingFuturesLevelDefinition` - registers in main registry (general-purpose)

**Formula**: 
```
ActiveReturn_t = (Px_active_t / Px_active_{t-1} - 1)
NextReturn_t = (Px_next_t / Px_next_{t-1} - 1)
FuturesReturn_t = WeightedSum([ActiveReturn_t, NextReturn_t], [w_active, w_next]) × FXConversion_t
RFL_t = RFL_{t-1} × (1 + FuturesReturn_t)
```

**Composition**: Uses WeightedSum calculator (pure math) + FX conversion

#### 2. ETF Total Return Node
**Purpose**: Calculate ETF total return with dividend reinvestment

**Components**:
- **Calculator**: Pure function for total return with dividends
- **Container**: `ETFTotalReturnAnalytic` - stores ETF level state
- **Executor**: `RecursiveExecutor` - maintains previous level
- **Definition**: `ETFTotalReturnDefinition` - registers in main registry (general-purpose)

**Formula**:
```
ETFTotalReturn_t = (ETF_Close_t + div_t) / ETF_Close_{t-1}
ETFLevel_t = ETFLevel_{t-1} × ETFTotalReturn_t
```

#### 2b. ETF Excess Return (Composed from Total Return - Cash)
**Purpose**: Calculate ETF excess return by differencing total return and cash

**Composition**:
```
ETF Total Return ──┐
                   ├──> Difference ──> ETF Excess Return
Cash Asset (SOFR/LIBOR) ──┘
```

**Configuration Required**:
- Rate switch date (2020-12-31)
- SOFR RIC (USDSOFR=)
- LIBOR RIC (USD3MFSR=)
- LIBOR offset (-0.26161%)

**No new node needed**: Composed from ETF Total Return + Cash Asset + Difference calculator

#### 3. BaseIndex Node
**Purpose**: Calculate base index from component levels and target weights, output composition

**Components**:
- **Calculator**: None needed - uses existing `ArithReturnAnalytic` + `WeightedSumMerge`
- **Container**: `BaseIndexAnalytic` - stores base index level and composition
- **Executor**: `RecursiveExecutor` - maintains previous base index level and composition
- **Definition**: `BaseIndexDefinition` - registers in index-specific registry

**DAG Structure**:
```
Component1 Level ──> ArithReturn ──┐
Component2 Level ──> ArithReturn ──┤
...                                 ├──> WeightedSumMerge ──> BaseIndex (recursive) ──> IndexComposition
ComponentN Level ──> ArithReturn ──┘
```

**Output**: `IndexComposition` struct containing:
- Level: `f64` (base index level)
- Composition: Either weights-based or quantities-based (config-driven)
  - **Weights**: `Vec<IndexComponent>` (can contain assets or nested indices)
  - **Quantities**: `Vec<IndexComponent>` (can contain assets or nested indices) + `Divisor: f64`

**Dependencies**:
- Component level nodes (ETFExcessReturn, RollingFuturesLevel) OR nested index nodes
- Target weights (from external source)
- Component prices (from component level nodes) OR nested index compositions
- Uses existing `ArithReturnAnalytic` for component returns

**Conversion Logic**:
- If config says "weights": Output weights + prices directly (as `IndexComponent::Asset` or `IndexComponent::Index`)
- If config says "quantities": Convert weights → quantities using `q_i = (w_i × Level) / P_i,t`, `Divisor = 1`
- Supports nested indices: If component is an index, store as `IndexComponent::Index` with nested composition

**Flattening**:
- `flatten()` method available to collapse nested indices to flat asset-based composition
- Recursively multiplies weights/quantities through hierarchy
- Returns composition with only `IndexComponent::Asset` entries

#### 4. IndexLevel Node (Composed from Multiple Assets)
**Purpose**: Calculate final index level by differencing base return with costs, output composition

**Composition**:
```
BaseIndex Return ──┐
ARF Asset ─────────┤
TTC Asset ─────────├──> WeightedSum (with signs) ──> Net Return ──> Index Level (recursive) ──> IndexComposition
TRC Asset ─────────┘
```

**Components**:
- **Calculator**: Uses `WeightedSum` calculator with signed weights
- **Container**: `IndexLevelAnalytic` - stores index level and composition
- **Executor**: `RecursiveExecutor` - maintains previous index level and composition
- **Definition**: `IndexLevelDefinition` - registers in index-specific registry

**Output**: `IndexComposition` struct containing:
- Level: `f64` (final index level)
- Composition: Inherited from BaseIndex (weights or quantities, based on config)
  - Preserves nested structure (if BaseIndex contains nested indices)
  - Updates level but maintains component structure
  - If weights: Update weights vector (if changed) + prices
  - If quantities: Update quantities vector (if changed) + prices + divisor

**Dependencies**:
- BaseIndex composition node (provides base level + composition, can contain nested indices)
- ARF Asset node (compounds at 0.4% p.a.)
- TransactionCost Asset node
- ReplicationCost Asset node
- Uses WeightedSum with weights: [+1, -1, -1, -1] to subtract costs

**Formula** (conceptual):
```
NetReturn_t = BaseReturn_t - ARF_t - TTC_t - TRC_t
Index_t = max[0, Index_{t-1} × (1 + NetReturn_t)]
```

**Composition Update**:
- **Weights**: Inherit from BaseIndex (target weights, preserves nested structure)
- **Quantities**: Recalculate from weights using `q_i = (w_i × Index_t) / P_i,t`, `Divisor = 1` (preserves nested structure)

**Flattening**:
- `flatten()` method available to collapse to flat asset-based composition
- Useful for risk analysis and portfolio construction
- Quantities representation accurately models risk (e.g., offsetting cash in FX-hedged)

**Benefits**: All costs modeled as assets, composable through WeightedSum, composition output configurable, supports nested indices

#### 5. FX-Hedged Index Node (General-Purpose)
**Purpose**: Calculate FX-hedged index level using explicit hedge model

**Model** (models risk correctly):
- **Long Index Asset** (base currency, e.g., USD): Value = `Index_t`
- **Short Cash Asset** (hedge currency, e.g., GBP, no interest): Value = `Index_{t-1} × FX_{t-1}` (equal to previous day's index value in hedge currency)
- **FX Conversion**: Convert base currency index to hedge currency using current FX rate
- **Net Return**: Difference between converted index value and cash asset value

**DAG Structure**:
```
USD Index Level ──┐
                  ├──> Convert to GBP (× FX_t) ──┐
GBP Cash Level ───┘ (Index_{t-1} × FX_{t-1}, no interest) ├──> Difference ──> Net GBP Return ──> Hedged Index Level
```

**Components**:
- **Index Asset Node**: Provides base currency index level (uses existing IndexLevel node)
- **Cash Asset Node**: Fixed at `Index_{t-1} × FX_{t-1}` (no interest accrual)
- **FX Conversion Calculator**: Multiply index by current FX rate
- **Difference Calculator**: Net return = (Index in hedge currency) - (Cash asset)
- **Container**: `FXHedgedIndexAnalytic` - stores hedged index level
- **Executor**: `RecursiveExecutor` - maintains previous hedged index level and cash asset value
- **Definition**: `FXHedgedIndexDefinition` - registers in main registry (general-purpose)

**Why This Model**:
- ✅ Models the risk correctly (explicit hedge structure)
- ✅ Clear representation of the hedging mechanism
- ✅ Makes the cash position explicit (no interest accrual)
- ✅ Intuitive: index + cash hedge

**Formula**:
```
Net_GBP_Return_t = (Index_t × FX_t - Index_{t-1} × FX_{t-1}) / (Index_{t-1} × FX_{t-1})
                 = (Index_t/Index_{t-1}) × (FX_t/FX_{t-1}) - 1

Index_t^FX = Index_{t-1}^FX × (1 + Net_GBP_Return_t)
```

## Registry Strategy

### Hybrid Approach: General + Index-Specific

#### Option 1: Separate Index Registry (Recommended)
Create a separate `IndexNodeRegistry` that can be initialized from rulebook configuration:

```rust
pub struct IndexNodeRegistry {
    definitions: HashMap<String, Box<dyn AnalyticDefinition>>,
    rulebook_config: IndexRulebookConfig,
}

impl IndexNodeRegistry {
    pub fn from_rulebook(config: IndexRulebookConfig) -> Self {
        // Generate nodes from rulebook
        // Register index-specific nodes
    }
}
```

**Advantages**:
- Clear separation between general-purpose and index-specific nodes
- Can support multiple indices with different rulebooks
- Easy to add/remove indices without affecting main registry

#### Option 2: Factory Pattern in Main Registry
Extend `AnalyticRegistry` to support factory-based node creation:

```rust
impl AnalyticRegistry {
    pub fn register_index_nodes(&mut self, rulebook: IndexRulebookConfig) {
        // Generate and register index-specific nodes
    }
}
```

**Advantages**:
- Single registry for all nodes
- Simpler dependency resolution

**Recommendation**: Use Option 1 (Separate Index Registry) for better separation of concerns and multi-index support.

## Rulebook Configuration Format

Propose YAML format for rulebook configuration (converted from PDF):

```yaml
index:
  name: "Solactive Systematic Trend Alpha Replicator Excess Return Index"
  currency: "USD"
  start_date: "2006-07-13"
  initial_level: 100.0
  adjusted_return_factor: 0.004  # 0.4% p.a.
  composition_output: "weights"  # or "quantities" - determines output format

components:
  - id: "EEM.P"
    type: "ETF"
    ric: "EEM.P"
    exchange: "NYSE"
    
  - id: "0#ES:"
    type: "EquityFutures"
    ric: "0#ES:"
    exchange: "CME"
    futures_currency: "USD"
    roll_anchor: "Expiry"
    roll_offset: -6
    roll_days: 5
    active_contract_schedule:
      Jan: Mar
      Feb: Mar
      # ... etc
    next_active_contract_schedule:
      Jan: Mar
      # ... etc

calculation:
  transaction_cost_rate: 0.0002  # 0.02%
  replication_cost_rates:
    futures: 0.0015  # 0.15%
    etf: 0.0
  
  rate_switch_date: "2020-12-31"
  sofr_ric: "USDSOFR="
  libor_ric: "USD3MFSR="
  libor_offset: -0.0026161
```

## Persistence Strategy

### Database Schema Extension

Extend existing `analytics` table or create index-specific tables:

#### Option 1: Extend Analytics Table (Recommended)
Use existing `analytics` table with index-specific `analytics_name`:

```sql
-- Index level storage
INSERT INTO analytics (asset_key, date, analytics_name, value)
VALUES ('INDEX', '2024-01-01', 'solstae_index_level', '{"level": 100.5, "base_index": 100.3}');

-- Composition storage (as JSON)
INSERT INTO analytics (asset_key, date, analytics_name, value)
VALUES ('INDEX', '2024-01-01', 'solstae_composition', 
  '{"weights": {"EEM.P": 0.1, "0#ES:": 0.15, ...}, 
    "levels": {"EEM.P": 105.2, "0#ES:": 102.1, ...}}');
```

**Advantages**:
- Reuses existing schema
- Composition stored as JSON (flexible)
- Simple query pattern

#### Option 2: Dedicated Index Tables
Create separate tables for index-specific data:

```sql
CREATE TABLE index_levels (
    index_name TEXT NOT NULL,
    date TEXT NOT NULL,
    index_level REAL NOT NULL,
    base_index_level REAL NOT NULL,
    PRIMARY KEY (index_name, date)
);

CREATE TABLE index_composition (
    index_name TEXT NOT NULL,
    date TEXT NOT NULL,
    composition_json TEXT NOT NULL,  -- JSON with weights and levels
    PRIMARY KEY (index_name, date)
);
```

**Advantages**:
- More structured
- Better query performance for index-specific queries
- Clearer separation

**Recommendation**: Use Option 1 for MVP (simpler), Option 2 for production (better performance).

## Hot Start Implementation

### Checkpoint Strategy

Store checkpoint data to enable hot start:

```rust
pub struct IndexCheckpoint {
    pub index_name: String,
    pub checkpoint_date: NaiveDate,
    pub index_level: f64,
    pub base_index_level: f64,
    pub component_levels: HashMap<String, f64>,  // component_id -> level
    pub composition: IndexComposition,  // weights and levels
}
```

### Query Logic

```rust
impl IndexCalculation {
    pub fn calculate_with_hot_start(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<IndexLevel>, Error> {
        // 1. Find latest checkpoint <= start_date
        let checkpoint = self.find_checkpoint(start_date)?;
        
        // 2. Use checkpoint as initial state
        let mut state = IndexState::from_checkpoint(checkpoint);
        
        // 3. Calculate forward from checkpoint
        for date in dates_between(start_date, end_date) {
            let level = self.calculate_next_day(&mut state, date)?;
            self.store_checkpoint(&state, date)?;
        }
        
        Ok(levels)
    }
}
```

### Storage Pattern

Store checkpoints periodically (e.g., daily):

```sql
-- Store checkpoint
INSERT INTO analytics (asset_key, date, analytics_name, value)
VALUES ('INDEX', '2024-01-01', 'solstae_checkpoint', 
  '{"index_level": 100.5, "base_index": 100.3, 
    "component_levels": {...}, "composition": {...}}');
```

## Push vs Pull Mode

### Pull Mode (Historical Calculation)

**Use Case**: Calculate index for a date range on-demand

**Flow**:
1. Query database for checkpoint before start date
2. Load target weights for date range
3. Calculate index levels forward from checkpoint
4. Return time series

**Implementation**:
- Extend existing pull-mode DAG execution
- Add hot start checkpoint loading
- Calculate missing dates incrementally

### Push Mode (Real-time Updates)

**Use Case**: Receive new target weights daily, calculate incrementally

**Flow**:
1. Receive new target weights for day t
2. Load previous day's state (index level, component levels)
3. Calculate new index level
4. Store checkpoint
5. Stream update via SSE

**Implementation**:
- Use existing push-mode DAG execution
- Add checkpoint persistence after each update
- Support SSE streaming for real-time updates

## Data Provider Extensions

### Required Data Sources

1. **ETF Data**:
   - Closing prices
   - Dividends (ex-date, amount)
   - Source: Yahoo Finance or dummy data

2. **Futures Data**:
   - Settlement prices for active and next active contracts
   - Expiration dates
   - First notice dates
   - Source: Yahoo Finance (if available) or dummy data

3. **FX Rates**:
   - WM/Refinitiv rates (04:00 p.m. London time)
   - For non-USD futures
   - Source: External API or dummy data

4. **Reference Rates**:
   - SOFR (RIC: USDSOFR=)
   - 3-Month USD LIBOR (RIC: USD3MFSR=)
   - Source: External API or dummy data

5. **Target Weights**:
   - Daily weights per component (randomly generated for MVP)
   - Constraints:
     - Individual: -2.0 ≤ weight ≤ +2.0 (200% long/short)
     - Net: -1.0 ≤ Σ weights ≤ +1.0 (100% net exposure)
   - Format: `{date: {component_id: weight}}`
   - Source: Random weight generator with constraint validation

### Data Provider Interface

Extend `DataProvider` trait to support:
- ETF data with dividends
- Futures contract data (multiple contracts per chain)
- FX rates
- Reference rates
- Target weights

## Implementation Phases

### Phase 1: Foundation (MVP)
1. Create general-purpose calculators (WeightedSum, ExcessReturn, etc.)
2. Implement basic index calculation (without futures rolling)
3. Store index level and composition as JSON
4. Support pull-mode calculation

### Phase 2: Futures Rolling
1. Implement RollingFuturesLevel node
2. Add futures contract data provider
3. Implement roll schedule logic
4. Add FX conversion

### Phase 3: Hot Start
1. Implement checkpoint storage
2. Add checkpoint loading logic
3. Support hot start in pull mode
4. Optimize checkpoint frequency

### Phase 4: Push Mode & Real-time
1. Support push-mode index calculation
2. Add SSE streaming for real-time updates
3. Implement incremental checkpoint updates

### Phase 5: Rulebook Factory
1. Create rulebook parser (YAML/JSON)
2. Implement node factory from rulebook
3. Support multiple indices
4. Dynamic node registration

## Target Weight Generation

**Random Generation Strategy**:
- Generate random weights for each component daily
- Ensure constraints:
  - Individual component: -2.0 ≤ weight ≤ +2.0
  - Net exposure: -1.0 ≤ Σ weights ≤ +1.0
- Implementation: Constraint-satisfying random weight generator

## Open Questions

2. **Futures Data Availability**: Can we get futures contract data from Yahoo Finance?
   - If not, how detailed should dummy data be?

3. **Rulebook Format**: Should we parse PDF directly or require YAML/JSON conversion?
   - YAML/JSON recommended for MVP

4. **Corporate Actions**: Scope for MVP?
   - Defer to later phase

5. **Market Disruption**: Scope for MVP?
   - Defer to later phase

6. **Multiple Indices**: Should MVP support multiple indices or single index?
   - Single index recommended for MVP

