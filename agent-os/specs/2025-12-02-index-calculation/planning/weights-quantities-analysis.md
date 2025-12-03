# Weights vs Quantities Index Representation

## Overview

An index can be represented in two equivalent ways:
1. **Weights-based**: Vector of weights, vector of prices, and a level
2. **Quantities-based**: Vector of quantities, vector of prices, and a divisor

Both representations are mathematically equivalent and can be converted between each other.

## Mathematical Relationship

### Weights Index
**Representation**:
- Weights: `w = [w_1, w_2, ..., w_n]` (sum may not equal 1 due to long/short positions)
- Prices: `P_t = [P_1,t, P_2,t, ..., P_n,t]`
- Level: `Level_t`

**Level Calculation**:
```
Level_t = Level_{t-1} × (1 + Σ w_i × (P_i,t / P_i,t-1 - 1))
```

**Composition**: `{weights: [w_1, w_2, ..., w_n], prices: [P_1,t, ..., P_n,t], level: Level_t}`

### Quantities Index
**Representation**:
- Quantities: `q = [q_1, q_2, ..., q_n]`
- Prices: `P_t = [P_1,t, P_2,t, ..., P_n,t]`
- Divisor: `Divisor` (typically 1 when converting from weights)

**Level Calculation**:
```
Level_t = (Σ q_i × P_i,t) / Divisor
```

**Composition**: `{quantities: [q_1, q_2, ..., q_n], prices: [P_1,t, ..., P_n,t], divisor: Divisor, level: Level_t}`

### Conversion Between Representations

**Weights → Quantities**:
Given weights `w`, level `Level`, and prices `P_t`:
```
q_i = (w_i × Level) / P_i,t
Divisor = 1  (when converting from weights)
```

**Quantities → Weights**:
Given quantities `q`, prices `P_t`, and divisor `Divisor`:
```
Level = (Σ q_i × P_i,t) / Divisor
w_i = (q_i × P_i,t) / Level
```

**Verification**:
- Quantities level: `Level = (Σ q_i × P_i,t) / Divisor`
- Substituting `q_i = (w_i × Level) / P_i,t`:
  ```
  Level = (Σ (w_i × Level / P_i,t) × P_i,t) / Divisor
       = (Σ w_i × Level) / Divisor
       = Level × (Σ w_i) / Divisor
  ```
- For this to hold: `Divisor = Σ w_i` (which may not equal 1 for long/short indices)

**Actually, let me reconsider...**

If we have weights and want to convert to quantities:
- The weights represent the **notional exposure** relative to the index level
- `w_i × Level` = notional exposure to component i
- `q_i = (w_i × Level) / P_i,t` = quantity of component i

For the level calculation:
```
Level = (Σ q_i × P_i,t) / Divisor
     = (Σ (w_i × Level / P_i,t) × P_i,t) / Divisor
     = (Σ w_i × Level) / Divisor
```

So: `Divisor = Σ w_i` for the quantities representation to match the weights representation.

**However**, the user said "if converting from weights the divisor is 1". This suggests:
- When converting from weights, we set `Divisor = 1`
- Then `q_i = w_i × Level / P_i,t`
- And `Level = Σ q_i × P_i,t` (since Divisor = 1)

This means the quantities representation uses `Divisor = 1` as a convention when converting from weights, even though mathematically `Divisor = Σ w_i` would be more consistent.

## Implementation Implications

### Node Output Generalization

**Current**: Nodes output single `f64` values (returns, levels)

**Required**: Nodes need to output structs containing:
- Level: `f64`
- Composition: Either weights or quantities (based on config)
  - Weights: `Vec<f64>` (weights vector)
  - Quantities: `Vec<f64>` (quantities vector) + `Divisor: f64`

**Proposed Structure** (Generalized for Nested Indices):
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
    /// Direct asset component
    Asset {
        component_id: String,
        weight: f64,  // or quantity
        price: f64,
    },
    /// Nested index component
    Index {
        component_id: String,
        weight: f64,  // or quantity
        composition: Box<IndexComposition>,  // Recursive - can contain nested indices
    },
}
```

**Key Points**:
- Components can be either assets or nested indices
- Quantities representation accurately models risk (e.g., offsetting cash in FX-hedged)
- Supports arbitrary nesting depth

### Node Architecture Updates

#### 1. BaseIndex Node
**Current**: Outputs base index level (f64)

**Updated**: Outputs `IndexComposition` with:
- Level: Base index level
- Composition: Weights-based (from target weights) or Quantities-based (converted from weights)

**Conversion Logic**:
- If config says "weights": Output weights + prices
- If config says "quantities": Convert weights → quantities, output quantities + prices + divisor

#### 2. IndexLevel Node
**Current**: Outputs final index level (f64)

**Updated**: Outputs `IndexComposition` with:
- Level: Final index level
- Composition: Inherited from BaseIndex (weights or quantities, based on config)

#### 3. Conversion Nodes

**WeightsToQuantities Node**:
- Input: Weights-based composition
- Output: Quantities-based composition
- Formula: `q_i = (w_i × Level) / P_i,t`, `Divisor = 1`

**QuantitiesToWeights Node**:
- Input: Quantities-based composition
- Output: Weights-based composition
- Formula: `w_i = (q_i × P_i,t) / Level`

### Configuration

**Rulebook Config Extension**:
```yaml
index:
  # ... existing config ...
  composition_output: "weights" | "quantities"  # Default: "weights"
```

### Storage

**Database Schema**:
```sql
-- Index composition storage (extend existing analytics table)
-- Value column stores JSON:
{
  "level": 100.5,
  "representation": "weights",  # or "quantities"
  "weights": [0.1, 0.2, -0.15, ...],  # if weights
  "quantities": [10.5, 20.3, -15.2, ...],  # if quantities
  "prices": [100.0, 50.0, 200.0, ...],
  "divisor": 1.0  # if quantities
}
```

## Benefits

1. **Flexibility**: Support both weights-based and quantities-based indices
2. **Conversion**: Easy conversion between representations
3. **Composability**: Quantities representation useful for portfolio construction
4. **Clarity**: Explicit representation matches user's mental model

## Flattening Nested Indices

**Purpose**: Collapse nested index composition to a flat representation based on underlying assets

**Algorithm**:
1. For each component in the composition:
   - If `Asset`: Add to flat composition directly
   - If `Index`: Recursively flatten the nested index, then multiply weights/quantities by parent weight/quantity
2. Aggregate weights/quantities for assets that appear in multiple nested indices
3. Return flat composition with only asset components

**Example**:
```
Index A (weights):
  - Asset X: 0.5
  - Index B: 0.5
    - Asset Y: 0.6
    - Asset Z: 0.4

Flattened Index A:
  - Asset X: 0.5
  - Asset Y: 0.5 × 0.6 = 0.3
  - Asset Z: 0.5 × 0.4 = 0.2
```

**Implementation**:
```rust
impl IndexComposition {
    /// Flatten nested index composition to flat asset-based composition
    pub fn flatten(&self) -> IndexComposition {
        match &self.representation {
            CompositionRepresentation::Weights { components } => {
                let mut flat_components: HashMap<String, f64> = HashMap::new();
                let mut flat_prices: HashMap<String, f64> = HashMap::new();
                
                for component in components {
                    match component {
                        IndexComponent::Asset { component_id, weight, price } => {
                            *flat_components.entry(component_id.clone()).or_insert(0.0) += weight;
                            flat_prices.insert(component_id.clone(), *price);
                        }
                        IndexComponent::Index { component_id, weight, composition } => {
                            // Recursively flatten nested index
                            let flat_nested = composition.flatten();
                            match flat_nested.representation {
                                CompositionRepresentation::Weights { components: nested_components } => {
                                    for nested_comp in nested_components {
                                        match nested_comp {
                                            IndexComponent::Asset { component_id: nested_id, weight: nested_weight, price: nested_price } => {
                                                *flat_components.entry(nested_id.clone()).or_insert(0.0) += weight * nested_weight;
                                                flat_prices.insert(nested_id.clone(), nested_price);
                                            }
                                            IndexComponent::Index { .. } => {
                                                // Should not happen after flattening, but handle gracefully
                                            }
                                        }
                                    }
                                }
                                _ => {} // Handle quantities case if needed
                            }
                        }
                    }
                }
                
                // Convert HashMap to Vec
                let mut component_ids: Vec<String> = flat_components.keys().cloned().collect();
                component_ids.sort();
                let components: Vec<IndexComponent> = component_ids.iter().map(|id| {
                    IndexComponent::Asset {
                        component_id: id.clone(),
                        weight: flat_components[id],
                        price: flat_prices[id],
                    }
                }).collect();
                
                IndexComposition {
                    level: self.level,
                    representation: CompositionRepresentation::Weights { components },
                    timestamp: self.timestamp,
                }
            }
            CompositionRepresentation::Quantities { components, divisor } => {
                // Similar logic for quantities
                // ...
            }
        }
    }
}
```

**Risk Representation**:
- Quantities accurately represent the risk of the position
- For FX-hedged index: The offsetting cash asset is explicitly included in the composition
- Flattening preserves risk: nested index quantities are multiplied by parent quantities

## Implementation Plan

1. **Generalize Node Outputs**: Extend `NodeOutput` to support structs (not just `f64`)
2. **Create Composition Types**: Define `IndexComposition`, `CompositionRepresentation`, and `IndexComponent` (supporting nested indices)
3. **Implement Flatten Function**: Recursively flatten nested index compositions to flat asset-based compositions
4. **Update BaseIndex Node**: Output composition (weights or quantities based on config, can contain nested indices)
5. **Create Conversion Nodes**: WeightsToQuantities, QuantitiesToWeights (handle nested indices)
6. **Update IndexLevel Node**: Inherit and pass through composition (preserve nesting)
7. **Update Storage**: Store composition as JSON in database (supports nested structure)
8. **Update API**: Return composition in API responses (with optional flattening parameter)

