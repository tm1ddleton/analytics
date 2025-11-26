# Spec Requirements: dag-node-design

## Initial Description
Node design refactor: align node keying with salsa-style interned keys, ensure push/pull DAG nodes share same key representation without forcing identical node structures. Focus on node identifiers, node metadata, and compatibility for push-mode replay while keeping the DAG semantics consistent. No further instructions yet.

## Requirements Discussion

### First Round Questions

**Q1:** I assume the new node design still represents each analytic/computation as a node with dependencies, but you want the identity/keying to shift toward a salsa-style interned key (unique, immutable identifier). Is that accurate, or should we keep the current string-based IDs and layer salsa-style keys only where we cache results?
**Answer:** yes, consider the approach from salsa—use the new keying approach.

**Q2:** I'm thinking we can unify how push-mode and pull-mode reference nodes by sharing a single `NodeKey` struct that holds the analytic type, parameters, and asset list. Should that replace the current `NodeId`/`NodeParams` pairing, or do you prefer to keep them separate but derive the same hash from their metadata?
**Answer:** interested to see both (registry vs metadata).

**Q3:** I assume each node should retain metadata about its analytic type (returns/volatility/data provider) in a lightweight enum rather than raw strings, so we can avoid the lowercase/uppercase mismatch we patched. Do you want this enum to be the single source of truth for both execution modes?
**Answer:** overrides should be keyed differently.

**Q4:** [Not answered yet]
**Answer:** [pending]

**Q5:** [Not answered yet]
**Answer:** [pending]

**Q6:** [Not answered yet]
**Answer:** [pending]

**Q7:** [Not answered yet]
**Answer:** [pending]

**Q8:** [Not answered yet]
**Answer:** [pending]

### Existing Code to Reference
No similar existing features identified for reference.

### Follow-up Questions
No follow-up questions issued.

## Visual Assets

### Files Provided:
No visual assets provided.

### Visual Insights:
- No visuals were supplied (folder empty).

## Requirements Summary

### Functional Requirements
- Design analytics nodes as stateless primitives (average, standard deviation, exponential smoothing) paired with windowing traits that feed the analytic function the correct inputs.
- Ensure the analytics tree can self-construct: requesting rolling volatility should automatically resolve dependencies (returns → prices) and extend date ranges using window lag metadata.
- Provide an override mechanism where analytics keys reflect non-default inputs, allowing behavior such as arithmetic returns.

### Reusability Opportunities
- None identified.

### Scope Boundaries
**In Scope:**
- Redesign node identifiers/keys to align with salsa-style interning.
- Keep analytic functions stateless and compose them with windowing wrappers.
- Ensure push-mode and pull-mode nodes agree on key metadata so caching and replay reuse the same definitions.

**Out of Scope:**
- REST API or frontend changes unrelated to node metadata.
- Comprehensive error handling improvements at this stage.

### Technical Considerations
- Node keys must encode analytic type, parameters, asset/date ranges, and optional overrides so different inputs produce distinct keys.
- Push-mode replay should consume the same node metadata as pull-mode for deterministic DAG construction.
- Consider introducing a registry or DSL for defining analytics so DAG dependencies can be inferred automatically.
