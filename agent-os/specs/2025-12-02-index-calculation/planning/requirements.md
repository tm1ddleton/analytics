# Requirements: Index Calculation with Rulebook Support

## Rulebook Analysis: Solactive Systematic Trend Alpha Replicator Excess Return Index

### Index Overview
- **Name**: Solactive Systematic Trend Alpha Replicator Excess Return Index
- **Type**: Multi-asset excess return index
- **Currency**: USD
- **Start Date**: 2006-07-13
- **Initial Level**: 100
- **Components**: 13 static components
  - 4 ETFs (EEM.P, GLD.P, XLE.P, XME.P)
  - 2 FX Futures (EUR/USD, JPY/USD)
  - 4 Equity Futures (E-mini S&P 500, E-mini Nasdaq-100, Japan 225, Euro 50)
  - 3 Bond Futures (10y Treasury, 2y Treasury, Bund)

### Key Calculation Requirements

#### 1. Base Index Calculation (Section 3.2)
**Formula**: 
```
B_t = B_{t-1} × (1 + Σ w_i,t × (IC_i,t / IC_i,t-1 - 1))
```
Where:
- `B_t`: Base Index level at time t
- `w_i,t`: Target weight of component i for day t (provided by INDEX CONSULTANT)
- `IC_i,t`: Level of component i at time t

**Requirements**:
- Daily rebalancing using target weights
- Component levels calculated differently based on asset type:
  - **ETFs**: Excess return level (dividend-adjusted, SOFR-adjusted)
  - **Futures**: Rolling futures level (with contract rolling logic)

#### 2. Index Level Calculation (Section 3.1)
**Formula**:
```
Index_t = max[0, Index_{t-1} × (B_t/B_{t-1} - ARF × DCF/365 - TTC_t - TRC_t)]
```
Where:
- `ARF`: Adjusted Return Factor (0.4% p.a.)
- `TTC_t`: Total Transaction Cost
- `TRC_t`: Total Replication Cost
- `DCF`: Day count fraction

**Requirements**:
- Apply adjusted return factor daily
- Calculate transaction costs based on weight changes
- Calculate replication costs based on component types
- Floor at zero (max function)

#### 3. ETF Level Calculation (Section 3.3)
**Formula**:
```
ETFLevel_t = ETFLevel_{t-1} × ((ETF_Close_t + div_t) / ETF_Close_{t-1} - (rate_{t-2} × DCF/365))
```
Where:
- `div_t`: Cash dividends with ex-date on day t
- `rate_t`: SOFR (after 2020-12-31) or LIBOR - 0.26161% (before)

**Requirements**:
- Dividend reinvestment
- Excess return over financing rate
- Rate switch date handling (2020-12-31)

#### 4. Futures Rolling Logic (Appendix 1)
**Key Concepts**:
- Active Contract: Contract expiring per roll schedule (Table 4)
- Next Active Contract: Contract to roll into (Table 5)
- Roll Period: Gradual transition over roll days (typically 5 days)
- Roll Anchor: First Notice Day or Expiry Day (per component)

**Requirements**:
- Determine active/next active contracts based on calendar month
- Calculate contract weights during roll period
- Handle FX conversion for non-USD futures
- Calculate rolling futures level with contract transitions

#### 5. Transaction Cost Calculation (Section 3.4)
**Formula**:
- First day: `TTC_t = Σ ftc × ABS(w_i,t)`
- Subsequent days: `TTC_t = Σ ftc × ABS(w_i,t - w_i,t-1)`
- `ftc`: Fixed transaction cost (0.02%)

**Requirements**:
- Track weight changes between days
- Apply transaction cost only on rebalancing

#### 6. Replication Cost Calculation (Section 3.5)
**Formula**:
```
TRC_t = Σ RC × ABS(w_i,t) × DCF/365
```
Where:
- `RC`: 0.15% for futures, 0.0% for ETFs

**Requirements**:
- Different rates for different asset types
- Daily accrual based on absolute weights

### Data Requirements

#### Input Data Needed
1. **Component Prices**:
   - ETF closing prices (with dividends)
   - Futures settlement prices (active and next active contracts)
   - FX rates (for non-USD futures)

2. **Target Weights**:
   - Daily weights for each component (generated randomly for MVP)
   - Constraints:
     - Individual component weights: -200% to +200% (i.e., -2.0 to +2.0)
     - Net exposure (sum of all weights): -100% to +100% (i.e., -1.0 to +1.0)
   - Format: Component identifier → weight mapping
   - Generation: Random weight generator with constraint validation

3. **Reference Rates**:
   - SOFR (RIC: USDSOFR=) for dates >= 2020-12-31
   - 3-Month USD LIBOR (RIC: USD3MFSR=) minus 0.26161% for dates < 2020-12-31

4. **Futures Metadata**:
   - Expiration dates
   - First notice dates
   - Roll schedules (Tables 4 & 5)

#### Data Source Strategy
- **Yahoo Finance**: Use for ETFs and available futures data
- **Dummy Data**: Generate for:
  - Futures contracts not available on Yahoo Finance
  - FX rates (if not available)
  - Reference rates (SOFR/LIBOR)
  - Target weights (random generation with constraints)

### Persistence Requirements

#### What to Persist
1. **Index Level**: Single value per calculation day
   - Date, Index Level, Base Index Level

2. **Composition**: JSON object per calculation day
   - Date
   - Component weights: `{component_id: weight, ...}`
   - Component levels: `{component_id: level, ...}`

3. **Component Levels** (for hot start):
   - ETF levels (excess return levels)
   - Rolling futures levels
   - Per component, per date

#### Database Schema Considerations
- Extend existing analytics table or create new index-specific tables
- Store composition as JSON in existing schema
- Support querying by date range with hot start capability

### Node Architecture Proposal

Based on the calculation requirements, the following nodes are needed:

#### General-Purpose Nodes (Reusable)
1. **WeightedSum Calculator**: Pure math function
   - Domain-agnostic: `Σ w_i × value_i`
   - Can be used for any weighted aggregation
   - Location: `src/analytics/calculators.rs`

2. **Cash Asset Node**: Model cash/funding as an asset
   - Accrues interest: `Cash_t = Cash_{t-1} × (1 + rate × DCF/365)`
   - Cash returns calculated using existing `ArithReturnAnalytic`
   - Reusable for any funding calculation
   - General-purpose, not SOLSTAE-specific
   - **Excess Return**: Use WeightedSum with weights [1, -1] to get `asset_return - cash_return`

4. **Day Count / Funding Asset Node**: Model day count effects as an asset
   - Any compounding effect becomes an asset
   - Can be differenced with other assets
   - Input: Price, previous level, dividend, rate, day count
   - Output: Excess return level

3. **TransactionCost Calculator**: Calculate transaction costs
   - Input: Current weights, previous weights, fixed cost rate
   - Output: Total transaction cost

4. **ReplicationCost Calculator**: Calculate replication costs
   - Input: Weights, asset types, day count
   - Output: Total replication cost

5. **DayCountFraction Calculator**: Calculate day count between dates
   - Input: Start date, end date
   - Output: Day count fraction

#### General-Purpose Nodes (Also Reusable)
1. **RollingFuturesLevel Node**: Calculate rolling futures level
   - Should be implemented as general-purpose feature (not SOLSTAE-specific)
   - Uses WeightedSum calculator (pure math)
   - Requires roll schedule configuration

2. **IndexLevel Node**: Calculate final index level
   - Combines base index, ARF, TTC, TRC
   - Index-specific formula

3. **ETFExcessReturn Node**: ETF-specific excess return calculation
   - Handles dividend reinvestment
   - Rate switch date logic

4. **BaseIndex Node**: Calculate base index from components
   - Combines component levels with target weights

### Node Registry Strategy

**Proposal**: Hybrid Approach
1. **General-Purpose Nodes**: Add to main `AnalyticRegistry`
   - WeightedSum, ExcessReturn, TransactionCost, ReplicationCost, DayCountFraction

2. **Index-Specific Nodes**: Use factory-based generation
   - Create nodes from rulebook configuration (YAML/JSON)
   - Register in separate `IndexNodeRegistry` or namespace
   - Nodes generated based on:
     - Component definitions
     - Roll schedules
     - Calculation formulas

3. **Factory Pattern**:
   - Parse rulebook → Generate node configs → Create nodes via factory
   - Nodes can be registered dynamically or at startup

### Hot Start Requirements

#### Capability Needed
- When requesting index for date range [start, end]:
  1. Query database for last known index level before start date
  2. Query database for component levels/composition
  3. Resume calculation from that point forward
  4. Avoid recalculating entire history

#### Implementation Approach
- Store checkpoint data:
  - Last calculated date
  - Index level at checkpoint
  - Component levels at checkpoint
  - Composition at checkpoint
- Query logic:
  - Find latest checkpoint <= start date
  - Use checkpoint values as initial state
  - Calculate forward from checkpoint

### Push vs Pull Mode

#### Pull Mode (Historical)
- Query index level for date range
- Use hot start if available
- Calculate missing dates forward

#### Push Mode (Real-time)
- Receive new target weights daily
- Calculate new index level incrementally
- Store results as they're calculated
- Support SSE streaming for real-time updates

### Integration Points

1. **Data Provider**: Extend to support:
   - ETF data with dividends
   - Futures data (multiple contracts)
   - FX rates
   - Reference rates (SOFR/LIBOR)

2. **DAG Framework**: 
   - Support dynamic node registration
   - Support factory-based node creation
   - Support checkpoint/resume for hot start

3. **Analytics Architecture**:
   - Calculator: Pure math functions
   - Container: Index-specific state
   - Executor: Orchestrate calculation with hot start
   - Definition: Register index nodes, define DAG structure

### Target Weight Generation

**Random Generation with Constraints**:
- Generate random weights for each component daily
- Constraints:
  - Individual component: -2.0 ≤ weight ≤ +2.0 (200% long/short limit)
  - Net exposure: -1.0 ≤ Σ weights ≤ +1.0 (100% net limit)
- Implementation: Weight generator that ensures constraints are met

### Open Questions

2. **Futures Data**:
   - Can we get futures contract data from Yahoo Finance?
   - How to handle contract rolling if data unavailable?

3. **Corporate Actions**:
   - Scope for MVP? (Can defer to later phase)

4. **Market Disruption**:
   - Scope for MVP? (Can defer to later phase)

5. **Rulebook Format**:
   - Should we parse PDF or convert to structured format (YAML/JSON)?
   - How to handle rulebook updates/versions?

