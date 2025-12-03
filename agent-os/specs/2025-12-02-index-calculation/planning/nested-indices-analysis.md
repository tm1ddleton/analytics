# Nested Indices and Flattening Analysis

## Overview

Index components can themselves be indices, creating nested index structures. The composition representation must support this nesting, and provide a flattening function to collapse nested indices to a flat asset-based representation.

## Key Requirements

1. **Nested Index Support**: Index components can be either assets or other indices
2. **Risk Representation**: Quantities accurately represent the risk of the position (rationale for offsetting cash in FX-hedged)
3. **Flattening**: Ability to collapse nested index composition to flat asset-based composition

## Generalized Index Composition Structure

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
        weight: f64,      // For weights representation
        quantity: f64,    // For quantities representation
        price: f64,
    },
    /// Nested index component
    Index {
        component_id: String,
        weight: f64,      // For weights representation
        quantity: f64,    // For quantities representation
        composition: Box<IndexComposition>,  // Recursive - can contain nested indices
    },
}
```

**Alternative Design** (Single weight/quantity field):
```rust
pub enum IndexComponent {
    Asset {
        component_id: String,
        exposure: f64,   // weight or quantity depending on representation
        price: f64,
    },
    Index {
        component_id: String,
        exposure: f64,   // weight or quantity depending on representation
        composition: Box<IndexComposition>,
    },
}
```

## Flattening Algorithm

### Weights-Based Flattening

**Input**: Nested index composition (weights representation)
**Output**: Flat index composition (weights representation, only asset components)

**Algorithm**:
1. Initialize empty flat composition: `HashMap<component_id, weight>`
2. For each component in the composition:
   - If `Asset`: Add `(component_id, weight)` to flat composition
   - If `Index`: 
     a. Recursively flatten the nested index
     b. For each asset in flattened nested index: Add `(asset_id, parent_weight × nested_weight)` to flat composition
3. Aggregate weights for assets that appear multiple times
4. Convert to flat composition structure

**Example**:
```
Index A (Level: 100, Weights):
  - Asset X: weight = 0.5, price = 100
  - Index B: weight = 0.5, composition:
      Level: 200, Weights:
        - Asset Y: weight = 0.6, price = 50
        - Asset Z: weight = 0.4, price = 200

Flattened Index A:
  - Asset X: weight = 0.5, price = 100
  - Asset Y: weight = 0.5 × 0.6 = 0.3, price = 50
  - Asset Z: weight = 0.5 × 0.4 = 0.2, price = 200
```

### Quantities-Based Flattening

**Input**: Nested index composition (quantities representation)
**Output**: Flat index composition (quantities representation, only asset components)

**Algorithm**:
1. Initialize empty flat composition: `HashMap<component_id, quantity>`
2. For each component in the composition:
   - If `Asset`: Add `(component_id, quantity)` to flat composition
   - If `Index`:
     a. Recursively flatten the nested index
     b. For each asset in flattened nested index: Add `(asset_id, parent_quantity × nested_quantity)` to flat composition
3. Aggregate quantities for assets that appear multiple times
4. Convert to flat composition structure

**Example**:
```
Index A (Level: 100, Quantities, Divisor: 1):
  - Asset X: quantity = 0.5, price = 100
  - Index B: quantity = 0.5, composition:
      Level: 200, Quantities, Divisor: 1:
        - Asset Y: quantity = 2.4, price = 50
        - Asset Z: quantity = 0.4, price = 200

Flattened Index A:
  - Asset X: quantity = 0.5, price = 100
  - Asset Y: quantity = 0.5 × 2.4 = 1.2, price = 50
  - Asset Z: quantity = 0.5 × 0.4 = 0.2, price = 200
```

### Risk Representation in Quantities

**Key Insight**: Quantities accurately represent the risk of the position.

**FX-Hedged Index Example**:
```
FX-Hedged Index (Quantities):
  - USD Index: quantity = 1.0, composition: {...}
  - GBP Cash (offsetting): quantity = -1.0, price = 1.0 (no interest)

Flattened:
  - All underlying assets from USD Index (with quantities)
  - GBP Cash: quantity = -1.0 (explicitly shows the hedge)
```

The offsetting cash asset is explicitly included in the quantities representation, accurately modeling the risk.

## Implementation

### Flatten Function

```rust
impl IndexComposition {
    /// Flatten nested index composition to flat asset-based composition
    /// 
    /// Recursively resolves all nested indices to their underlying assets,
    /// multiplying weights/quantities through the hierarchy.
    pub fn flatten(&self) -> IndexComposition {
        match &self.representation {
            CompositionRepresentation::Weights { components } => {
                self.flatten_weights(components)
            }
            CompositionRepresentation::Quantities { components, divisor } => {
                self.flatten_quantities(components, *divisor)
            }
        }
    }
    
    fn flatten_weights(&self, components: &[IndexComponent]) -> IndexComposition {
        let mut flat_weights: HashMap<String, f64> = HashMap::new();
        let mut flat_prices: HashMap<String, f64> = HashMap::new();
        let mut component_ids: Vec<String> = Vec::new();
        
        for component in components {
            match component {
                IndexComponent::Asset { component_id, weight, price } => {
                    *flat_weights.entry(component_id.clone()).or_insert(0.0) += weight;
                    flat_prices.insert(component_id.clone(), *price);
                    if !component_ids.contains(component_id) {
                        component_ids.push(component_id.clone());
                    }
                }
                IndexComponent::Index { component_id: _idx_id, weight: parent_weight, composition } => {
                    // Recursively flatten nested index
                    let flat_nested = composition.flatten();
                    
                    match flat_nested.representation {
                        CompositionRepresentation::Weights { components: nested_components } => {
                            for nested_comp in nested_components {
                                match nested_comp {
                                    IndexComponent::Asset { component_id, weight: nested_weight, price } => {
                                        *flat_weights.entry(component_id.clone()).or_insert(0.0) += 
                                            parent_weight * nested_weight;
                                        flat_prices.insert(component_id.clone(), *price);
                                        if !component_ids.contains(component_id) {
                                            component_ids.push(component_id.clone());
                                        }
                                    }
                                    IndexComponent::Index { .. } => {
                                        // Should not happen after flattening, but handle gracefully
                                        // Could recursively flatten again if needed
                                    }
                                }
                            }
                        }
                        CompositionRepresentation::Quantities { .. } => {
                            // Convert quantities to weights for flattening
                            // Level = (Σ q_i × P_i) / Divisor
                            // w_i = (q_i × P_i) / Level
                            // Then multiply by parent_weight
                        }
                    }
                }
            }
        }
        
        // Convert to IndexComponent vector
        component_ids.sort();
        let flat_components: Vec<IndexComponent> = component_ids.iter().map(|id| {
            IndexComponent::Asset {
                component_id: id.clone(),
                weight: flat_weights[id],
                price: flat_prices[id],
            }
        }).collect();
        
        IndexComposition {
            level: self.level,
            representation: CompositionRepresentation::Weights { 
                components: flat_components 
            },
            timestamp: self.timestamp,
        }
    }
    
    fn flatten_quantities(&self, components: &[IndexComponent], divisor: f64) -> IndexComposition {
        // Similar logic for quantities
        // ...
    }
}
```

### Cross-Representation Flattening

**Handling Mixed Representations**:
- Flattening can handle mixed representations (weights and quantities in same hierarchy)
- For nested index components:
  - If nested index is in different representation, convert it first (using conversion node)
  - Or flatten preserves the nested representation and converts during aggregation
- **Simpler approach**: Flatten works on the representation it receives - if nested indices are in different representation, they're converted during flattening aggregation

**Flattening Algorithm** (handles mixed representations):
1. For each component:
   - If `Asset`: Add to flat composition directly
   - If `Index`: 
     a. Flatten nested index (recursive call) - returns flat composition
     b. If nested flattened result is in different representation, convert it to match parent
     c. Multiply nested asset exposures by parent exposure
     d. Add nested assets to flat composition
2. Aggregate exposures for assets appearing multiple times
3. Return flat composition in parent's representation

**Note**: Flattening is the operation that handles recursion and conversion of nested indices. The conversion functions themselves are non-recursive and only convert the current level.

## Use Cases

### 1. FX-Hedged Index
```
FX-Hedged Index:
  - USD Index (nested): weight = 1.0
  - GBP Cash (asset): weight = -1.0

Flattened:
  - All assets from USD Index (with weights)
  - GBP Cash: weight = -1.0
```

### 2. Multi-Asset Index with Sub-Indices
```
Multi-Asset Index:
  - Equity Index (nested): weight = 0.6
  - Bond Index (nested): weight = 0.4

Flattened:
  - All equities from Equity Index (weights × 0.6)
  - All bonds from Bond Index (weights × 0.4)
```

### 3. Risk Analysis
- Flattened composition shows all underlying asset exposures
- Quantities representation accurately models risk (including hedges)
- Can aggregate risk across nested indices

## Benefits

1. **Flexibility**: Support arbitrary nesting depth
2. **Risk Accuracy**: Quantities representation accurately models position risk
3. **Composability**: Build complex indices from simpler indices
4. **Analysis**: Flattening enables risk analysis and portfolio construction
5. **Transparency**: Clear view of underlying asset exposures

## Implementation Considerations

1. **Performance**: Flattening can be expensive for deeply nested indices - consider caching
2. **Circular Dependencies**: Need to detect and prevent circular index references
3. **Validation**: Ensure nested index levels are consistent with parent index
4. **Storage**: Store nested structure in database, flatten on-demand for analysis

