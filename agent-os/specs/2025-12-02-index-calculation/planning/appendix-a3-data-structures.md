# Appendix A.3: Data Structures (Detailed)

This document provides detailed data structure definitions for the index calculation framework.

## Index Composition Structures

### `IndexComposition`

**Purpose**: Represents index composition output (level + components)

**Location**: `src/analytics/index_composition.rs`

```rust
use chrono::{DateTime, Utc};

/// Index composition output structure
pub struct IndexComposition {
    /// Index level value
    pub level: f64,
    /// Composition representation (weights or quantities)
    pub representation: CompositionRepresentation,
    /// Timestamp of this composition
    pub timestamp: DateTime<Utc>,
}
```

**Key Features**:
- Supports both weights-based and quantities-based representation
- Includes timestamp for time-series tracking
- Can contain nested indices (recursive structure)

---

### `CompositionRepresentation`

**Purpose**: Enum representing composition format (weights vs quantities)

```rust
/// Composition representation (weights or quantities)
pub enum CompositionRepresentation {
    /// Weights-based representation
    Weights {
        /// Component list (can contain assets or nested indices)
        components: Vec<IndexComponent>,
    },
    /// Quantities-based representation
    Quantities {
        /// Component list (can contain assets or nested indices)
        components: Vec<IndexComponent>,
        /// Divisor (typically 1.0 when converting from weights)
        divisor: f64,
    },
}
```

**Usage**:
- **Weights**: Used for target weight-based indices (e.g., SOLSTAE)
- **Quantities**: Used for position-based indices (e.g., variance replication)

---

### `IndexComponent`

**Purpose**: Represents a single component in index composition (asset or nested index)

```rust
/// Index component (can be asset or nested index)
pub enum IndexComponent {
    /// Direct asset component (e.g., ETF, futures contract, option)
    Asset {
        /// Component identifier (e.g., "EEM.P", "0#ES:", option key)
        component_id: String,
        /// Exposure: weight (0.0-1.0) or quantity (number of units)
        /// Depends on CompositionRepresentation type
        exposure: f64,
        /// Current price of the asset
        price: f64,
    },
    /// Nested index component
    Index {
        /// Nested index identifier
        component_id: String,
        /// Exposure: weight (0.0-1.0) or quantity (number of index units)
        /// Depends on CompositionRepresentation type
        exposure: f64,
        /// Nested index composition (recursive structure)
        composition: Box<IndexComposition>,
    },
}
```

**Key Features**:
- **Recursive**: Supports nested indices (index containing other indices)
- **Flexible**: Exposure can be weight or quantity depending on representation
- **Price Tracking**: Includes current price for mark-to-market

**Example (Weights-based)**:
```rust
IndexComponent::Asset {
    component_id: "EEM.P".to_string(),
    exposure: 0.15,  // 15% weight
    price: 105.23,
}
```

**Example (Quantities-based)**:
```rust
IndexComponent::Asset {
    component_id: "0#ES:".to_string(),
    exposure: 10.5,  // 10.5 futures contracts
    price: 4500.0,
    divisor: 1.0,
}
```

**Example (Nested Index)**:
```rust
IndexComponent::Index {
    component_id: "sub_index_1".to_string(),
    exposure: 0.30,  // 30% weight in nested index
    composition: Box::new(IndexComposition {
        level: 102.5,
        representation: CompositionRepresentation::Weights {
            components: vec![
                IndexComponent::Asset {
                    component_id: "AAPL".to_string(),
                    exposure: 0.5,
                    price: 150.0,
                },
                // ... more components
            ],
        },
        timestamp: DateTime::from_utc(...),
    }),
}
```

---

## Conversion Functions

### `weights_to_quantities(composition: &IndexComposition) -> IndexComposition`

**Purpose**: Convert weights-based composition to quantities-based

**Formula**:
- For assets: `q_i = (w_i × Level) / P_i`
- For nested indices: `q_i = (w_i × Level) / nested_index_level` (nested composition unchanged)
- `Divisor = 1.0`

**Key Point**: Conversion does NOT recurse into nested indices - only converts current level.

**Implementation**:
```rust
pub fn weights_to_quantities(composition: &IndexComposition) -> IndexComposition {
    match &composition.representation {
        CompositionRepresentation::Weights { components } => {
            let quantities: Vec<IndexComponent> = components
                .iter()
                .map(|comp| match comp {
                    IndexComponent::Asset { component_id, exposure, price } => {
                        let quantity = (exposure * composition.level) / price;
                        IndexComponent::Asset {
                            component_id: component_id.clone(),
                            exposure: quantity,
                            price: *price,
                        }
                    }
                    IndexComponent::Index { component_id, exposure, composition: nested } => {
                        let quantity = (exposure * composition.level) / nested.level;
                        IndexComponent::Index {
                            component_id: component_id.clone(),
                            exposure: quantity,
                            composition: nested.clone(),
                        }
                    }
                })
                .collect();
            
            IndexComposition {
                level: composition.level,
                representation: CompositionRepresentation::Quantities {
                    components: quantities,
                    divisor: 1.0,
                },
                timestamp: composition.timestamp,
            }
        }
        _ => composition.clone(), // Already quantities
    }
}
```

---

### `quantities_to_weights(composition: &IndexComposition) -> IndexComposition`

**Purpose**: Convert quantities-based composition to weights-based

**Formula**:
- For assets: `Level = (Σ q_i × P_i) / Divisor`, `w_i = (q_i × P_i) / Level`
- For nested indices: `w_i = (q_i × nested_index_level) / Level` (nested composition unchanged)

**Key Point**: Conversion does NOT recurse into nested indices - only converts current level.

**Implementation**:
```rust
pub fn quantities_to_weights(composition: &IndexComposition) -> IndexComposition {
    match &composition.representation {
        CompositionRepresentation::Quantities { components, divisor } => {
            // Calculate level from quantities
            let level = components
                .iter()
                .map(|comp| match comp {
                    IndexComponent::Asset { exposure, price, .. } => exposure * price,
                    IndexComponent::Index { exposure, composition: nested, .. } => {
                        exposure * nested.level
                    }
                })
                .sum::<f64>() / divisor;
            
            // Convert to weights
            let weights: Vec<IndexComponent> = components
                .iter()
                .map(|comp| match comp {
                    IndexComponent::Asset { component_id, exposure, price } => {
                        let weight = (exposure * price) / level;
                        IndexComponent::Asset {
                            component_id: component_id.clone(),
                            exposure: weight,
                            price: *price,
                        }
                    }
                    IndexComponent::Index { component_id, exposure, composition: nested } => {
                        let weight = (exposure * nested.level) / level;
                        IndexComponent::Index {
                            component_id: component_id.clone(),
                            exposure: weight,
                            composition: nested.clone(),
                        }
                    }
                })
                .collect();
            
            IndexComposition {
                level,
                representation: CompositionRepresentation::Weights {
                    components: weights,
                },
                timestamp: composition.timestamp,
            }
        }
        _ => composition.clone(), // Already weights
    }
}
```

---

### `flatten(&self) -> IndexComposition`

**Purpose**: Flatten nested index composition to flat asset-based composition

**Algorithm**:
1. Recursively resolve all nested indices to underlying assets
2. Multiply weights/quantities through the hierarchy
3. Aggregate exposures for assets that appear in multiple nested indices
4. Return flat composition with only `IndexComponent::Asset` entries

**Implementation**:
```rust
impl IndexComposition {
    pub fn flatten(&self) -> IndexComposition {
        let mut asset_map: HashMap<String, (f64, f64)> = HashMap::new(); // (exposure, price)
        
        self.flatten_recursive(1.0, &mut asset_map);
        
        let components: Vec<IndexComponent> = asset_map
            .into_iter()
            .map(|(component_id, (exposure, price))| {
                IndexComponent::Asset {
                    component_id,
                    exposure,
                    price,
                }
            })
            .collect();
        
        IndexComposition {
            level: self.level,
            representation: match &self.representation {
                CompositionRepresentation::Weights { .. } => {
                    CompositionRepresentation::Weights { components }
                }
                CompositionRepresentation::Quantities { divisor, .. } => {
                    CompositionRepresentation::Quantities {
                        components,
                        divisor: *divisor,
                    }
                }
            },
            timestamp: self.timestamp,
        }
    }
    
    fn flatten_recursive(&self, multiplier: f64, asset_map: &mut HashMap<String, (f64, f64)>) {
        match &self.representation {
            CompositionRepresentation::Weights { components } |
            CompositionRepresentation::Quantities { components, .. } => {
                for comp in components {
                    match comp {
                        IndexComponent::Asset { component_id, exposure, price } => {
                            let total_exposure = exposure * multiplier;
                            asset_map
                                .entry(component_id.clone())
                                .and_modify(|(exp, _)| *exp += total_exposure)
                                .or_insert((total_exposure, *price));
                        }
                        IndexComponent::Index { exposure, composition, .. } => {
                            let nested_multiplier = multiplier * exposure;
                            composition.flatten_recursive(nested_multiplier, asset_map);
                        }
                    }
                }
            }
        }
    }
}
```

---

## Option Data Structures

### `OptionDefinition`

**Purpose**: Option asset definition (separate from position)

**Location**: `src/analytics/options.rs`

```rust
use chrono::NaiveDate;
use crate::asset_key::AssetKey;

/// Option asset definition
pub struct OptionDefinition {
    /// Unique asset key
    pub key: AssetKey,
    /// Underlying asset identifier
    pub underlying: String,
    /// Expiry date of underlying (for variance swaps)
    pub expiry_underlying: Option<NaiveDate>,
    /// Strike price
    pub strike: f64,
    /// Expiration date
    pub expiration: NaiveDate,
    /// Option type (Call or Put)
    pub option_type: OptionType,
    /// Exercise style (American or European)
    pub exercise_style: ExerciseStyle,
}
```

**Key Point**: This is the asset definition only - position (units) is separate.

---

### `OptionPosition`

**Purpose**: Option position (option definition + units)

```rust
/// Option position (option definition + units)
pub struct OptionPosition {
    /// Option definition
    pub option: OptionDefinition,
    /// Number of units (can be fractional)
    pub units: f64,
}
```

**Key Point**: Units are always 1 for a single option, but can be different for portfolio positions.

---

### `OptionPricing`

**Purpose**: Option pricing result (premium + all Greeks)

```rust
/// Option pricing result
pub struct OptionPricing {
    /// Option premium
    pub premium: f64,
    /// Delta (price sensitivity to underlying)
    pub delta: f64,
    /// Gamma (delta sensitivity)
    pub gamma: f64,
    /// Vega (volatility sensitivity)
    pub vega: f64,
    /// Theta (time decay)
    pub theta: f64,
    /// Rho (interest rate sensitivity)
    pub rho: f64,
}
```

**Usage**: Returned by `option_pricing()` calculator function.

---

### `ContinuingOptionPortfolio`

**Purpose**: Option portfolio tracking structure

```rust
/// Continuing option portfolio
pub struct ContinuingOptionPortfolio {
    /// Current option positions
    pub positions: Vec<OptionPosition>,
    /// Portfolio mark-to-market value
    pub mark_to_market: f64,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
}
```

**Usage**: Maintained by `PortfolioMarkToMarketDefinition`.

---

## Registry Structures

### `AnalyticRegistry`

**Purpose**: Main registry for generic, reusable components

**Location**: `src/analytics/registry.rs`

**Contains**:
- All generic calculators
- All generic containers
- All generic executors
- All generic definitions

**Registration**:
- Registered at application startup
- Available to all indices
- Examples: `WeightedSum`, `ExcessReturnDefinition`, `VolatilityDefinition`

---

### `IndexNodeRegistry`

**Purpose**: Index-specific registry (factory-generated from rulebook)

**Location**: Factory-generated

**Contains**:
- Index-specific nodes
- Index-specific DAG structure
- Rulebook-derived configuration

**Registration**:
- Created per index instance
- Generated from rulebook configuration
- Examples: `VarianceStrikeCalculator`, `VarianceReplicationPortfolioBuilder`

---

## Persistence Structures

### Checkpoint Storage Format

**Database Table**: `analytics`

**Schema**:
```sql
CREATE TABLE analytics (
    asset_key TEXT NOT NULL,
    date TEXT NOT NULL,
    analytics_name TEXT NOT NULL,
    value TEXT NOT NULL,  -- JSON blob
    PRIMARY KEY (asset_key, date, analytics_name)
);
```

**Storage Pattern**:
```json
{
  "level": 100.5,
  "base_index": 100.3,
  "component_levels": {
    "EEM.P": 105.2,
    "0#ES:": 102.1
  },
  "composition": {
    "representation": "weights",
    "components": [
      {
        "type": "asset",
        "component_id": "EEM.P",
        "exposure": 0.15,
        "price": 105.23
      },
      {
        "type": "asset",
        "component_id": "0#ES:",
        "exposure": 0.25,
        "price": 4500.0
      }
    ]
  }
}
```

**Key Design Decisions**:
- **`asset_key`**: Index identifier (e.g., `'solstae_index'`)
- **`analytics_name`**: Type of data (`'index_level'`, `'index_composition'`, `'index_checkpoint'`)
- **`value`**: JSON blob (flexible, no schema changes needed)

---

## Data Provider Extensions

### Extended `DataProvider` Trait

**Purpose**: Support for all asset types and reference data

**Location**: `src/time_series.rs`

**Extensions**:
```rust
pub trait DataProvider {
    // Existing methods
    fn get_time_series(&self, asset_key: &AssetKey, date_range: &DateRange) -> Result<Vec<TimeSeriesPoint>, DataProviderError>;
    fn available_dates(&self, asset_key: &AssetKey, date_range: &DateRange) -> Result<Vec<DateTime<Utc>>, DataProviderError>;
    
    // New methods for index calculation
    fn get_etf_data(&self, ticker: &str, date_range: &DateRange) -> Result<Vec<ETFDataPoint>, DataProviderError>;
    fn get_futures_data(&self, series: &str, contract: &str, date_range: &DateRange) -> Result<Vec<FuturesDataPoint>, DataProviderError>;
    fn get_option_data(&self, option_key: &AssetKey, date_range: &DateRange) -> Result<Vec<OptionDataPoint>, DataProviderError>;
    fn get_fx_rate(&self, from: &str, to: &str, date: NaiveDate) -> Result<f64, DataProviderError>;
    fn get_reference_rate(&self, ric: &str, date: NaiveDate) -> Result<f64, DataProviderError>;
    fn get_target_weights(&self, index_id: &str, date: NaiveDate) -> Result<HashMap<String, f64>, DataProviderError>;
}
```

---

## Summary

### Core Structures
- **`IndexComposition`**: Main composition output structure
- **`CompositionRepresentation`**: Weights vs quantities enum
- **`IndexComponent`**: Asset or nested index component

### Option Structures
- **`OptionDefinition`**: Option asset definition
- **`OptionPosition`**: Option position (definition + units)
- **`OptionPricing`**: Pricing result (premium + Greeks)
- **`ContinuingOptionPortfolio`**: Portfolio tracking

### Registry Structures
- **`AnalyticRegistry`**: Generic components registry
- **`IndexNodeRegistry`**: Index-specific registry

### Persistence Structures
- **Checkpoint JSON**: Flexible JSON storage format
- **Database Schema**: Single `analytics` table for all indices

### Data Provider Structures
- **Extended `DataProvider`**: Support for all asset types and reference data

