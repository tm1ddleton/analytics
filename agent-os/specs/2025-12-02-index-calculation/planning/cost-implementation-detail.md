# Detailed Implementation Plan: Transaction Costs and Replication Costs

## Overview

Transaction Costs and Replication Costs are modeled as return streams (negative returns) that can be subtracted from the base index return using WeightedSum. They need access to target weights data and component metadata.

## Transaction Cost Implementation

### Formula (from Rulebook Section 3.4)

**First day** (t = START_DATE + 1):
```
TTC_t = Σ ftc × ABS(w_i,t)
```
Where `ftc = 0.02% = 0.0002`

**Subsequent days**:
```
TTC_t = Σ ftc × ABS(w_i,t - w_i,t-1)
```

### Implementation Approach

#### 1. Transaction Cost Return Calculator
**Location**: `src/analytics/calculators.rs`

```rust
/// Calculate transaction cost return for a given day
/// 
/// # Arguments
/// * `current_weights` - Current target weights for all components [w_1, w_2, ..., w_n]
/// * `previous_weights` - Previous target weights [w_1_prev, w_2_prev, ..., w_n_prev]
/// * `is_first_day` - Whether this is the first calculation day
/// * `fixed_transaction_cost_rate` - Fixed transaction cost rate (default: 0.0002 = 0.02%)
/// 
/// # Returns
/// Negative return representing transaction cost (to be subtracted from base return)
pub fn transaction_cost_return(
    current_weights: &[f64],
    previous_weights: Option<&[f64]>,
    is_first_day: bool,
    fixed_transaction_cost_rate: f64,
) -> f64 {
    if is_first_day {
        // First day: TTC = Σ ftc × ABS(w_i,t)
        current_weights.iter()
            .map(|w| fixed_transaction_cost_rate * w.abs())
            .sum::<f64>()
    } else {
        // Subsequent days: TTC = Σ ftc × ABS(w_i,t - w_i,t-1)
        let prev = previous_weights.expect("Previous weights required for non-first day");
        current_weights.iter()
            .zip(prev.iter())
            .map(|(w_curr, w_prev)| fixed_transaction_cost_rate * (w_curr - w_prev).abs())
            .sum::<f64>()
    }
}
```

**Note**: Returns a positive value representing the cost, which will be negated when used in WeightedSum.

#### 2. Transaction Cost Container
**Location**: `src/analytics/containers.rs`

```rust
/// Container for transaction cost calculation
pub struct TransactionCostAnalytic {
    fixed_transaction_cost_rate: f64,
}

impl TransactionCostAnalytic {
    pub fn new(fixed_transaction_cost_rate: f64) -> Self {
        Self {
            fixed_transaction_cost_rate,
        }
    }

    /// Compute transaction cost return
    /// 
    /// # Arguments
    /// * `current_weights` - Current target weights
    /// * `previous_weights` - Previous target weights (None for first day)
    /// 
    /// # Returns
    /// Transaction cost as a return (positive value, will be negated in WeightedSum)
    pub fn compute(
        &self,
        current_weights: &[f64],
        previous_weights: Option<&[f64]>,
    ) -> f64 {
        let is_first_day = previous_weights.is_none();
        transaction_cost_return(
            current_weights,
            previous_weights,
            is_first_day,
            self.fixed_transaction_cost_rate,
        )
    }
}
```

#### 3. Transaction Cost Executor
**Approach**: Use `RecursiveExecutor` to maintain previous weights state

**State to maintain**:
- Previous target weights: `Vec<f64>`
- Flag for first day

**Execution flow**:
1. Get current target weights from data source (via node params or data provider)
2. Load previous weights from state (if not first day)
3. Calculate transaction cost return using calculator
4. Store current weights as previous weights for next iteration
5. Return negative return (to be subtracted)

#### 4. Transaction Cost Definition
**Location**: `src/analytics/registry.rs`

```rust
struct TransactionCostDefinition {
    executor: Box<dyn AnalyticExecutor>,
    fixed_transaction_cost_rate: f64,
}

impl TransactionCostDefinition {
    fn new(fixed_transaction_cost_rate: f64) -> Self {
        let container = Arc::new(TransactionCostAnalytic::new(fixed_transaction_cost_rate));
        
        TransactionCostDefinition {
            executor: Box::new(RecursiveExecutor::new(
                move |node, previous_state, timestamp, value| {
                    // Get current weights from data source
                    let current_weights = get_target_weights(node, timestamp)?;
                    
                    // Get previous weights from state
                    let previous_weights = previous_state
                        .and_then(|s| s.downcast_ref::<Vec<f64>>())
                        .map(|w| w.as_slice());
                    
                    // Calculate transaction cost return
                    let cost_return = container.compute(&current_weights, previous_weights);
                    
                    // Store current weights as new state
                    let new_state = Box::new(current_weights);
                    
                    // Return negative return (cost to be subtracted)
                    Ok(NodeOutput::new(timestamp, -cost_return))
                },
            )),
            fixed_transaction_cost_rate,
        }
    }
}
```

#### 5. Target Weights Data Source

**Options for accessing target weights**:

**Option A: Via Data Provider**
- Store target weights in database/DataProvider
- Query by date: `get_target_weights(index_name, date) -> HashMap<component_id, weight>`
- Convert to ordered vector matching component order

**Option B: Via Node Parameters**
- Pass weights as node parameters (for static/known weights)
- Less flexible for dynamic weights

**Option C: Via Separate Weight Provider Node**
- Create a `TargetWeightsNode` that provides weights as time series
- Transaction Cost node depends on TargetWeightsNode
- More composable but adds complexity

**Recommendation**: Option A (Data Provider) for MVP, Option C for future flexibility.

## Replication Cost Implementation

### Formula (from Rulebook Section 3.5)

```
TRC_t = Σ RC × ABS(w_i,t) × DCF/365
```

Where:
- `RC = 0.15% = 0.0015` for futures (Bond Futures, FX Futures, Equity Futures)
- `RC = 0.0%` for ETFs
- `DCF = calendar days between t and t-1`

### Implementation Approach

#### 1. Replication Cost Return Calculator
**Location**: `src/analytics/calculators.rs`

```rust
/// Calculate replication cost return for a given day
/// 
/// # Arguments
/// * `weights` - Current target weights for all components [w_1, w_2, ..., w_n]
/// * `component_types` - Asset types for each component ["ETF", "EquityFutures", ...]
/// * `day_count_fraction` - Day count fraction (DCF/365)
/// * `futures_replication_cost_rate` - Replication cost rate for futures (default: 0.0015 = 0.15%)
/// 
/// # Returns
/// Negative return representing replication cost (to be subtracted from base return)
pub fn replication_cost_return(
    weights: &[f64],
    component_types: &[AssetType],
    day_count_fraction: f64,
    futures_replication_cost_rate: f64,
) -> f64 {
    weights.iter()
        .zip(component_types.iter())
        .map(|(w, asset_type)| {
            let rc = match asset_type {
                AssetType::Future => futures_replication_cost_rate,
                AssetType::Equity => 0.0, // ETFs have 0% replication cost
            };
            rc * w.abs() * day_count_fraction
        })
        .sum::<f64>()
}
```

**Note**: Need to extend `AssetType` enum or create separate `IndexComponentType` enum to distinguish ETFs from Futures.

#### 2. Day Count Fraction Calculator
**Location**: `src/analytics/calculators.rs`

```rust
/// Calculate day count fraction between two dates
/// 
/// # Arguments
/// * `start_date` - Start date (t-1)
/// * `end_date` - End date (t)
/// 
/// # Returns
/// Day count fraction (calendar days / 365)
pub fn day_count_fraction(
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> f64 {
    let days = (end_date - start_date).num_days() as f64;
    days / 365.0
}
```

#### 3. Replication Cost Container
**Location**: `src/analytics/containers.rs`

```rust
/// Container for replication cost calculation
pub struct ReplicationCostAnalytic {
    futures_replication_cost_rate: f64,
}

impl ReplicationCostAnalytic {
    pub fn new(futures_replication_cost_rate: f64) -> Self {
        Self {
            futures_replication_cost_rate,
        }
    }

    /// Compute replication cost return
    /// 
    /// # Arguments
    /// * `weights` - Current target weights
    /// * `component_types` - Asset types for each component
    /// * `day_count_fraction` - Day count fraction
    /// 
    /// # Returns
    /// Replication cost as a return (positive value, will be negated in WeightedSum)
    pub fn compute(
        &self,
        weights: &[f64],
        component_types: &[AssetType],
        day_count_fraction: f64,
    ) -> f64 {
        replication_cost_return(
            weights,
            component_types,
            day_count_fraction,
            self.futures_replication_cost_rate,
        )
    }
}
```

#### 4. Replication Cost Executor
**Approach**: Use `RecursiveExecutor` to track previous date for DCF calculation

**State to maintain**:
- Previous date: `NaiveDate`
- Component types: `Vec<AssetType>` (static, from rulebook)

**Execution flow**:
1. Get current target weights from data source
2. Get current date from timestamp
3. Calculate day count fraction: `DCF = (current_date - previous_date).num_days() / 365.0`
4. Calculate replication cost return using calculator
5. Store current date as previous date
6. Return negative return (to be subtracted)

#### 5. Replication Cost Definition
**Location**: `src/analytics/registry.rs`

```rust
struct ReplicationCostDefinition {
    executor: Box<dyn AnalyticExecutor>,
    futures_replication_cost_rate: f64,
    component_types: Vec<AssetType>,
}

impl ReplicationCostDefinition {
    fn new(futures_replication_cost_rate: f64, component_types: Vec<AssetType>) -> Self {
        let container = Arc::new(ReplicationCostAnalytic::new(futures_replication_cost_rate));
        let component_types_clone = component_types.clone();
        
        ReplicationCostDefinition {
            executor: Box::new(RecursiveExecutor::new(
                move |node, previous_state, timestamp, value| {
                    // Get current weights from data source
                    let current_weights = get_target_weights(node, timestamp)?;
                    
                    // Get previous date from state
                    let current_date = timestamp.date_naive();
                    let previous_date = previous_state
                        .and_then(|s| s.downcast_ref::<NaiveDate>())
                        .copied()
                        .unwrap_or(current_date); // Use current_date if first day (DCF = 0)
                    
                    // Calculate day count fraction
                    let dcf = day_count_fraction(previous_date, current_date);
                    
                    // Calculate replication cost return
                    let cost_return = container.compute(
                        &current_weights,
                        &component_types_clone,
                        dcf,
                    );
                    
                    // Store current date as new state
                    let new_state = Box::new(current_date);
                    
                    // Return negative return (cost to be subtracted)
                    Ok(NodeOutput::new(timestamp, -cost_return))
                },
            )),
            futures_replication_cost_rate,
            component_types,
        }
    }
}
```

## Data Dependencies

### Target Weights Data Source

**Database Schema Extension**:
```sql
CREATE TABLE IF NOT EXISTS index_target_weights (
    index_name TEXT NOT NULL,
    date TEXT NOT NULL,
    component_id TEXT NOT NULL,
    weight REAL NOT NULL,
    PRIMARY KEY (index_name, date, component_id)
);
```

**Data Provider Extension**:
```rust
pub trait DataProvider {
    // ... existing methods ...
    
    /// Get target weights for an index on a specific date
    fn get_target_weights(
        &self,
        index_name: &str,
        date: NaiveDate,
    ) -> Result<HashMap<String, f64>, DataProviderError>;
    
    /// Get target weights as ordered vector matching component order
    fn get_target_weights_ordered(
        &self,
        index_name: &str,
        date: NaiveDate,
        component_order: &[String],
    ) -> Result<Vec<f64>, DataProviderError>;
}
```

**Helper Function**:
```rust
fn get_target_weights(
    node: &Node,
    timestamp: DateTime<Utc>,
) -> Result<Vec<f64>, DagError> {
    // Extract index name and component order from node params
    let index_name = node.params.get("index_name")?;
    let component_order: Vec<String> = serde_json::from_str(
        node.params.get("component_order")?
    )?;
    
    // Get data provider from node context
    let data_provider = node.data_provider.as_ref()?;
    
    // Query target weights
    let date = timestamp.date_naive();
    data_provider.get_target_weights_ordered(
        index_name,
        date,
        &component_order,
    ).map_err(|e| DagError::Other(format!("Failed to get target weights: {}", e)))
}
```

## Integration with Index Level Calculation

### DAG Structure

```
BaseIndex Return ──┐
ARF Asset Return ─┤
TTC Return ───────├──> WeightedSum([1, -1, -1, -1]) ──> Net Return ──> Index Level
TRC Return ───────┘
```

### WeightedSum Configuration

**Weights**: `[1.0, -1.0, -1.0, -1.0]`
- `+1.0` for BaseIndex Return (add base return)
- `-1.0` for ARF Asset Return (subtract ARF cost)
- `-1.0` for TTC Return (subtract transaction cost)
- `-1.0` for TRC Return (subtract replication cost)

**Formula**:
```
NetReturn_t = BaseReturn_t - ARF_t - TTC_t - TRC_t
Index_t = max[0, Index_{t-1} × (1 + NetReturn_t)]
```

## Testing Strategy

### Transaction Cost Tests

1. **First day calculation**:
   - Input: `weights = [0.1, 0.2, -0.15]`, `previous_weights = None`
   - Expected: `TTC = 0.0002 × (0.1 + 0.2 + 0.15) = 0.00009`

2. **Weight change calculation**:
   - Input: `weights = [0.1, 0.25, -0.15]`, `previous_weights = [0.1, 0.2, -0.15]`
   - Expected: `TTC = 0.0002 × (0.0 + 0.05 + 0.0) = 0.00001`

3. **No weight change**:
   - Input: `weights = [0.1, 0.2, -0.15]`, `previous_weights = [0.1, 0.2, -0.15]`
   - Expected: `TTC = 0.0`

### Replication Cost Tests

1. **Futures-only weights**:
   - Input: `weights = [0.1, 0.2]`, `types = [Future, Future]`, `DCF = 1/365`
   - Expected: `TRC = 0.0015 × (0.1 + 0.2) × (1/365) = 0.00000123...`

2. **Mixed ETF and Futures**:
   - Input: `weights = [0.1, 0.2]`, `types = [ETF, Future]`, `DCF = 1/365`
   - Expected: `TRC = 0.0 × 0.1 × (1/365) + 0.0015 × 0.2 × (1/365) = 0.00000082...`

3. **Zero weights**:
   - Input: `weights = [0.0, 0.0]`, `types = [Future, Future]`, `DCF = 1/365`
   - Expected: `TRC = 0.0`

## Summary

### Key Design Decisions

1. **Costs as Return Streams**: Both TTC and TRC are calculated as return values (not levels), making them composable with WeightedSum
2. **State Management**: Use `RecursiveExecutor` to maintain previous weights (TTC) and previous date (TRC)
3. **Data Source**: Target weights accessed via DataProvider extension (database query)
4. **Component Metadata**: Component types stored in node definition (from rulebook config)
5. **Negative Returns**: Costs returned as positive values, negated when used in WeightedSum

### Implementation Order

1. Extend DataProvider trait with target weights methods
2. Create database schema for target weights
3. Implement day_count_fraction calculator
4. Implement transaction_cost_return calculator
5. Implement replication_cost_return calculator
6. Create TransactionCostAnalytic container
7. Create ReplicationCostAnalytic container
8. Create TransactionCostDefinition with RecursiveExecutor
9. Create ReplicationCostDefinition with RecursiveExecutor
10. Integrate into IndexLevel node using WeightedSum
11. Write comprehensive tests


