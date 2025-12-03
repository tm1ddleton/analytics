# Spec Initialization: Index Calculation with Rulebook Support

**Date:** 2025-12-02  
**Feature:** Index Calculation Framework with Rulebook-Driven Node Generation

## Initial Description

Implement a framework for calculating financial indices using the existing analytics DAG framework. The system should:

1. **Rulebook-Driven Calculation**: Accept a rulebook (uploaded file) that defines how an index is calculated
2. **Data Sources**: Use Yahoo Finance data when available, fall back to dummy data otherwise
3. **Target Weights**: Randomly generate daily target weights with constraints:
   - Individual component weights: -200% to +200% (i.e., -2.0 to +2.0)
   - Net exposure: -100% to +100% (i.e., -1.0 to +1.0)
4. **Node Architecture**: Generate the necessary DAG nodes (calculator, executor, container, definition) based on the rulebook
4. **Node Reusability**: Consider whether nodes are general-purpose or index-specific:
   - If general-purpose: integrate into main node registry
   - If specific: support separate node registries or factory-based node generation from configs (YAML/JSON)
5. **Hot Start Capability**: When requesting index levels and composition for a date range, the system should be able to retrieve and use previous index values from the database to avoid recalculating from scratch

## Key Components

- Rulebook parser/interpreter
- Node factory/generator for index-specific nodes
- Index calculation logic
- Database persistence for index values (hot start)
- Integration with existing DAG framework
- Data source abstraction (Yahoo Finance vs dummy data)

## Integration Points

- Existing DAG computation framework
- Existing data provider system (SQLite)
- Existing analytics architecture (calculator/container/executor/definition layers)
- Yahoo Finance data downloader

