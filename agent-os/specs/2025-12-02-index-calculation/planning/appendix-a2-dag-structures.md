# Appendix A.2: DAG Structures (Detailed)

This document provides detailed DAG structures for all three indices, showing how components are wired together.

## SOLSTAE Index DAG

### Complete Structure

```
StartingState ──> BaseIndex ──> IndexLevel (WeightedSum of components)
                                ├── ETF Total Return
                                ├── Futures Rolling (RollingFuturesLevel)
                                ├── Transaction Costs (asset)
                                ├── Replication Costs (asset)
                                └── Cash Asset (SOFR/LIBOR)
```

### Detailed Component Breakdown

#### ETF Total Return Path
```
ETF Price Data ──> ETF Total Return (RecursiveExecutor)
                    ├── Uses: compound_return calculator
                    ├── Maintains: previous ETF level
                    └── Formula: ETFLevel_t = ETFLevel_{t-1} × ((Price_t + div_t) / Price_{t-1})
```

#### ETF Excess Return Path
```
ETF Total Return ──┐
                   ├──> ExcessReturn (WeightedSum [1, -1])
Cash Asset Return ─┘     ├── Uses: weighted_sum calculator
                          └── Formula: ETFReturn - CashReturn
```

#### Futures Rolling Path
```
Active Contract Price ──┐
                        ├──> Active Return (ArithReturnAnalytic)
Next Contract Price ────┘
                        ├──> Next Return (ArithReturnAnalytic)
                        │
                        ├──> WeightedSum (w_active, w_next) ──> Futures Return
                        │     ├── Uses: weighted_sum calculator
                        │     └── Roll weights from schedule
                        │
FX Rate ────────────────┼──> FX Conversion ──┐
                        │                     │
                        └─────────────────────┼──> Rolling Futures Level (RecursiveExecutor)
                                              │     ├── Uses: compound_return calculator
                                              │     └── Formula: RFL_t = RFL_{t-1} × (1 + FuturesReturn_t)
```

#### Base Index Path
```
StartingState ────┐ (provides initial base index level + composition)
                  │
ETF Excess Return ──> ArithReturn ──┐
Futures Level ──────> ArithReturn ─┤
                                    ├──> WeightedSumMerge (target weights)
                                    │     ├── Uses: weighted_sum calculator
                                    │     └── Weights from DataProvider
                                    │
                                    └──> BaseIndex (RecursiveExecutor)
                                          ├── Uses: compound_return calculator
                                          ├── Maintains: previous base index level + composition
                                          └── Output: IndexComposition (weights or quantities)
```

#### Index Level Path
```
StartingState ────┐ (provides initial index level + composition)
                  │
BaseIndex Return ──┤
ARF Asset ─────────┤
TTC Asset ─────────├──> WeightedSum (weights: [+1, -1, -1, -1])
TRC Asset ─────────┘     ├── Uses: weighted_sum calculator
                         └── Formula: NetReturn = BaseReturn - ARF - TTC - TRC
                         │
                         └──> Index Level (RecursiveExecutor)
                               ├── Uses: compound_return calculator
                               ├── Maintains: previous index level + composition
                               └── Output: IndexComposition (final index level)
```

#### Cost Assets Path
```
Transaction Cost Calculator ──> TTC Asset (RecursiveExecutor)
                                  ├── Uses: compound_return calculator
                                  └── Formula: TTC_t = TTC_{t-1} × (1 + TTC_return_t)

Replication Cost Calculator ──> TRC Asset (RecursiveExecutor)
                                  ├── Uses: compound_return calculator
                                  └── Formula: TRC_t = TRC_{t-1} × (1 + TRC_return_t)

SOFR/LIBOR Rate ──────────────> Cash Asset (RecursiveExecutor)
                                  ├── Uses: compound_return calculator
                                  └── Formula: Cash_t = Cash_{t-1} × (1 + rate × DCF/365)
```

---

## Variance Replication Index DAG

### Complete Structure

```
Option Prices (TWAP) ──> Variance Strike Calculator ──> Portfolio Builder ──> Option Portfolio
                                                                              ├──> Option Delta ──> Delta Hedge Position (asset)
                                                                              ├──> Option Cash Flows
                                                                              └──> Portfolio Mark-to-Market

Total Return Level ──> Total Return ──┐
                                      ├──> Excess Return (WeightedSum [1, -1]) ──> Excess Return Level (ReturnCompounder)
Cash Asset ──> Cash Return ───────────┘
```

### Detailed Component Breakdown

#### Variance Strike Calculation Path
```
Option Prices (TWAP) ──> Variance Strike Calculator (index-specific)
                          ├── Dependencies: Option prices
                          ├── Formula: Variance-specific calculation
                          └── Output: Variance strike value
```

#### Option Portfolio Path
```
Variance Strike ──> Variance Replication Portfolio Builder (index-specific)
                     ├── Dependencies: Variance strike, option prices
                     ├── Formula: Portfolio-specific logic
                     └── Output: Option portfolio (positions)
                     │
                     ├──> Option Pricing (OptionPricingDefinition)
                     │     ├── Uses: option_pricing calculator
                     │     └── Output: Premium + Greeks
                     │
                     ├──> Option Delta (OptionDeltaDefinition)
                     │     ├── Extracts: delta from OptionPricing
                     │     └── Output: Delta value
                     │
                     ├──> Delta Hedge Position (DeltaHedgePositionDefinition)
                     │     ├── Uses: delta value
                     │     ├── Treated as: Regular asset
                     │     └── Output: Hedge position value
                     │
                     ├──> Option Cash Flows (OptionCashFlowsDefinition)
                     │     └── Output: Cash flow stream
                     │
                     └──> Portfolio Mark-to-Market (PortfolioMarkToMarketDefinition)
                           ├── Uses: portfolio_mark_to_market calculator
                           ├── Dependencies: Option positions, option prices
                           └── Output: Portfolio value
```

#### Total Return Path
```
Portfolio Mark-to-Market ──> Total Return Level (TotalReturnLevelDefinition)
                               ├── Uses: weighted_sum calculator
                               ├── Maintains: Previous portfolio level
                               └── Output: Total return level

Cash Asset ──> Cash Return ──> Cash Return (ArithReturnAnalytic)
```

#### Excess Return Path
```
Total Return Level ──> Total Return ──┐
                                       ├──> Excess Return (WeightedSum [1, -1])
Cash Asset ──> Cash Return ────────────┘     ├── Uses: weighted_sum calculator
                                             └── Formula: TotalReturn - CashReturn
                                             │
                                             └──> Excess Return Level (ReturnCompounderDefinition)
                                                   ├── Uses: compound_return calculator
                                                   ├── Maintains: Previous excess return level
                                                   └── Output: Final index level
```

---

## AIPEX5 Index DAG

### Complete Structure

```
Base Index Level ──> Base Index Return ──┐
                                         ├──> Base Excess Return (WeightedSum [1, -1]) ──┐
LIBOR Rate Asset ──> LIBOR Return ───────┘                                                │
                                                                                          ├──> Scaled Excess Return (Exposure × Base Excess Return) ──┐
Exposure ───────────────────────────────────────────────────────────────────────────────┘                                                          │
                                                                                                                                                  ├──> Net Return (WeightedSum [1, -1]) ──> Index Level (ReturnCompounder)
Fee Asset ──> Fee Return ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Detailed Component Breakdown

#### Realized Volatility Path
```
Base Index Returns ──> Realized Volatility 21-Day (RealizedVolatilityDefinition)
                        ├── Container: RealizedVolatilityAnalytic
                        ├── Executor: WindowedAnalyticExecutor (window: 21)
                        ├── Calculator: root_mean_square
                        └── Output: 21-day realized volatility

Base Index Returns ──> Realized Volatility 63-Day (RealizedVolatilityDefinition)
                        ├── Container: RealizedVolatilityAnalytic
                        ├── Executor: WindowedAnalyticExecutor (window: 63)
                        ├── Calculator: root_mean_square
                        └── Output: 63-day realized volatility

21-Day Volatility ──┐
                    ├──> Max Volatility (MaxVolatilityDefinition)
63-Day Volatility ──┘     ├── Calculator: max_value
                          ├── Executor: MergeExecutor
                          └── Output: Max of 21-day and 63-day volatilities
```

#### Volatility Targeting Path
```
Max Volatility ──> Volatility Targeting Exposure (VolatilityTargetingExposureDefinition)
                    ├── Calculator: volatility_targeting_exposure
                    ├── Executor: RecursiveExecutor (maintains previous exposure)
                    ├── Dependencies: Target volatility, realized volatility
                    └── Formula: exposure_t = exposure_{t-1} × (target_vol / realized_vol_t)
                    │
                    └──> Output: Dynamic exposure (0 to 1)
```

#### Base Excess Return Path
```
Base Index Level ──> Base Index Return (ArithReturnAnalytic) ──┐
                                                               ├──> Base Excess Return (WeightedSum [1, -1])
LIBOR Rate Asset ──> LIBOR Return (ArithReturnAnalytic) ───────┘     ├── Uses: weighted_sum calculator
                                                                      └── Formula: BaseReturn - LIBORReturn
```

#### Scaled Excess Return Path
```
Base Excess Return ──┐
                     ├──> Scaled Excess Return (Multiply)
Exposure ────────────┘     ├── Calculator: multiply (or weighted_sum with [exposure, 0])
                          └── Formula: ScaledReturn = Exposure × BaseExcessReturn
```

#### Net Return Path
```
Scaled Excess Return ──┐
                       ├──> Net Return (WeightedSum [1, -1])
Fee Asset Return ──────┘     ├── Uses: weighted_sum calculator
                              └── Formula: NetReturn = ScaledReturn - FeeReturn
                              │
                              └──> Index Level (ReturnCompounderDefinition)
                                    ├── Uses: compound_return calculator
                                    ├── Executor: RecursiveExecutor (maintains previous level)
                                    └── Formula: Level_t = Level_{t-1} × (1 + NetReturn_t)
```

---

## Common Patterns

### Pattern 1: Return Calculation
```
Asset Level ──> Return (ArithReturnAnalytic or LogReturnAnalytic)
                 └── Uses: ReturnAnalytic trait
```

### Pattern 2: Excess Return
```
Asset Return ──┐
               ├──> Excess Return (WeightedSum [1, -1])
Cash Return ───┘     └── Uses: weighted_sum calculator
```

### Pattern 3: Compounding
```
Return ──> Level (ReturnCompounderDefinition)
            ├── Uses: compound_return calculator
            ├── Executor: RecursiveExecutor
            └── Maintains: Previous level
```

### Pattern 4: Volatility Calculation
```
Returns ──> Volatility (VolatilityDefinition or RealizedVolatilityDefinition)
             ├── Container: VolatilityAnalytic trait
             ├── Executor: WindowedAnalyticExecutor
             └── Calculator: population_std_dev or root_mean_square
```

### Pattern 5: Weighted Aggregation
```
Component1 Return ──┐
Component2 Return ──┤
...                 ├──> WeightedSumMerge (WeightedSum calculator)
ComponentN Return ──┘     └── Uses: weighted_sum calculator with target weights
```

---

## DAG Execution Flow

### Push Mode (Real-time)
1. **StartingState** loads previous checkpoint or initial values
2. **Data nodes** receive new market data
3. **Calculation nodes** execute in topological order:
   - Return calculations
   - Volatility calculations
   - Aggregations
   - Compounding
4. **Index level** node produces final result
5. **Persistence layer** saves checkpoint (separate concern)

### Pull Mode (Historical)
1. **StartingState** loads checkpoint or initial values
2. **Data nodes** query historical data for date range
3. **Calculation nodes** execute for each date in range:
   - State rebuilt incrementally
   - Results accumulated into time series
4. **Index level** node produces time series
5. **Return** complete historical time series

---

## State Management

### Recursive Nodes
- **State**: Stored per-node in DAG
- **Type**: `Option<T>` where T is state type (e.g., `f64`, `IndexComposition`)
- **Persistence**: Can be serialized for hot start
- **Reset**: Cleared and rebuilt in pull mode

### Windowed Nodes
- **State**: Rolling window buffer
- **Type**: `Vec<f64>` (window of values)
- **Persistence**: Not persisted (rebuilt from data)
- **Reset**: Cleared and rebuilt in pull mode

### Merge Nodes
- **State**: None (stateless)
- **Persistence**: Not applicable
- **Reset**: Not applicable

