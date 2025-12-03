# Specification: Index Calculation Framework with Rulebook Support

## Goal
Implement a framework for calculating multi-asset indices based on rulebook specifications, starting with the Solactive Systematic Trend Alpha Replicator Excess Return Index (SOLSTAE). The framework should support rulebook-driven node generation, hot start from database checkpoints, and both push and pull mode execution.

## User Stories
- As a quantitative analyst, I want to calculate an index based on a rulebook specification so that I can replicate index methodologies programmatically
- As a researcher, I want to query historical index levels and composition so that I can analyze index performance over time
- As a system developer, I want to generate index-specific nodes from rulebook configuration so that I can support multiple indices without hardcoding each one
- As a trader, I want to receive real-time index updates as new target weights arrive so that I can monitor index performance incrementally
- As a backtester, I want to calculate index levels for historical date ranges efficiently using hot start so that I don't recalculate from scratch each time

## Specific Requirements

### Index Overview: SOLSTAE
- **Name**: Solactive Systematic Trend Alpha Replicator Excess Return Index
- **Type**: Multi-asset excess return index
- **Currency**: USD
- **Start Date**: 2006-07-13
- **Initial Level**: 100.0
- **Components**: 13 static components
  - 4 ETFs: EEM.P, GLD.P, XLE.P, XME.P
  - 2 FX Futures: EUR/USD (0#URO:), JPY/USD (0#JY:)
  - 4 Equity Futures: E-mini S&P 500 (0#ES:), E-mini Nasdaq-100 (0#NQ:), Japan 225 (0#NIY:), Euro 50 (0#STXE:)
  - 3 Bond Futures: 10y Treasury (0#TY:), 2y Treasury (0#TU:), Bund (0#FGBL:)

### Calculation Formulas

#### 1. Index Level (Section 3.1)
```
Index_t = max[0, Index_{t-1} × (B_t/B_{t-1} - ARF × DCF/365 - TTC_t - TRC_t)]
```
Where:
- `ARF`: Adjusted Return Factor (0.4% p.a. = 0.004)
- `TTC_t`: Total Transaction Cost
- `TRC_t`: Total Replication Cost
- `DCF`: Day count fraction (calendar days / 365)

#### 2. Base Index (Section 3.2)
```
B_t = B_{t-1} × (1 + Σ w_i,t × (IC_i,t / IC_i,t-1 - 1))
```
Where:
- `w_i,t`: Target weight of component i for day t
- `IC_i,t`: Level of component i at time t

#### 3. ETF Excess Return Level (Section 3.3)
```
ETFLevel_t = ETFLevel_{t-1} × ((ETF_Close_t + div_t) / ETF_Close_{t-1} - (rate_{t-2} × DCF/365))
```
Where:
- `div_t`: Cash dividends with ex-date on day t
- `rate_t`: SOFR (after 2020-12-31) or LIBOR - 0.26161% (before)

#### 4. Rolling Futures Level (Appendix 1)
```
FuturesReturn_t = (w_active × (Px_active_t / Px_active_{t-1} - 1) + 
                   w_next × (Px_next_t / Px_next_{t-1} - 1)) × FXConversion_t
RFL_t = RFL_{t-1} × (1 + FuturesReturn_t)
```

#### 5. Transaction Cost (Section 3.4)
- First day: `TTC_t = Σ ftc × ABS(w_i,t)`
- Subsequent days: `TTC_t = Σ ftc × ABS(w_i,t - w_i,t-1)`
- `ftc`: Fixed transaction cost (0.02% = 0.0002)

#### 6. Replication Cost (Section 3.5)
```
TRC_t = Σ RC × ABS(w_i,t) × DCF/365
```
Where:
- `RC`: 0.15% (0.0015) for futures, 0.0% for ETFs

### Target Weight Generation

**Random Generation with Constraints**:
- Generate random weights for each component daily
- Constraints:
  - Individual component: -2.0 ≤ weight ≤ +2.0 (200% long/short limit)
  - Net exposure: -1.0 ≤ Σ weights ≤ +1.0 (100% net limit)
- Implementation: Constraint-satisfying random weight generator

### Weights vs Quantities Representation

**Weights Index**:
- Representation: Vector of weights, vector of prices, level
- Level calculation: `Level_t = Level_{t-1} × (1 + Σ w_i × (P_i,t / P_i,t-1 - 1))`
- On next day: Given new prices, compute new level directly

**Quantities Index**:
- Representation: Quantities, prices, divisor
- Level calculation: `Level_t = (Σ q_i × P_i,t) / Divisor`
- When converting from weights: `Divisor = 1`, `q_i = (w_i × Level) / P_i,t`

**Conversion**:
- Weights → Quantities: `q_i = (w_i × Level) / P_i,t`, `Divisor = 1`
- Quantities → Weights: `Level = (Σ q_i × P_i,t) / Divisor`, `w_i = (q_i × P_i,t) / Level`

**Output**: Final nodes output `IndexComposition` struct containing:
- Level: `f64`
- Composition: Either weights-based or quantities-based (config-driven)
  - **Weights**: `Vec<IndexComponent>` (can contain assets or nested indices)
  - **Quantities**: `Vec<IndexComponent>` (can contain assets or nested indices) + `Divisor: f64`
- **Flattening**: `flatten()` method collapses nested indices to flat asset-based composition
- **Risk Representation**: Quantities accurately represent position risk (e.g., offsetting cash in FX-hedged)

### Node Architecture

### General-Purpose Executor: RecursiveExecutor

**Purpose**: Maintain previous state for recursive/iterative calculations

**Location**: `src/analytics/registry.rs` (new executor, reusable across multiple analytics)

**Key Characteristics**:
- **Maintains State**: Stores previous calculation result (e.g., `Option<f64>` for levels, `Option<IndexComposition>` for indices)
- **Reusable**: Can be used for any calculation that depends on previous value
- **Generic**: Works with any state type that can be serialized/stored
- **Push-Mode**: Designed for incremental computation (point-by-point updates)

**Use Cases**:
1. **Exponential Moving Average (EMA)**: Maintains previous EMA value
   - Calculator: `ema_step(previous: Option<f64>, value: f64, lambda: f64) -> f64` (already exists in `calculators.rs`)
   - State: `Option<f64>` (previous EMA value)
   - Formula: `EMA_t = lambda × value_t + (1 - lambda) × EMA_{t-1}`
   - **Note**: EMA can be refactored to use `RecursiveExecutor` instead of `ExponentialWindow` for consistency with other recursive calculations
   
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

**State Management**:
- State is stored per-node in the DAG (node-specific state storage)
- State persists across push-mode iterations
- For pull-mode: State is reset and rebuilt by replaying historical data
- State can be serialized for persistence (hot start capability)

#### General-Purpose Nodes (Add to Main Registry)

1. **WeightedSum Calculator** (Pure Math, `src/analytics/calculators.rs`)
   - Domain-agnostic pure function
   - Formula: `Σ w_i × value_i`
   - Can be used for any weighted aggregation

2. **Cash Asset Returns** (Uses Existing Arithmetic Return)
   - Cash asset level compounds: `Cash_t = Cash_{t-1} × (1 + rate × DCF/365)`
   - Cash return = `(Cash_t / Cash_{t-1} - 1)` = arithmetic return
   - Uses existing `ArithReturnAnalytic` - no new calculator needed!
   - **Excess Return**: Use WeightedSum with weights [1, -1] to get `asset_return - cash_return`

3. **Cash Asset Node** (General-Purpose)
   - Models cash/funding as an asset that accrues interest
   - Formula: `Cash_t = Cash_{t-1} × (1 + rate × DCF/365)`
   - Reusable for any funding/financing calculation

4. **Day Count / Funding Asset Node** (General-Purpose)
   - Models day count effects as an asset
   - Any compounding effect becomes an asset that can be differenced
   - More composable than embedding day count logic everywhere

5. **Transaction Cost Asset Node** (General-Purpose)
   - Models transaction costs as a compounding asset
   - Compounds based on weight changes
   - Can be differenced with base return to get net return

6. **Replication Cost Asset Node** (General-Purpose)
   - Models replication costs as a compounding asset
   - Compounds based on component weights and asset types
   - Can be differenced with base return

#### Index-Specific Nodes (Factory-Generated)

1. **RollingFuturesLevel Node** (General-Purpose, Not SOLSTAE-Specific)
   - Calculate rolling futures level with contract transitions
   - Should be implemented as general-purpose feature
   - Uses WeightedSum calculator (pure math)
   - Components: Calculator, Container, Executor (RecursiveExecutor), Definition

2. **ETF Total Return Node** (General-Purpose)
   - Calculate ETF total return with dividend reinvestment
   - Components: Calculator, Container, Executor (RecursiveExecutor), Definition
   - Formula: `ETFLevel_t = ETFLevel_{t-1} × ((ETF_Close_t + div_t) / ETF_Close_{t-1})`

2b. **ETF Excess Return** (Composed, Not a Separate Node)
   - Composed from: ETF Total Return - Cash Asset (SOFR/LIBOR)
   - Uses WeightedSum with weights [1, -1] to difference returns
   - No new node needed - just composition

3. **BaseIndex Node**
   - Calculate base index from component returns and target weights
   - Uses WeightedSum calculator (pure math)
   - Components: Container, Executor (RecursiveExecutor), Definition
   - DAG: Component Returns → WeightedSum → BaseIndex Return → BaseIndex Level (recursive)
   - **Output**: `IndexComposition` struct (level + weights or quantities, based on config)

4. **IndexLevel Node** (Composed from Multiple Assets)
   - Calculate final index level by differencing base return with costs
   - All costs modeled as assets (ARF Asset, TTC Asset, TRC Asset)
   - Uses WeightedSum with signed weights to subtract costs
   - Components: Container, Executor (RecursiveExecutor), Definition
   - DAG: BaseIndex Return + ARF Asset + TTC Asset + TRC Asset → WeightedSum → Net Return → Index Level
   - **Output**: `IndexComposition` struct (level + composition inherited from BaseIndex, updated with new level)

5. **WeightsToQuantities Node** (General-Purpose)
   - Convert weights-based composition to quantities-based (preserves nested structure)
   - Formula: `q_i = (w_i × Level) / P_i,t`, `Divisor = 1`
   - **Converts current level only** - nested indices preserve their original representation
   - Components: Calculator, Container, Executor, Definition

6. **QuantitiesToWeights Node** (General-Purpose)
   - Convert quantities-based composition to weights-based (preserves nested structure)
   - Formula: `Level = (Σ q_i × P_i,t) / Divisor`, `w_i = (q_i × P_i,t) / Level`
   - **Converts current level only** - nested indices preserve their original representation
   - Components: Calculator, Container, Executor, Definition

7. **Flatten Node** (General-Purpose)
   - Flatten nested index composition to flat asset-based composition
   - Recursively resolves nested indices to underlying assets
   - Multiplies weights/quantities through hierarchy
   - Components: Calculator (flatten method), Container, Executor, Definition

### Registry Strategy

**Hybrid Approach**: Separate Index Registry

- **General-Purpose Nodes**: Add to main `AnalyticRegistry`
- **Index-Specific Nodes**: Create `IndexNodeRegistry` initialized from rulebook
- **Factory Pattern**: Generate nodes from rulebook configuration (YAML/JSON)

### Rulebook Configuration Format

**YAML Format** (converted from PDF):

```yaml
index:
  name: "Solactive Systematic Trend Alpha Replicator Excess Return Index"
  currency: "USD"
  start_date: "2006-07-13"
  initial_level: 100.0
  adjusted_return_factor: 0.004

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

calculation:
  transaction_cost_rate: 0.0002
  replication_cost_rates:
    futures: 0.0015
    etf: 0.0
  rate_switch_date: "2020-12-31"
  sofr_ric: "USDSOFR="
  libor_ric: "USD3MFSR="
  libor_offset: -0.0026161
```

### Node Output Generalization

**Current**: `NodeOutput` enum supports `Single`, `Collection`, `Scalar`, `None`

**Required**: Support for structured data (composition)

**Proposed**: Extend `NodeOutput` enum with `Composition(IndexComposition)` variant

**IndexComposition Structure**:
```rust
pub struct IndexComposition {
    pub level: f64,
    pub representation: CompositionRepresentation,
    pub timestamp: DateTime<Utc>,
}

pub enum CompositionRepresentation {
    Weights {
        components: Vec<IndexComponent>,
    },
    Quantities {
        components: Vec<IndexComponent>,
        divisor: f64,
    },
}

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

impl IndexComposition {
    /// Flatten nested index composition to flat asset-based composition
    /// Recursively resolves all nested indices to underlying assets
    pub fn flatten(&self) -> IndexComposition {
        // Implementation recursively multiplies weights/quantities through hierarchy
    }
}
```

### Persistence Strategy

**Extend Existing Analytics Table**:

```sql
-- Index level storage
INSERT INTO analytics (asset_key, date, analytics_name, value)
VALUES ('INDEX', '2024-01-01', 'solstae_index_level', 
  '{"level": 100.5, "base_index": 100.3}');

-- Composition storage (as JSON)
INSERT INTO analytics (asset_key, date, analytics_name, value)
VALUES ('INDEX', '2024-01-01', 'solstae_composition', 
  '{"weights": {"EEM.P": 0.1, "0#ES:": 0.15, ...}, 
    "levels": {"EEM.P": 105.2, "0#ES:": 102.1, ...}}');

-- Checkpoint storage (for hot start)
INSERT INTO analytics (asset_key, date, analytics_name, value)
VALUES ('INDEX', '2024-01-01', 'solstae_checkpoint', 
  '{"index_level": 100.5, "base_index": 100.3, 
    "component_levels": {...}, "composition": {...}}');
```

### Hot Start Implementation

**Checkpoint Strategy**:
- Store checkpoint data after each calculation day
- Checkpoint includes: index level, base index level, component levels, composition
- When requesting date range [start, end]:
  1. Find latest checkpoint ≤ start date
  2. Use checkpoint as initial state
  3. Calculate forward from checkpoint

**Query Logic**:
```rust
pub fn calculate_with_hot_start(
    &self,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<IndexLevel>, Error> {
    let checkpoint = self.find_checkpoint(start_date)?;
    let mut state = IndexState::from_checkpoint(checkpoint);
    // Calculate forward...
}
```

### Push vs Pull Mode

**Pull Mode (Historical)**:
- Query index level for date range
- Use hot start if available
- Calculate missing dates forward
- Return time series

**Push Mode (Real-time)**:
- Receive new target weights daily
- Calculate new index level incrementally
- Store checkpoint after each update
- Support SSE streaming for real-time updates

### Data Provider Extensions

**Required Data Sources**:
1. **ETF Data**: Closing prices, dividends (ex-date, amount)
2. **Futures Data**: Settlement prices for active/next active contracts, expiration dates, first notice dates
3. **FX Rates**: WM/Refinitiv rates (04:00 p.m. London time) for non-USD futures
4. **Reference Rates**: SOFR (USDSOFR=), 3-Month USD LIBOR (USD3MFSR=)
5. **Target Weights**: Randomly generated with constraints (for MVP)

**Data Source Strategy**:
- **Yahoo Finance**: Use for ETFs and available futures data
- **Dummy Data**: Generate for:
  - Futures contracts not available on Yahoo Finance
  - FX rates (if not available)
  - Reference rates (SOFR/LIBOR)
  - Target weights (random generation with constraints)

### Implementation Phases

#### Phase 1: Foundation (MVP)
1. Create WeightedSum calculator (pure math function) - can handle differences with negative weights
2. Create Cash Asset node (interest accrual) - general-purpose, uses existing ArithReturnAnalytic
3. Create ETF Total Return node - general-purpose
4. Compose ETF Excess Return from ETF Total Return - Cash Asset using WeightedSum with weights [1, -1]
5. Create Transaction Cost Asset node (models costs as compounding asset)
6. Create Replication Cost Asset node (models costs as compounding asset)
7. Create ARF Asset node (models adjusted return factor as asset)
8. **Generalize NodeOutput**: Extend `NodeOutput` enum to support `Composition(IndexComposition)` variant
9. **Create IndexComposition types**: Define `IndexComposition`, `CompositionRepresentation`, and `IndexComponent` (supporting nested indices)
10. **Implement flatten function**: Recursively flatten nested index compositions to flat asset-based compositions
11. **Update BaseIndex Node**: Output `IndexComposition` (weights or quantities based on config, can contain nested indices)
12. **Create conversion nodes**: WeightsToQuantities, QuantitiesToWeights (convert current level only, preserve nested structure)
13. **Create Flatten Node**: Flatten nested compositions to flat asset-based compositions
14. **Update IndexLevel Node**: Output `IndexComposition` (inherit from BaseIndex, update level, preserve nesting)
15. Implement basic index calculation (without futures rolling)
16. Store index composition as JSON (supports nested structure)
17. Support pull-mode calculation
18. Implement random weight generator with constraints

#### Phase 2: Futures Rolling
1. Implement RollingFuturesLevel node
2. Add futures contract data provider
3. Implement roll schedule logic
4. Add FX conversion

#### Phase 3: Hot Start
1. Implement checkpoint storage
2. Add checkpoint loading logic
3. Support hot start in pull mode
4. Optimize checkpoint frequency

#### Phase 4: Push Mode & Real-time
1. Support push-mode index calculation
2. Add SSE streaming for real-time updates
3. Implement incremental checkpoint updates

#### Phase 5: Rulebook Factory
1. Create rulebook parser (YAML/JSON)
2. Implement node factory from rulebook
3. Support multiple indices
4. Dynamic node registration

## Success Criteria

- [ ] Index level calculated correctly according to rulebook formulas
- [ ] Base index calculated from component levels and weights
- [ ] ETF excess return calculated with dividend reinvestment and rate switching
- [ ] Rolling futures level calculated with contract transitions
- [ ] Transaction and replication costs applied correctly
- [ ] Index level and composition persisted to database as JSON (supports nested indices)
- [ ] Composition supports nested indices (index components can be other indices)
- [ ] Flatten function collapses nested indices to flat asset-based composition
- [ ] Quantities representation accurately models position risk (e.g., offsetting cash in FX-hedged)
- [ ] Hot start works: can resume calculation from checkpoint
- [ ] Pull mode: can query historical index levels for date range
- [ ] Push mode: can calculate incrementally as new weights arrive
- [ ] Random weight generator satisfies constraints (-2.0 to +2.0 per component, -1.0 to +1.0 net)
- [ ] General-purpose calculators reusable across different indices
- [ ] Index-specific nodes generated from rulebook configuration

## Out of Scope (Future Phases)

- Corporate actions handling (defer to later phase)
- Market disruption handling (defer to later phase)
- Multiple indices support (defer to Phase 5)
- Real-time data feeds (use dummy/Yahoo Finance for MVP)
- PDF rulebook parsing (require YAML/JSON conversion)

