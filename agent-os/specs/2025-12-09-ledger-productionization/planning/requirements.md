# Spec Requirements: Quantitative Event Engine

## Initial Description

**UPDATED VISION:** This spec has evolved from "Ledger Productionization" to a unified **Quantitative Event Engine** - a general-purpose event processing system with the Ledger as the central abstraction.

The original scope was productionization of the Python POC in the Ledger/ directory into Rust, and integration with a complex event framework that will deal with all lifecycle operations in an equity derivatives trading operation. This includes:
- Overall architecture design
- Separation of concerns
- Stubbing other services that are not part of this repo

**Key Vision Shift:** The Ledger is now understood as a general-purpose abstraction that can serve multiple domains:
- Position Ledger (products and hedges)
- Index Ledger (constituents)
- These can be decomposed/flattened for unified risk views

## Requirements Discussion

### First Round Questions

**Q1:** Scope of "Productionization" - Does this mean porting the existing Python Ledger logic to Rust while maintaining the same accounting model, or are there known architectural changes needed?
**Answer:** Port to Rust maintaining the same accounting model. The Python POC validates the domain model; Rust provides production-grade performance.

**Q2:** What is the "complex event framework" - Is this an existing system we need to integrate with, or something to be designed as part of this spec?
**Answer:** The Event Framework is OUT OF SCOPE for this spec. It will be stubbed. This library needs to define the interface/contract it expects from the Event Framework.

**Q3:** Product Types Support - The existing Ledger handles 11 product types. Should the Rust version support all 11 from the start, or a subset?
**Answer:** All product types should be supported. The product type definitions should be SHARED between the Ledger and the existing Index calculation framework (which already has Rust product definitions).

**Q4:** Performance Requirements - What scale are we targeting? (positions, transactions per second, latency requirements)
**Answer:** Performance is important but not the primary driver. Correctness and maintainability come first. Rust chosen for type safety and eventual performance, not immediate optimization.

**Q5:** Integration Points - Beyond the Event Framework, what other systems does the Ledger need to interact with? (Market data, reference data, reporting)
**Answer:** Market data service will be stubbed. The Ledger produces moves that could feed into reporting, but reporting is out of scope. Focus is on the core Ledger + Event Framework integration.

**Q6:** Persistence Strategy - Should the Rust Ledger persist state to a database, or is it in-memory with event sourcing?
**Answer:** Out of scope for initial implementation. Focus on the core logic. Persistence strategy will be addressed in a future spec.

**Q7:** What is explicitly OUT of scope or should be deferred to future work?
**Answer:** Out of scope: Event Framework implementation, Market data service, Persistence/database, Reporting, UI/API layer. Focus is purely on the core Ledger domain logic and its interface contracts.

### Second Round Questions

**Q8:** Scope Confirmation - This spec covers ONLY the "Smart Contract Adapter" library, with Event Framework, Quant Lib, Market as stubs, and Ledger receiving moves?
**Answer:** Yes. This spec covers ONLY the "Smart Contract Adapter" library. Event Framework, Quant Lib, Market = stubs. Ledger = receives moves (but existing Python is just POC, will be Rust).

**Q9:** Trigger Registry Storage - Does the Event Framework maintain the registry, or does this library maintain its own internal trigger registry?
**Answer:** The triggers are registered by the event framework. As far as they go this library should be stateless. Event Framework maintains the trigger registry. This library is STATELESS - it doesn't store triggers or positions. Pure function-like behavior: receives inputs, produces outputs.

**Q10:** Initial Implementation Scope - Start narrow with ONE product type, or implement full trigger taxonomy from start?
**Answer:** Thin slice with one product type, but in the implementation plan extend to all products. Start with ONE product type to validate architecture. Implementation plan should include roadmap to extend to all product and asset types.

**UPDATED (Q20):** Thin slice changed from Reverse Convertible to **continuous barrier products** (Knock-Out Warrant, Mini Certificate) because the continuous barrier trigger logic already exists in the event framework.

**Q11:** Rust-Python Integration Method - PyO3 bindings, JSON serialization, or something else?
**Answer:** Forget all Python. The Ledger implementation is just a POC, this library should be entirely Rust. 100% Rust - no Python integration. The existing Python Ledger is just a POC/reference. This library AND the Ledger will be Rust. No PyO3 bindings needed.

### Clarification: Products vs Assets Distinction

**Q12:** What is the complete list of instrument types the library needs to support?
**Answer:** There is an important distinction between Products and Assets:

**Products** = Structured products sold to clients (14 types):
1. Barrier Reverse Convertible
2. Barrier Reverse Convertible Pro
3. Bonus Certificate
4. Capped Bonus Certificate
5. Capped Bonus Pro Certificate
6. Capped Warrant
7. Discount Certificate
8. Knock-Out Warrant
9. Reverse Capped Bonus Certificate
10. Reverse Convertible
11. Mini Certificate
12. Open End Turbo
13. Factor Certificate
14. Vanilla Options

**Assets** = Tradeable instruments used for hedging (10 types, plus products can also be traded as assets):
1. Equities
2. Bonds
3. Futures
4. Barrier Option
5. Interest Rate Futures
6. Interest Rate Options
7. Cash
8. FX Option
9. FX Swap
10. Stock Borrow Loan

The library needs to handle BOTH:
- Products for client transactions and their lifecycle events
- Assets for hedging operations

This replaces the earlier "11 instrument types" from the Python POC with this more comprehensive and correctly categorized list.

**Note on Vanilla Options:** Vanilla Options are client-facing products (not just hedging assets like Barrier Option in the Assets list). They support American exercise capability, making them the primary product requiring the American (early exercise) trigger type.

### Clarification: Complete Trigger Taxonomy

**Q13:** What are all the trigger types required for the products?
**Answer:** The complete trigger taxonomy is:

**Time-Based Triggers:**
1. **Expiry** - Product maturity/expiration date
2. **Coupon payments** - Scheduled coupon payment dates

**Price-Based Triggers (Discrete):**
3. **Barrier** - Discrete barrier observation at specific times (e.g., at market close)

**Price-Based Triggers (Continuous):**
4. **Continuous barrier monitoring** - Real-time/tick-level barrier observation throughout trading hours

**Exercise-Based Triggers:**
5. **American (early exercise)** - Early exercise rights that can be triggered anytime during the exercise window (CONFIRMED IN SCOPE)

**Corporate Action Triggers:**
6. **Dividends** - Corporate action dividend events affecting underlying
7. **Stock splits** - Corporate action stock split events affecting underlying

**Note on American Exercise:** This trigger type is CONFIRMED IN SCOPE. Vanilla Options are the primary product requiring American exercise capability, providing a clear production use case for this trigger type.

### Clarification: Fixing Trigger Type

**Q14:** Is there a trigger type for scheduled price observations (distinct from barrier triggers)?
**Answer:** Yes - **Fixing** is a critical trigger type that was missing from the initial taxonomy.

A **Fixing** is a "price at time" trigger - the observation of a price at a specific scheduled time. This is fundamentally different from barrier triggers:

- **Barrier**: Checks if price has breached a level (binary yes/no outcome)
- **Fixing**: Records the actual price at a specific moment (captures the price value itself)

**Fixing subtypes:**
- **Initial fixing** - Sets strike/reference price at trade start
- **Final fixing** - Determines settlement amount at expiry
- **Periodic fixings** - For averaging (Asian-style features), swap resets
- **Reset fixings** - For floating rate resets, variable coupon calculations

**Fixings determine:**
- Strike prices
- Settlement amounts
- Coupon amounts (for variable coupons)
- Barrier levels (for relative barriers)

This is the 8th trigger type in the taxonomy.

### Clarification: Fixing Callback Simplification

**Q15:** How does this library receive price data for fixing triggers?
**Answer:** The Event Framework passes the price directly in the callback - this library does NOT query Market for prices.

**Key Design Decision:** When a fixing trigger fires, the Event Framework:
1. Detects that the fixing time has been reached
2. Obtains the price from Market (or wherever prices come from)
3. Calls this library with: trigger info + observed price (NO positions in first call - see Q19)

This library then:
1. Receives trigger data in the callback
2. Passes the data to Quant Lib to get product-level action
3. Returns product-level action to Event Framework (see Q19 for two-call pattern)

**Architectural Implications:**
- This library has NO direct dependency on Market for fixing triggers
- Event Framework is responsible for price acquisition
- Reinforces the stateless, pure-function nature of this library
- Callback signature for fixing triggers includes the price value
- Simplifies this library's responsibilities and testing

**Note:** The original flow diagram has been superseded by the Two-Call Pattern described in Q19.

### Clarification: Vanilla Options and American Exercise

**Q16:** Is there an additional product type that uses American exercise?
**Answer:** Yes - **Vanilla Options** should be added as the 14th product type, and American exercise is CONFIRMED IN SCOPE.

**Vanilla Options characteristics:**
- Client-facing product (not just a hedging asset like Barrier Option)
- Supports **American exercise** capability (early exercise trigger)
- Standard option payoff (call/put)
- No barrier features (simpler than structured products with barriers)

**Trigger types for Vanilla Options:**
- Expiry: Yes (option maturity)
- Fixing: Yes (initial fixing for strike, final fixing for settlement)
- American (early exercise): Yes - this is the defining feature
- Dividends: Yes (affects option value and exercise decisions)
- Stock splits: Yes (adjustments required)
- Coupon: No
- Barrier: No
- Continuous barrier: No

**Significance:** Vanilla Options confirm that American exercise is a production-required trigger type, providing a clear and common use case beyond the "possible" designation on other products like Capped Warrant.

### Clarification: Callback Mechanism and Transport Agnosticism

**Q17:** How exactly do callbacks work between Event Framework and this library?
**Answer:** The callback mechanism is transport-agnostic and follows a registration-based contract pattern.

**Key Design Principles:**

1. **Transport Agnostic**: This library's runner receives messages that could arrive via JSON over REST API, Kafka, or any other transport mechanism. The core library logic does not care about transport - it processes structured data.

2. **Registration Defines the Contract**: During product setup (registration phase), this library tells the Event Framework exactly what data it needs in callbacks. The Event Framework stores both the trigger condition AND the callback payload requirements.

3. **Event Framework is Trigger-Focused and Downstream-Agnostic**: The Event Framework only cares about detecting when trigger conditions are met. It does not know or care what happens downstream when the event fires - it simply dispatches the message with the requested data.

**Registration Phase Flow:**
```
New Trade Arrives
       |
       v
This Library: Analyze product, determine required triggers
       |
       v
This Library --> Event Framework: "Register these triggers"
       |
       +-- Trigger 1: condition + callback payload schema
       +-- Trigger 2: condition + callback payload schema
       +-- ...
       |
       v
Event Framework: Stores trigger conditions + payload requirements
```

**Note:** The Execution Phase flow has been refined - see Q19 for the Two-Call Pattern.

**Separation of Concerns:**

| Component | Responsibility | Does NOT Know |
|-----------|---------------|---------------|
| Event Framework | WHEN to fire (trigger detection) | What happens downstream |
| This Library | WHAT TO DO (processing logic) | Transport mechanism |
| Runner/Transport | HOW to deliver messages | Business logic |

**Architectural Implications:**
- This library defines callback contracts (schemas for what data it needs)
- Event Framework is a "dumb dispatcher" - matches triggers to callbacks, constructs payloads
- Transport layer is completely abstracted from this library's core logic
- Clean separation: Event Framework knows WHEN, this library knows WHAT TO DO
- Enables testing without transport infrastructure (just pass structured data)
- Supports multiple deployment patterns (REST API, message queue, in-process)

**Registration Contract Structure:**
When this library registers a trigger, it specifies:
1. **Trigger Condition**: What event should cause the callback (e.g., "date = 2024-03-15" or "price crosses 100.00")
2. **Callback Payload Schema**: What data to include in the callback message

Example registration for a fixing trigger:
```
{
  "trigger_type": "fixing",
  "condition": {
    "date": "2024-03-15",
    "time": "16:00:00",
    "underlying": "AAPL"
  },
  "callback_payload": {
    "include_observed_price": true,
    "include_fixing_type": true,
    "product_ids": ["prod-123", "prod-456"]
  }
}
```

### Clarification: Portfolios (GAP in Python POC)

**Q18:** Does the Ledger need the concept of Portfolios for organizing and viewing positions?
**Answer:** Yes - **Portfolios** are a critical feature that is MISSING from the Python POC and must be added in the Rust implementation.

**GAP IDENTIFIED:** The Python Ledger POC does NOT have the notion of portfolios. This is a known gap that must be addressed in the Rust implementation.

**What Portfolios Are:**
- Ways of tagging wallets or positions within wallets
- Represent different ways of "slicing and dicing" the portfolio
- Enable multiple views/aggregations of the same underlying positions
- A "view" or "lens" over positions, NOT a separate container

**Key Characteristic - Non-Exclusive Grouping:**
- Multiple portfolios can contain the same position
- A single position can belong to many portfolios simultaneously
- Portfolios are orthogonal dimensions, not hierarchical containers

**Use Cases:**
| Portfolio Dimension | Example Views |
|--------------------|---------------|
| Strategy | "Delta-neutral hedging", "Yield enhancement", "Directional bets" |
| Desk/Trader | "Desk A", "Trader John", "London desk" |
| Client | "Client ABC positions", "Institutional clients" |
| Product type | "All BRCs", "All warrants", "Vanilla options book" |
| Underlying | "AAPL-linked positions", "Tech sector" |
| Accounting view | "Regulatory reporting", "Management P&L", "Tax book" |
| Risk category | "High delta", "Expiring this month", "Near barrier" |

**Architectural Implications:**

1. **Tagging/Labeling System:**
   - Wallets and/or positions need a flexible tagging mechanism
   - Tags are key-value pairs or hierarchical labels
   - Multiple tags per position (non-exclusive)

2. **Portfolio as Query/Filter:**
   - Portfolio is defined by a set of tag criteria
   - Portfolio membership is computed, not stored
   - Same position appears in multiple portfolio queries

3. **Ledger Query Interface:**
   - Ledger must support portfolio-filtered queries
   - "Get all positions in portfolio X"
   - "Get all moves affecting portfolio X"
   - "Aggregate P&L by portfolio"

4. **Move Generation:**
   - When moves are generated, they inherit position tags
   - Moves can be attributed to portfolios for reporting
   - P&L attribution by portfolio dimension

**Data Model Implications:**

```
Position
  +-- id
  +-- wallet_id
  +-- instrument
  +-- quantity
  +-- tags: Map<TagKey, TagValue>  <-- NEW: flexible tagging

Portfolio (conceptual, not stored)
  +-- name
  +-- filter_criteria: Vec<TagFilter>  <-- defines membership

Move
  +-- position_id
  +-- amount
  +-- inherited_tags: Map<TagKey, TagValue>  <-- for attribution
```

**Example Tag Structure:**
```
Position tags:
  - strategy: "yield_enhancement"
  - desk: "desk_a"
  - trader: "john"
  - client: "client_abc"
  - product_type: "brc"
  - underlying: "AAPL"
  - book: "trading"
  - regulatory_book: "banking_book"
```

**Portfolio Query Examples:**
```
Portfolio "Desk A BRCs":
  - filter: desk = "desk_a" AND product_type = "brc"

Portfolio "Client ABC":
  - filter: client = "client_abc"

Portfolio "Regulatory Banking Book":
  - filter: regulatory_book = "banking_book"
```

**Implementation Notes:**
- Tags should be defined at position creation time
- Tags may be inherited from wallet-level defaults
- Tag schema should be extensible (not hard-coded dimensions)
- Consider performance implications of tag-based filtering at scale
- May need indexes on common tag dimensions

### Clarification: Two-Call Pattern and Move Rules (ARCHITECTURAL REFINEMENT)

**Q19:** How does the event trigger execution pattern work in detail?
**Answer:** The event trigger pattern is a **TWO-CALL pattern** that cleanly separates product-level logic from position-level accounting/booking logic.

**KEY ARCHITECTURAL INSIGHT:** This two-call pattern separates:
- **WHAT happened** (product level, pure quant/economics)
- **HOW to book it** (position level, configurable accounting rules)

#### Two-Call Pattern Overview

```
+------------------+          +------------------+          +------------------+
|  Event Framework |          |   This Library   |          |    Quant Lib     |
+------------------+          +------------------+          +------------------+
        |                              |                              |
        |  CALL 1: Product-Level       |                              |
        |  (trigger + product data)    |                              |
        |----------------------------->|                              |
        |                              |  get_product_action()        |
        |                              |----------------------------->|
        |                              |                              |
        |                              |<-- ProductAction (qty=1) ----|
        |<-- ProductAction ------------|                              |
        |                              |                              |
        |  CALL 2: Position-Level      |                              |
        |  (ProductAction + positions  |                              |
        |   + MoveRules)               |                              |
        |----------------------------->|                              |
        |                              |                              |
        |                              |  [apply move rules]          |
        |                              |  [scale to position qty]     |
        |                              |  [generate moves]            |
        |                              |  [generate downstream actions]|
        |                              |                              |
        |<-- (Vec<Move>,               |                              |
        |     Vec<ProductAction>) -----|                              |
        |                              |                              |
        |  [send moves to Ledger]      |                              |
        |  [process downstream actions]|                              |
```

#### Call 1: Product-Level Action

**Purpose:** Determine WHAT happened at the product level (unit economics, quantity=1)

**Input from Event Framework:**
- Trigger information (type, date, price if applicable)
- Product definition/details
- NO positions involved

**Processing:**
- This library calls Quant Lib
- Quant Lib calculates the product-level action

**Output to Event Framework:**
- **ProductAction**: The economic event that occurred, expressed as a unit event (quantity=1)
- Examples:
  - "Pay $1 coupon per unit"
  - "Barrier breached, product knocked out"
  - "Settlement amount is $105 per unit"

**Key Characteristic:** This call is purely about product economics. It knows nothing about positions, entities, or accounting rules.

#### Call 2: Position-Level Application (UPDATED - Returns Moves AND ProductActions)

**Purpose:** Determine HOW to book the product-level action into ledger moves, AND identify any downstream actions that should be triggered.

**Input from Event Framework:**
- **ProductAction** (from Call 1)
- **Positions**: The actual positions affected (with quantities, entity info, tags)
- **MoveRules**: Configuration dictating how to translate the action into ledger entries

**Processing:**
- This library applies the ProductAction to each position
- Scales by position quantity
- Applies MoveRules to determine ledger entries
- Generates moves according to the applicable rules
- **Identifies any downstream ProductActions that should be triggered**

**Output to Event Framework:**
- **Vec<Move>**: The actual ledger moves to be recorded
- **Vec<ProductAction>**: Additional product-level actions that need to be processed (NEW)

**Key Insight - Chained/Recursive Events:** Call 2 can return additional ProductActions that trigger further processing. This enables complex event chains where one lifecycle event leads to others.

#### Event Chaining Pattern (NEW - Q23)

**Q23:** Can Call 2 return additional ProductActions for downstream processing?
**Answer:** Yes - Call 2 returns BOTH moves AND downstream ProductActions. This enables event chaining.

**Why Event Chaining is Needed:**

Many lifecycle events naturally trigger follow-on events. Examples:

| Initial Event | Generates Moves | Also Triggers |
|--------------|-----------------|---------------|
| Barrier breach (knockout) | Knockout booking moves | Settlement ProductAction |
| American exercise | Exercise booking moves | Settlement ProductAction, Hedge unwind ProductAction |
| Final fixing | Fixing observation moves | Settlement ProductAction (if determines payoff) |
| Corporate action (dividend) | Dividend adjustment moves | Hedge rebalancing ProductAction |
| Coupon payment | Coupon payment moves | Cash settlement ProductAction |

**Chaining Flow:**

```
Event Framework                    This Library
      |                                  |
      |  CALL 1: Barrier breach trigger  |
      |----------------------------->|   |
      |                              |   |
      |<-- ProductAction: BarrierBreach  |
      |                                  |
      |  CALL 2: Apply to positions      |
      |----------------------------->|   |
      |                              |   |
      |                              |   [generate knockout moves]
      |                              |   [determine: needs settlement]
      |                              |   [create Settlement ProductAction]
      |                              |   |
      |<-- (                             |
      |      moves: [knockout_move_1, ...],
      |      downstream_actions: [Settlement ProductAction]
      |    )                              |
      |                                  |
      |  [post moves to Ledger]          |
      |                                  |
      |  [for each downstream action:]   |
      |  CALL 2: Apply Settlement        |
      |----------------------------->|   |
      |                              |   |
      |                              |   [generate settlement moves]
      |                              |   [no further downstream actions]
      |                              |   |
      |<-- (                             |
      |      moves: [settlement_move_1, ...],
      |      downstream_actions: []
      |    )                              |
      |                                  |
      |  [post moves to Ledger]          |
```

**Event Framework Responsibility for Chaining:**
- Receive (moves, downstream_actions) tuple from Call 2
- Post moves to Ledger
- For each downstream ProductAction:
  - Look up affected positions (may be different from original)
  - Build context for each position
  - Match MoveRules based on context
  - Call apply_to_positions with the downstream ProductAction
  - Recursively process any further downstream actions
- Implement loop/recursion detection to prevent infinite chains

**Downstream ProductAction Characteristics:**
- Created by this library based on product logic
- May target the same positions OR different positions (e.g., hedge positions)
- May require different MoveRules (different action type)
- Carries lineage/causation reference to source action (for audit)

**Example: Knock-Out Warrant Barrier Breach**

```
Initial Trigger: Continuous barrier breach on KO Warrant

Call 1 Output:
  ProductAction {
    action_type: BarrierBreach,
    details: { breach_price: 95.50, breach_time: "2024-03-15T14:32:00Z" }
  }

Call 2 Input:
  - ProductAction: BarrierBreach
  - Positions: [{ product: KO_Warrant_123, qty: 1000, ... }]
  - MoveRules: [knockout_booking_rules]

Call 2 Output:
  moves: [
    { type: "knockout", debit: product_value, credit: pnl, amount: 50000 }
  ],
  downstream_actions: [
    ProductAction {
      action_type: Settlement,
      source_action_ref: "barrier_breach_xyz",  // Lineage
      details: {
        settlement_type: "knockout",
        settlement_amount_per_unit: 0,  // Worthless knockout
        settlement_date: "2024-03-17"
      }
    }
  ]
```

**Downstream Action for Hedging:**

When a product event occurs, it may also trigger hedge-related actions:

```
Initial Trigger: American exercise on Vanilla Option

Call 2 Output:
  moves: [exercise_moves],
  downstream_actions: [
    ProductAction {
      action_type: Settlement,
      target: "product_positions",
      details: { ... }
    },
    ProductAction {
      action_type: HedgeUnwind,
      target: "hedge_positions",  // Different positions!
      details: {
        reason: "product_exercise",
        source_product_id: "vanilla_opt_456"
      }
    }
  ]
```

**Interface Contract Update:**

```rust
/// Combined output from Call 2
struct ApplyResult {
    /// Ledger moves to be recorded
    moves: Vec<Move>,
    /// Additional ProductActions requiring processing
    downstream_actions: Vec<ProductAction>,
}

trait MoveApplicator {
    /// Apply a product action to positions using move rules
    /// Returns moves AND any downstream actions to process
    fn apply_to_positions(
        &self,
        product_action: &ProductAction,
        positions: &[PositionWithContext],
        move_rules: &[MoveRule],
    ) -> Result<ApplyResult, Error>;
}

/// ProductAction extended with lineage tracking
struct ProductAction {
    action_type: ActionType,
    amount_per_unit: Option<Decimal>,
    settlement_date: Option<Date>,
    details: ActionDetails,
    /// Reference to the action that caused this one (for audit trail)
    source_action_ref: Option<ProductActionRef>,
    /// Target position selector (if different from source)
    target_selector: Option<PositionSelector>,
}

enum PositionSelector {
    /// Same positions as the source action
    SameAsSource,
    /// Positions linked as hedges to the source product
    LinkedHedges { source_product_id: ProductId },
    /// Specific position IDs
    Specific { position_ids: Vec<PositionId> },
    /// Query-based selection
    ByQuery { filter: PortfolioFilter },
}
```

**Recursion Safety:**

The Event Framework must implement safeguards:
1. **Max depth limit**: Prevent infinite recursion (e.g., max 10 levels)
2. **Cycle detection**: Detect if same (action_type, product_id) appears twice in chain
3. **Audit trail**: Log full chain for debugging and compliance
4. **Atomic processing**: All moves in a chain succeed or fail together (transaction)

**Benefits of Event Chaining:**
1. **Natural modeling**: Lifecycle events naturally cascade (breach -> settlement)
2. **Separation of concerns**: Each action type handled by appropriate rules
3. **Flexibility**: Same ProductAction types can be triggered by different sources
4. **Auditability**: Full chain captured with lineage references
5. **Hedge integration**: Product events can trigger hedge operations

#### What Are Move Rules?

**Move Rules** are configuration/rules that dictate how lifecycle events translate into ledger entries. They are a **general-purpose, flexible mechanism** - NOT limited to any single dimension.

**Why Move Rules Are Needed:**
- The same economic event can require different accounting treatments
- Different contexts may require different ledger entry structures
- Some contexts require multiple ledger entries for a single event
- Some contexts use **wash books** (intermediate accounts for transfers)
- Regulatory, accounting, and business requirements vary across many dimensions

**Move Rules Are Multi-Dimensional:**

Move rules can be driven by ANY combination of dimensions, including but not limited to:

| Dimension | Description | Example Impact |
|-----------|-------------|----------------|
| **Legal Entity** | Which legal entity holds the position | Different chart of accounts, tax treatment |
| **Product Type** | Type of structured product | Different accounting classifications |
| **Regulatory Jurisdiction** | Where the position is regulated | Different reporting requirements |
| **Accounting Standard** | IFRS, GAAP, local GAAP | Different recognition rules |
| **Business Line** | Trading, treasury, client services | Different P&L attribution |
| **Client Type** | Institutional, retail, internal | Different fee structures, wash book routing |
| **Book/Portfolio** | Trading book vs banking book | Different valuation, capital treatment |
| **Tax Jurisdiction** | Where taxes apply | Withholding, deferred tax entries |

**Key Insight:** Move rules are a **general configuration mechanism**. The system should support flexible rule matching based on context attributes, not hard-coded entity lookups.

**Move Rules Definition:**

```rust
/// Context for selecting applicable move rules
struct MoveRuleContext {
    /// All attributes that can influence rule selection
    attributes: HashMap<String, String>,
    // Examples:
    // - "legal_entity" -> "Entity_A"
    // - "product_type" -> "reverse_convertible"
    // - "jurisdiction" -> "CH"
    // - "accounting_standard" -> "IFRS"
    // - "business_line" -> "structured_products"
    // - "client_type" -> "institutional"
}

/// A move rule defines how to book a specific action type
struct MoveRule {
    /// Conditions for when this rule applies (all must match)
    conditions: Vec<RuleCondition>,
    /// The booking pattern to apply
    booking_pattern: BookingPattern,
}

enum RuleCondition {
    Equals { attribute: String, value: String },
    In { attribute: String, values: Vec<String> },
    Exists { attribute: String },
    Not(Box<RuleCondition>),
}

enum BookingPattern {
    /// Direct booking: single debit/credit entry
    DirectBooking {
        debit_account: AccountTemplate,
        credit_account: AccountTemplate,
    },

    /// Wash book: route through intermediate account
    WashBookRouting {
        source_account: AccountTemplate,
        wash_book: AccountTemplate,
        destination_account: AccountTemplate,
    },

    /// Split: one event becomes multiple entries
    Split {
        entries: Vec<SplitEntry>,
    },

    /// Tax withholding: deduct tax from gross
    TaxWithholding {
        gross_account: AccountTemplate,
        tax_account: AccountTemplate,
        net_account: AccountTemplate,
        tax_rate_lookup: String,  // Reference to rate configuration
    },

    /// Composite: apply multiple patterns
    Composite {
        patterns: Vec<BookingPattern>,
    },

    /// Custom: extensible for complex scenarios
    Custom {
        rule_id: String,
        parameters: HashMap<String, Value>,
    },
}

struct SplitEntry {
    percentage: Option<Decimal>,
    fixed_amount: Option<Decimal>,
    account: AccountTemplate,
    description: String,
}

/// Account templates can include placeholders resolved from context
struct AccountTemplate {
    template: String,  // e.g., "cash_{currency}" or "trading_pnl_{desk}"
}
```

**Move Rules Are Provided BY Event Framework:**
- Event Framework determines the context (entity, product, jurisdiction, etc.)
- Event Framework looks up applicable MoveRules based on context
- This library receives MoveRules as input - it does NOT determine which rules to apply
- This library applies the rules mechanically to generate moves

**Rule Selection is Event Framework's Responsibility:**
```
Event Framework:
  1. Receives ProductAction from Call 1
  2. Looks up affected positions
  3. For each position, determines MoveRuleContext:
     - Legal entity from position/wallet
     - Product type from product
     - Jurisdiction from entity or position tags
     - Accounting standard from entity config
     - Business line from position tags
     - etc.
  4. Matches context against rule repository
  5. Passes matched rules to This Library in Call 2
```

#### Example: Same Event, Different Rules

**Scenario:** Coupon payment on a Reverse Convertible - same economic event, different booking treatments

**Call 1 - Product Level (Same for All):**
```
Input:
  - trigger: coupon_date = 2024-03-15
  - product: Reverse Convertible (ISIN: XYZ)
  - coupon_rate: 5%
  - notional: $100

Output (ProductAction):
  - action_type: "coupon_payment"
  - amount_per_unit: $5.00
  - payment_date: 2024-03-15
```

**Call 2 - Position A (Trading Book, IFRS, Direct):**
```
Context: { legal_entity: "A", book: "trading", accounting_standard: "IFRS" }
MoveRules matched: DirectBooking

Input:
  - ProductAction: { coupon_payment, $5.00 per unit }
  - Position: { quantity: 1000 }
  - MoveRules: DirectBooking { debit: cash, credit: interest_income }

Output:
  moves: [
    { debit: cash, credit: interest_income, amount: $5,000 }
  ],
  downstream_actions: []  // No further actions needed
```

**Call 2 - Position B (Client Services, Wash Book Required):**
```
Context: { legal_entity: "B", business_line: "client_services", client_type: "institutional" }
MoveRules matched: WashBookRouting

Input:
  - ProductAction: { coupon_payment, $5.00 per unit }
  - Position: { quantity: 500 }
  - MoveRules: WashBookRouting {
      source: cash_incoming,
      wash_book: coupon_suspense,
      destination: client_payable
    }

Output:
  moves: [
    { debit: cash_incoming, credit: coupon_suspense, amount: $2,500 },
    { debit: coupon_suspense, credit: client_payable, amount: $2,500 }
  ],
  downstream_actions: []
```

**Call 2 - Position C (Foreign Jurisdiction, Withholding Tax):**
```
Context: { legal_entity: "C", tax_jurisdiction: "DE", client_type: "retail" }
MoveRules matched: TaxWithholding (25% WHT)

Input:
  - ProductAction: { coupon_payment, $5.00 per unit }
  - Position: { quantity: 200 }
  - MoveRules: TaxWithholding {
      gross: coupon_receivable,
      tax: wht_payable,
      net: cash,
      rate: 0.25
    }

Output:
  moves: [
    { debit: coupon_receivable, credit: wht_payable, amount: $250 },
    { debit: coupon_receivable, credit: cash, amount: $750 }
  ],
  downstream_actions: []
```

**Call 2 - Position D (Complex Multi-Entity Structure):**
```
Context: { structure: "synthetic", requires_pnl_split: true, desks: ["desk_a", "desk_b"] }
MoveRules matched: Split + Composite

Input:
  - ProductAction: { coupon_payment, $5.00 per unit }
  - Position: { quantity: 1000 }
  - MoveRules: Composite {
      patterns: [
        Split { entries: [
          { percentage: 0.6, account: pnl_desk_a },
          { percentage: 0.4, account: pnl_desk_b }
        ]},
        DirectBooking { debit: cash, credit: pnl_suspense }
      ]
    }

Output:
  moves: [
    { debit: cash, credit: pnl_suspense, amount: $5,000 },
    { debit: pnl_suspense, credit: pnl_desk_a, amount: $3,000 },
    { debit: pnl_suspense, credit: pnl_desk_b, amount: $2,000 }
  ],
  downstream_actions: []
```

#### Why This Separation Matters

| Concern | Call 1 (Product Level) | Call 2 (Position Level) |
|---------|------------------------|-------------------------|
| Focus | Economics/Quant | Accounting/Booking |
| Question | WHAT happened? | HOW to book it? |
| Scope | Single product, qty=1 | Multiple positions, actual quantities |
| Rules | Product terms only | Configurable booking rules |
| Variability | Same for all holders | Different by context |
| Quant Lib | YES - calculates action | NO - pure transformation |
| Move Rules | NO | YES - drives output structure |
| Dimensions | None | Multiple (entity, jurisdiction, etc.) |
| Output | Single ProductAction | Moves AND downstream actions |

**Benefits of Two-Call Pattern:**

1. **Separation of Concerns:**
   - Product logic (Quant Lib) is isolated from accounting logic (Move Rules)
   - Changes to accounting rules don't affect product calculations
   - New booking patterns can be added without touching product logic

2. **Flexibility:**
   - Same ProductAction generates different moves based on context
   - Move Rules can be changed/added without code changes
   - Supports complex multi-dimensional rule matching

3. **Testability:**
   - Product-level logic can be tested independently
   - Move Rules can be tested with mock ProductActions
   - Rule matching logic can be tested separately

4. **Auditability:**
   - Clear separation between "what happened" and "how it was booked"
   - ProductAction provides audit trail of economic event
   - MoveRules provide audit trail of booking decisions
   - Rule context captured for compliance

5. **Extensibility:**
   - New dimensions can be added to context without schema changes
   - New booking patterns can be added via Custom type
   - Composite patterns allow complex scenarios

6. **Event Chaining (NEW):**
   - Call 2 can trigger downstream ProductActions
   - Enables natural modeling of cascading lifecycle events
   - Supports product-to-hedge event propagation

#### Updated Lifecycle Event Flow (with Event Chaining)

**Previous (Single-Call - SUPERSEDED):**
```
Event Framework --> This Library --> Quant Lib
                                 --> Ledger (moves)
```

**Current (Two-Call Pattern with Chaining):**
```
Event Framework                    This Library                 Quant Lib
      |                                  |                          |
      |  CALL 1: get_product_action(     |                          |
      |    trigger_info,                 |                          |
      |    product                       |                          |
      |  ) ------------------------------>|                          |
      |                                  |  get_action(              |
      |                                  |    product,               |
      |                                  |    trigger_data           |
      |                                  |  ) ---------------------->|
      |                                  |                          |
      |                                  |<-- ProductAction --------|
      |<-- ProductAction ----------------|                          |
      |                                  |                          |
      |  [Event Framework determines     |                          |
      |   affected positions,            |                          |
      |   builds context for each,       |                          |
      |   looks up MoveRules]            |                          |
      |                                  |                          |
      |  CALL 2: apply_to_positions(     |                          |
      |    product_action,               |                          |
      |    positions_with_context,       |                          |
      |    move_rules                    |                          |
      |  ) ------------------------------>|                          |
      |                                  |                          |
      |                                  |  [apply rules]           |
      |                                  |  [scale quantities]      |
      |                                  |  [generate moves]        |
      |                                  |  [identify downstream]   |
      |                                  |                          |
      |<-- (moves, downstream_actions) --|                          |
      |                                  |                          |
      |  [send moves to Ledger]          |                          |
      |                                  |                          |
      |  [FOR EACH downstream_action:]   |                          |
      |    [lookup positions for action] |                          |
      |    [build context, match rules]  |                          |
      |    CALL 2: apply_to_positions()  |                          |
      |    [recursively process results] |                          |
```

#### Interface Contracts (UPDATED)

**Call 1 - Product Action Interface:**
```rust
trait ProductActionCalculator {
    /// Calculate the product-level action for a triggered event
    /// Returns a unit-level action (quantity=1)
    fn get_product_action(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
        quant_lib: &dyn QuantLib,
    ) -> Result<ProductAction, Error>;
}

struct ProductAction {
    action_type: ActionType,
    amount_per_unit: Option<Decimal>,
    settlement_date: Option<Date>,
    details: ActionDetails,
    /// Reference to the action that caused this one (for chained events)
    source_action_ref: Option<ProductActionRef>,
    /// Target position selector (for downstream actions targeting different positions)
    target_selector: Option<PositionSelector>,
}

enum ActionType {
    CouponPayment,
    Settlement,
    BarrierBreach,
    EarlyExercise,
    DividendAdjustment,
    StockSplitAdjustment,
    FixingObserved,
    HedgeUnwind,      // NEW: For hedge-related downstream actions
    HedgeRebalance,   // NEW: For hedge adjustment downstream actions
}

enum PositionSelector {
    /// Same positions as the source action (default)
    SameAsSource,
    /// Positions linked as hedges to the source product
    LinkedHedges { source_product_id: ProductId },
    /// Specific position IDs
    Specific { position_ids: Vec<PositionId> },
    /// Query-based selection
    ByQuery { filter: PortfolioFilter },
}
```

**Call 2 - Move Application Interface (UPDATED):**
```rust
/// Combined output from Call 2
struct ApplyResult {
    /// Ledger moves to be recorded
    moves: Vec<Move>,
    /// Additional ProductActions requiring processing (event chaining)
    downstream_actions: Vec<ProductAction>,
}

trait MoveApplicator {
    /// Apply a product action to positions using move rules
    /// Returns moves AND any downstream actions to process
    fn apply_to_positions(
        &self,
        product_action: &ProductAction,
        positions: &[PositionWithContext],
        move_rules: &[MoveRule],
    ) -> Result<ApplyResult, Error>;
}

struct PositionWithContext {
    position: Position,
    context: MoveRuleContext,
    applicable_rules: Vec<MoveRule>,  // Pre-matched by Event Framework
}

struct Move {
    id: MoveId,
    position_id: PositionId,
    move_type: MoveType,
    debit_account: Account,
    credit_account: Account,
    amount: Decimal,
    effective_date: Date,
    source_action: ProductActionRef,
    rule_applied: MoveRuleRef,  // Audit trail
    context_snapshot: MoveRuleContext,  // Audit trail
    inherited_tags: HashMap<TagKey, TagValue>,
}
```

### Clarification: Unified Vision - Quantitative Event Engine (MAJOR UPDATE)

**Q20:** What is the broader vision for this system?
**Answer:** The product has been reframed from "Ledger Productionization" to a **Quantitative Event Engine** - a general-purpose event processing system with the Ledger as the central abstraction.

#### Two Ledgers Only

The system uses exactly TWO ledger types:

| Ledger Type | Purpose | Contents |
|-------------|---------|----------|
| **Position Ledger** | Products and hedges | Client products, hedging assets, trading positions |
| **Index Ledger** | Index constituents | Constituent weights, rebalancing events |

**Key Design Decision:** There is NO separate "Hedge Ledger" - hedges are tracked in the Position Ledger alongside products.

**Unified Risk Views:**
- Both ledgers can be decomposed/flattened for unified risk analysis
- Cross-ledger queries enable holistic portfolio views
- Same tagging/portfolio system works across both ledger types

#### Thin Slice Changed to Continuous Barriers

**UPDATED:** The thin slice implementation has changed from Reverse Convertible (time-based triggers) to **continuous barrier products**:

**New Thin Slice Products:**
- **Knock-Out Warrant**
- **Mini Certificate**

**Rationale:**
- The continuous barrier trigger logic **already exists** in the event framework
- This allows immediate validation of the integration pattern
- Exercises the most complex trigger type first (continuous price monitoring)
- Time-based triggers (coupon, expiry) can be added after core architecture is validated

**Thin Slice Triggers:**
- Continuous barrier monitoring (PRIMARY - already implemented)
- Fixing (initial + final)
- Expiry
- Dividends, Stock splits (corporate actions)

### Clarification: Provisional Calculations for Index Integration (NEW CONCEPT)

**Q21:** How do index calculations work before official fixings are available?
**Answer:** The system supports **Provisional Calculations** to enable real-time index values before official fixings.

#### Provisional Value Concept

Before official fixings are available, indices can be calculated using provisional (estimated/real-time) prices. These provisional values have special handling:

| Aspect | Provisional Value | Official Value |
|--------|-------------------|----------------|
| Source | Real-time prices, estimates | Official fixing |
| Status Flag | `is_provisional: true` | `is_provisional: false` |
| DAG Propagation | YES - flows downstream | YES - flows downstream |
| Stateful Persistence | **NO** - not persisted | **YES** - persisted |

#### Provisional Value Behavior

**1. DAG Propagation:**
- Provisional values propagate through the calculation DAG
- Downstream nodes receive and process provisional values
- All downstream results are also marked provisional
- Enables real-time/indicative index values for downstream consumers

**2. Stateful Operations - NO Persistence:**
- Provisional values are **NOT persisted** in stateful operations
- Affected operations include:
  - Windowing functions (rolling averages, etc.)
  - Recursive calculations
  - Moving averages
  - Any calculation that accumulates state over time

**3. Official Fixing Replacement:**
- When the official fixing arrives, it **replaces** the provisional value
- The official value **IS persisted** in stateful operations
- DAG recalculates with official value
- Downstream values update to non-provisional status

#### Example: Index Calculation Flow

```
Time: 15:30 (before market close)
  - Real-time price for AAPL: $175.50 (provisional)
  - Index calculation runs with provisional price
  - Index value: 1050.25 (provisional)
  - NOT persisted to moving average window

Time: 16:00 (market close - official fixing)
  - Official fixing for AAPL: $175.75
  - Index recalculates with official price
  - Index value: 1050.50 (official)
  - IS persisted to moving average window
  - All downstream calculations update
```

#### Data Model for Provisional Values

```rust
struct CalculatedValue<T> {
    value: T,
    is_provisional: bool,
    source_timestamp: DateTime,
    fixing_type: Option<FixingType>,  // None for provisional
}

// In DAG nodes:
impl DagNode {
    fn process(&self, inputs: Vec<CalculatedValue<f64>>) -> CalculatedValue<f64> {
        // If ANY input is provisional, output is provisional
        let is_provisional = inputs.iter().any(|v| v.is_provisional);

        let result = self.calculate(&inputs);

        CalculatedValue {
            value: result,
            is_provisional,
            source_timestamp: Utc::now(),
            fixing_type: if is_provisional { None } else { Some(FixingType::Official) },
        }
    }
}

// Stateful operations check provisional flag:
impl MovingAverageNode {
    fn process(&mut self, input: CalculatedValue<f64>) -> CalculatedValue<f64> {
        // Only persist official values
        if !input.is_provisional {
            self.window.push(input.value);  // Persist to state
        }

        // Calculate using current window (may include input for immediate result)
        let avg = self.calculate_average(input.value, input.is_provisional);

        CalculatedValue {
            value: avg,
            is_provisional: input.is_provisional,
            ..
        }
    }
}
```

#### Implications for Event Framework

The Event Framework must:
1. Track provisional vs official status of price data
2. Pass provisional flag through to calculations
3. Trigger recalculation when official fixing replaces provisional
4. Ensure provisional values don't pollute persistent state

### Clarification: Output Model and Serialization (NEW)

**Q22:** What format are moves output in and how are they persisted?
**Answer:** This library generates moves as **Rust objects** that are serialized for downstream consumption. Persistence is out of scope.

#### Output Model

| Aspect | Specification |
|--------|---------------|
| Internal Format | Rust structs (strongly typed) |
| Serialization | JSON or **Protobuf** (protobuf is standard with quant library) |
| Persistence | **OUT OF SCOPE** - how moves get posted to storage is not covered |
| Consumers | Event Framework routes serialized moves to downstream systems |

#### Move Output Flow (UPDATED for Event Chaining)

```
This Library                     Event Framework                Downstream
     |                                  |                           |
     |-- ApplyResult {                  |                           |
     |     moves: Vec<Move>,            |                           |
     |     downstream_actions: Vec<ProductAction>                   |
     |   } (Rust objects) ------------>|                           |
     |                                  |                           |
     |                        [serialize moves to JSON/protobuf]   |
     |                                  |                           |
     |                                  |-- serialized moves ------>|
     |                                  |                           |
     |                        [process downstream_actions]         |
     |                                  |                           |
     |                        [for each: lookup positions,         |
     |                         call apply_to_positions,            |
     |                         recursively handle results]         |
     |                                  |                           |
     |                                  |        [persistence layer - OUT OF SCOPE]
```

#### Serialization Format

**Protobuf (Preferred):**
- Standard format used with the quant library
- Efficient binary serialization
- Strong schema definition
- Cross-language compatibility

**JSON (Alternative):**
- Human-readable for debugging
- Wider tooling support
- Slightly larger payload size

**Move Struct with Serialization:**
```rust
use serde::{Serialize, Deserialize};
use prost::Message;  // For protobuf

#[derive(Serialize, Deserialize, Message)]
struct Move {
    #[prost(string, tag = "1")]
    id: String,

    #[prost(string, tag = "2")]
    position_id: String,

    #[prost(enumeration = "MoveType", tag = "3")]
    move_type: i32,

    #[prost(string, tag = "4")]
    debit_account: String,

    #[prost(string, tag = "5")]
    credit_account: String,

    #[prost(double, tag = "6")]
    amount: f64,

    #[prost(int64, tag = "7")]
    effective_date: i64,

    // ... additional fields
}
```

#### What This Library Does NOT Do

- **NO direct database writes** - moves are returned to Event Framework
- **NO persistence logic** - out of scope for this library
- **NO transport implementation** - serialization format only, not delivery mechanism

### Existing Code to Reference

**Similar Features Identified:**
- Feature: Product Type Definitions - Path: Existing Index calculation framework (Rust)
- Feature: Python Ledger POC - Path: `Ledger/` directory
- Feature: Continuous Barrier Logic - Path: Existing Event Framework (for thin slice)
- Components to potentially reuse: Shared product type definitions between Ledger and Index framework
- Backend logic to reference: Existing Rust patterns in the codebase (DAG framework, petgraph usage if present)

Note: The spec-writer should explore the existing Rust codebase for:
- DAG computation framework patterns
- Existing product type definitions in Index framework
- Continuous barrier trigger implementation (EVENT FRAMEWORK)
- Protobuf definitions used with quant library
- Any Axum REST API examples if present
- Adapter/bridge patterns

**Python POC Gaps to Address:**
- **Portfolios**: The Python POC does NOT have portfolio concepts. This must be designed fresh for Rust implementation.
- **Two-Call Pattern**: The separation of product-level and position-level processing is an architectural refinement beyond the Python POC.
- **Move Rules**: Configurable, multi-dimensional booking rules are a new capability.
- **Provisional Calculations**: New concept for index integration, not in Python POC.
- **Two Ledgers Architecture**: Unified Position + Index ledger model is new.
- **Event Chaining**: Call 2 returning downstream ProductActions is a new capability.

### Follow-up Questions

No additional follow-up questions were needed after the final round.

## Visual Assets

### Files Provided:
- `flows.jpg`: Hand-drawn architecture diagram showing two main flows - "New Trade" registration and "Lifecycle Event" processing (e.g., coupon payment)

### Visual Insights:

**New Trade Flow (Registration):**
1. Event Framework (Stub) initiates
2. Calls "get events" to Interface (part of this library)
3. Interface queries Quant Lib (Stub) via "register"
4. Quant Lib returns events
5. Interface registers triggers back with Event Framework

**Lifecycle Event Flow (UPDATED per Q19/Q23 - Two-Call Pattern with Chaining):**

The original visual showed a single-call pattern. This has been refined to a two-call pattern with event chaining:

**Call 1 (Product Level):**
1. Event Framework (Stub) sends trigger + product data to This Lib
2. This Lib calls Quant Lib to get product-level action
3. Quant Lib returns ProductAction (qty=1 economics)
4. This Lib returns ProductAction to Event Framework

**Call 2 (Position Level) - UPDATED:**
1. Event Framework builds context for each position (entity, jurisdiction, etc.)
2. Event Framework matches MoveRules based on context
3. Event Framework sends ProductAction + positions with context + matched rules to This Lib
4. This Lib applies MoveRules, scales to position quantities
5. This Lib generates Moves
6. **This Lib identifies any downstream ProductActions needed**
7. **(moves, downstream_actions) tuple returned to Event Framework**
8. Event Framework posts moves to Ledger
9. **Event Framework recursively processes downstream actions**

**Architecture Pattern Confirmed:**
- This library is a STATELESS adapter/bridge
- Two-way communication with Event Framework (register triggers, receive callbacks)
- One-way queries to Quant Lib (get product-level actions) - **ONLY in Call 1**
- Move output goes BACK to Event Framework (which routes to Ledger)
- **NEW: Two-call pattern separates product logic from accounting/booking logic**
- **NEW: Move rules are general-purpose, multi-dimensional configuration**
- **NEW: Call 2 can return downstream ProductActions for event chaining**

**Fidelity Level:** Low-fidelity wireframe/sketch - shows architectural flow, not implementation details

## Requirements Summary

### Unified Vision: Quantitative Event Engine

**Core Concept:** A general-purpose event processing system with the Ledger as the central abstraction. The Ledger pattern applies to multiple domains within quantitative finance.

**Two Ledgers Architecture:**

| Ledger | Purpose | Content |
|--------|---------|---------|
| **Position Ledger** | Products and hedges | Client products, hedging assets, all trading positions |
| **Index Ledger** | Index constituents | Weights, rebalancing, constituent changes |

**Key Design Decisions:**
- NO separate Hedge Ledger - hedges are in Position Ledger
- Both ledgers can be decomposed for unified risk views
- Same tagging/portfolio system across both ledger types

### Functional Requirements

**Core Smart Contract Adapter Responsibilities:**

**Call 1 - Product Action Calculation:**
- Receive lifecycle event triggers from Event Framework (stub)
- Query Quant Lib (stub) for product-level action calculations
- Return ProductAction (unit-level, quantity=1 economics)
- **NO positions involved in this call**

**Call 2 - Move Application (UPDATED - Returns Moves AND ProductActions):**
- Receive ProductAction + positions with context + MoveRules from Event Framework
- Apply MoveRules to determine ledger entry structure
- Scale ProductAction by position quantities
- Generate position-level ledger moves
- **Identify downstream ProductActions that need processing (event chaining)**
- Return (moves, downstream_actions) tuple to Event Framework

**Registration:**
- Register triggers with Event Framework during new trade processing
- Define callback payload contracts for each trigger type

**Stateless Design:**
- No internal state storage for triggers or positions
- Pure function-like behavior: inputs -> outputs
- Event Framework owns trigger registry, position lookup, MoveRules lookup
- Each call is self-contained with all needed context passed in
- **For fixing triggers: price is passed IN the callback, not queried separately**
- **Transport-agnostic: core logic processes structured data, not transport-specific messages**

**Move Rules Processing:**
- Accept MoveRules as input parameter (not internally stored or looked up)
- Support multiple booking pattern types:
  - Direct booking (single entry)
  - Wash book routing (intermediate accounts)
  - Split (one event to multiple entries)
  - Tax withholding (gross/tax/net split)
  - Composite (multiple patterns combined)
  - Custom (extensible)
- Generate appropriate Move entries based on pattern type
- Preserve audit trail (link moves to source ProductAction and applied rule)
- Capture context snapshot for compliance/audit

**Event Chaining (NEW):**
- Call 2 can return downstream ProductActions alongside moves
- Downstream actions may target same positions OR different positions (e.g., hedges)
- Each downstream action carries lineage reference to source action
- Support PositionSelector for flexible target position identification
- Event Framework responsible for recursive processing of downstream actions

**Output Model:**
- Generate moves as Rust struct objects
- Support serialization to JSON and Protobuf (protobuf is standard with quant library)
- Return ApplyResult (moves + downstream_actions) to Event Framework
- **Persistence is OUT OF SCOPE** - how moves get posted to storage is not covered

**Provisional Calculations (Index Integration):**
- Support provisional vs official value distinction
- Propagate provisional values through DAG for downstream consumers
- **DO NOT persist** provisional values in stateful operations
- Replace provisional with official when fixing arrives
- Official values ARE persisted in stateful operations

**Ledger Requirements (GAP: Portfolios):**

**Portfolio Support (not in Python POC):**
- Flexible tagging system for positions and wallets
- Non-exclusive portfolio membership (position can be in multiple portfolios)
- Portfolio-filtered queries on positions and moves
- P&L attribution by portfolio dimension
- Tag inheritance from wallet to position level
- Extensible tag schema (not hard-coded dimensions)

**Portfolio Query Capabilities:**
- Get positions by portfolio (tag filter)
- Get moves by portfolio
- Aggregate values (P&L, exposure) by portfolio
- Cross-portfolio analysis (positions appearing in multiple views)

**Products and Assets Coverage:**

The library must support two distinct categories:

**Products (14 client-facing structured products):**
1. Barrier Reverse Convertible
2. Barrier Reverse Convertible Pro
3. Bonus Certificate
4. Capped Bonus Certificate
5. Capped Bonus Pro Certificate
6. Capped Warrant
7. Discount Certificate
8. Knock-Out Warrant
9. Reverse Capped Bonus Certificate
10. Reverse Convertible
11. Mini Certificate
12. Open End Turbo
13. Factor Certificate
14. Vanilla Options

**Assets (10 tradeable hedging instruments):**
1. Equities
2. Bonds
3. Futures
4. Barrier Option
5. Interest Rate Futures
6. Interest Rate Options
7. Cash
8. FX Option
9. FX Swap
10. Stock Borrow Loan

Note: Products can also be traded as assets (e.g., for hedging purposes), so the total tradeable universe is 24 instrument types.

### Trigger Taxonomy

The library must support 8 trigger types across 6 categories:

**Category 1: Time-Based Triggers**
| Trigger | Description | Complexity | Data Required |
|---------|-------------|------------|---------------|
| Expiry | Product maturity/expiration | Low | Date only |
| Coupon payments | Scheduled coupon dates | Low | Date + coupon schedule |

**Category 2: Price Observation Triggers (Scheduled)**
| Trigger | Description | Complexity | Data Required |
|---------|-------------|------------|---------------|
| Fixing | Price observation at scheduled time | Medium | Date + time + underlying + fixing type + **observed price (passed in callback)** |

**Fixing Subtypes:**
- **Initial fixing**: Sets strike/reference price at trade inception
- **Final fixing**: Determines settlement amount at expiry
- **Periodic fixing**: For averaging (Asian-style), swap resets
- **Reset fixing**: For floating rate resets, variable coupon calculations

**Key Distinction - Fixing vs Barrier:**
| Aspect | Fixing | Barrier |
|--------|--------|---------|
| Purpose | Record actual price value | Check if level breached |
| Output | Price (decimal value) | Boolean (yes/no) |
| Timing | Specific scheduled moment | Window or continuous |
| Use | Determines amounts | Triggers state change |
| **Price Source** | **Passed in callback by Event Framework** | Event Framework evaluates |

**Category 3: Price-Based Triggers (Discrete Barrier)**
| Trigger | Description | Complexity | Data Required |
|---------|-------------|------------|---------------|
| Barrier | Barrier observation at specific times | Medium | Date + price level + direction + observation time |

**Category 4: Price-Based Triggers (Continuous Barrier)**
| Trigger | Description | Complexity | Data Required |
|---------|-------------|------------|---------------|
| Continuous barrier monitoring | Real-time/tick-level observation | High | Price level + direction + time window |

**Category 5: Exercise-Based Triggers**
| Trigger | Description | Complexity | Data Required |
|---------|-------------|------------|---------------|
| American (early exercise) | Exercise rights anytime in window | Medium | Exercise window dates + decision logic |

**CONFIRMED IN SCOPE:** American exercise is a production-required trigger type due to Vanilla Options support.

**Category 6: Corporate Action Triggers**
| Trigger | Description | Complexity | Data Required |
|---------|-------------|------------|---------------|
| Dividends | Dividend events on underlying | Medium | Ex-date + record date + payment date + amount |
| Stock splits | Split events on underlying | Medium | Ex-date + split ratio |

### Two-Call Pattern Architecture (UPDATED with Event Chaining)

**Design Principle:** Separate WHAT happened (product-level economics) from HOW to book it (position-level accounting). Call 2 can trigger downstream events.

**Call 1 - Product Action:**

| Aspect | Description |
|--------|-------------|
| Input | Trigger info + Product definition |
| Processing | Call Quant Lib for product-level calculation |
| Output | ProductAction (unit economics, qty=1) |
| Positions | NOT involved |
| Move Rules | NOT involved |

**Call 2 - Move Application (UPDATED):**

| Aspect | Description |
|--------|-------------|
| Input | ProductAction + Positions with Context + MoveRules |
| Processing | Apply rules, scale quantities, generate moves, identify downstream actions |
| Output | **ApplyResult { moves: Vec<Move>, downstream_actions: Vec<ProductAction> }** |
| Quant Lib | NOT called |
| Move Rules | Drives output structure |
| Event Chaining | Returns downstream ProductActions for further processing |

**ProductAction Types (UPDATED):**

| Action Type | Description | Typical Triggers | May Chain To |
|-------------|-------------|------------------|--------------|
| CouponPayment | Periodic interest payment | Coupon date | (usually none) |
| Settlement | Final product settlement | Expiry, Final fixing | (terminal) |
| BarrierBreach | Barrier level crossed | Barrier, Continuous barrier | Settlement |
| EarlyExercise | Option exercised early | American exercise | Settlement, HedgeUnwind |
| DividendAdjustment | Adjustment for dividend | Dividend | HedgeRebalance |
| StockSplitAdjustment | Adjustment for split | Stock split | HedgeRebalance |
| FixingObserved | Price observation recorded | Fixing | Settlement (if final) |
| HedgeUnwind | Close hedge positions | (downstream only) | (terminal) |
| HedgeRebalance | Adjust hedge positions | (downstream only) | (terminal) |

### Event Chaining Architecture (NEW)

**Design Principle:** Lifecycle events naturally cascade. Call 2 returns both moves AND downstream ProductActions.

**Chaining Examples:**

| Initial Event | Immediate Moves | Downstream Actions |
|--------------|-----------------|-------------------|
| Barrier breach (knockout) | Knockout booking | Settlement |
| American exercise | Exercise booking | Settlement, HedgeUnwind |
| Final fixing | Fixing observation | Settlement |
| Dividend announcement | Dividend adjustment | HedgeRebalance |
| Coupon payment | Coupon booking | (none typically) |

**Event Framework Responsibilities for Chaining:**
- Receive ApplyResult (moves, downstream_actions) from Call 2
- Post moves to Ledger
- For each downstream ProductAction:
  - Resolve target positions (using PositionSelector)
  - Build context for each position
  - Match MoveRules
  - Call apply_to_positions recursively
- Implement recursion safety (max depth, cycle detection)
- Maintain full audit trail of event chains

**Recursion Safety:**
- Max depth limit (e.g., 10 levels)
- Cycle detection (same action_type + product_id in chain)
- Full chain logging for audit
- Atomic processing (all moves succeed or fail together)

**ApplyResult Structure:**
```rust
struct ApplyResult {
    moves: Vec<Move>,
    downstream_actions: Vec<ProductAction>,
}
```

**PositionSelector for Downstream Actions:**
```rust
enum PositionSelector {
    SameAsSource,                              // Default: same positions
    LinkedHedges { source_product_id: ProductId },  // Hedge positions
    Specific { position_ids: Vec<PositionId> },     // Explicit list
    ByQuery { filter: PortfolioFilter },            // Query-based
}
```

### Move Rules Architecture

**Design Principle:** Move rules are a general-purpose, multi-dimensional configuration mechanism. They are NOT limited to entity-specific rules.

**Move Rule Dimensions (Non-Exhaustive):**

| Dimension | Description | Typical Impact |
|-----------|-------------|----------------|
| Legal Entity | Which entity holds position | Chart of accounts, tax |
| Product Type | Type of instrument | Accounting classification |
| Jurisdiction | Regulatory jurisdiction | Reporting, WHT |
| Accounting Standard | IFRS, GAAP, etc. | Recognition rules |
| Business Line | Trading, treasury, etc. | P&L attribution |
| Client Type | Institutional, retail | Fee handling, wash books |
| Book Type | Trading vs banking book | Valuation, capital |
| Tax Jurisdiction | Where taxes apply | WHT, deferred tax |

**Key Insight:** Rule selection is based on **context matching**, not simple entity lookup. Context is a flexible key-value map that can include any relevant dimensions.

**Booking Pattern Types:**

| Pattern | Description | Use Case |
|---------|-------------|----------|
| DirectBooking | Single debit/credit | Simple cash movements |
| WashBookRouting | Route through intermediate | Inter-entity, suspense |
| Split | One event to multiple entries | P&L allocation, fees |
| TaxWithholding | Gross/tax/net split | WHT jurisdictions |
| Composite | Multiple patterns combined | Complex multi-step |
| Custom | Extensible rule type | Unusual scenarios |

**Event Framework Responsibilities for Move Rules:**
- Build context for each position (gather relevant dimensions)
- Maintain MoveRules repository
- Match rules based on context
- Pass matched rules to this library in Call 2

**This Library Responsibilities for Move Rules:**
- Accept MoveRules as input (pre-matched by Event Framework)
- Apply booking patterns to generate move structure
- Scale amounts by position quantity
- Preserve audit trail (rule applied, context snapshot)

### Provisional Calculations Architecture

**Design Principle:** Support real-time/indicative values while protecting persistent state integrity.

**Provisional vs Official Values:**

| Aspect | Provisional | Official |
|--------|-------------|----------|
| Source | Real-time prices, estimates | Official fixing |
| Flag | `is_provisional: true` | `is_provisional: false` |
| DAG Propagation | YES | YES |
| Stateful Persistence | **NO** | **YES** |
| Use Case | Real-time indicative values | Authoritative calculations |

**Stateful Operations Protected:**
- Windowing functions
- Recursive calculations
- Moving averages
- Any accumulating state

**Replacement Flow:**
1. Provisional value calculated and propagated
2. Official fixing arrives
3. Official value replaces provisional
4. DAG recalculates with official value
5. Official value persisted to stateful operations
6. All downstream values update to non-provisional

### Callback Contract Architecture

**Design Principle:** This library defines what data it needs; Event Framework provides it. Transport is abstracted.

**Three-Phase Pattern:**

**Phase 1 - Registration (This Library -> Event Framework):**
| What This Library Provides | Purpose |
|---------------------------|---------|
| Trigger condition | When to fire (date, price level, etc.) |
| Callback payload schema | What data to include when firing |
| Product identifiers | Which products are affected |

**Phase 2 - Call 1 Execution (Event Framework -> This Library):**
| What Event Framework Provides | Purpose |
|------------------------------|---------|
| Trigger metadata | What triggered |
| Product data | Product definition for calculation |
| Observed price (if fixing) | Price at fixing time |
| Provisional flag (if applicable) | Whether price is provisional |

**Phase 3 - Call 2 Execution (Event Framework -> This Library):**
| What Event Framework Provides | Purpose |
|------------------------------|---------|
| ProductAction (from Call 1) | What happened |
| Positions with context | Who is affected + their context |
| Matched MoveRules | How to book (pre-matched) |

**Call 2 Returns (UPDATED):**
| What This Library Returns | Purpose |
|--------------------------|---------|
| Vec<Move> | Ledger entries to record |
| Vec<ProductAction> | Downstream actions for chaining |

**Callback Payload Contracts by Trigger Type (Call 1):**

| Trigger Type | Call 1 Payload | Notes |
|--------------|----------------|-------|
| Expiry | `{ date, product }` | Settlement calculation |
| Coupon | `{ date, product, coupon_details }` | Payment amount |
| Fixing | `{ date, product, observed_price, fixing_type, is_provisional }` | Price + provisional flag |
| Barrier (discrete) | `{ date, product, breach_status, barrier_level }` | Boolean breach |
| Barrier (continuous) | `{ breach_timestamp, product, barrier_level }` | Real-time notification |
| American exercise | `{ exercise_date, product, exercise_decision }` | Early exercise |
| Dividend | `{ ex_date, product, dividend_amount }` | Corp action |
| Stock split | `{ ex_date, product, split_ratio }` | Corp action |

**Call 2 Payload (All Trigger Types):**

```
{
  product_action: ProductAction,
  positions: [
    {
      position: { position_id, quantity, wallet_id, tags },
      context: { /* multi-dimensional context map */ },
      applicable_rules: [ /* pre-matched rules */ ]
    }
  ]
}
```

**Call 2 Response (UPDATED):**

```
{
  moves: [
    { id, position_id, move_type, debit_account, credit_account, amount, ... }
  ],
  downstream_actions: [
    {
      action_type: "Settlement",
      source_action_ref: "barrier_breach_xyz",
      target_selector: { type: "SameAsSource" },
      details: { ... }
    }
  ]
}
```

**Transport Agnosticism (UPDATED):**

```
                    +------------------+
                    |  Event Framework |
                    +------------------+
                            |
              +-------------+-------------+
              |                           |
         Call 1                       Call 2
              |                           |
              v                           v
    +------------------+        +------------------+
    |   This Library   |        |   This Library   |
    | (product action) |        | (move application)|
    +------------------+        +------------------+
              |                           |
              v                           v
    +------------------+        +------------------+
    |    Quant Lib     |        |    ApplyResult   |
    +------------------+        | (moves + actions)|
                                +------------------+
                                          |
                        +-----------------+-----------------+
                        |                                   |
                        v                                   v
              +------------------+                +------------------+
              |  Serialized Moves|                | Downstream       |
              | (JSON/Protobuf)  |                | ProductActions   |
              +------------------+                | (recurse Call 2) |
                        |                         +------------------+
                        v
              +------------------+
              |      Ledger      |
              | (via Event Fwk)  |
              +------------------+
```

**Key Architectural Points:**
- Core library receives structured Rust types, not raw JSON/bytes
- Runner handles deserialization (separate concern)
- Same core logic works regardless of transport
- Enables unit testing without transport infrastructure
- Supports future transport changes without core logic changes
- **Two distinct call interfaces, each with specific contracts**
- **Move output serialized to JSON or Protobuf**
- **Move output goes back to Event Framework, not directly to Ledger**
- **Persistence is OUT OF SCOPE**
- **Context is multi-dimensional, rules are pre-matched**
- **Call 2 returns ApplyResult with moves AND downstream actions**
- **Event Framework handles recursive processing of downstream actions**

### Product-to-Trigger Mapping

Based on typical structured product characteristics:

| Product | Expiry | Coupon | Fixing | Barrier (Discrete) | Barrier (Continuous) | American | Dividends | Stock Splits |
|---------|--------|--------|--------|-------------------|---------------------|----------|-----------|--------------|
| Barrier Reverse Convertible | Y | Y | Y (init/final) | Y | possible | N | Y | Y |
| Barrier Reverse Convertible Pro | Y | Y | Y (init/final) | Y | possible | N | Y | Y |
| Bonus Certificate | Y | N | Y (init/final) | Y | possible | N | Y | Y |
| Capped Bonus Certificate | Y | N | Y (init/final) | Y | possible | N | Y | Y |
| Capped Bonus Pro Certificate | Y | N | Y (init/final) | Y | possible | N | Y | Y |
| Capped Warrant | Y | N | Y (init/final) | N | N | possible | Y | Y |
| Discount Certificate | Y | N | Y (init/final) | N | N | N | Y | Y |
| **Knock-Out Warrant** | **Y** | **N** | **Y (init/final)** | **Y** | **Y** | **N** | **Y** | **Y** |
| Reverse Capped Bonus Certificate | Y | N | Y (init/final) | Y | possible | N | Y | Y |
| Reverse Convertible | Y | Y | Y (init/final) | N | N | N | Y | Y |
| **Mini Certificate** | **Y** | **N** | **Y (init/final)** | **Y** | **Y** | **N** | **Y** | **Y** |
| Open End Turbo | N* | N | Y (init) | Y | Y | N | Y | Y |
| Factor Certificate | N* | N | Y (periodic/reset) | N | N | N | Y | Y |
| Vanilla Options | Y | N | Y (init/final) | N | N | Y | Y | Y |

*Open-ended products have no fixed expiry but may have other termination triggers
**Highlighted products are the thin slice implementation targets**

**Fixing Usage by Product:**
- **All products**: Initial fixing (to set strike/reference)
- **Fixed-term products**: Final fixing (settlement calculation)
- **Factor Certificate**: Periodic/reset fixings (daily leverage reset)
- **Products with variable coupons**: Reset fixings (coupon amount calculation)
- **Asian-style variants**: Periodic fixings (averaging)
- **Vanilla Options**: Initial fixing (strike), final fixing (settlement)

**American Exercise Usage:**
- **Vanilla Options**: Primary product with American exercise (CONFIRMED production requirement)
- **Capped Warrant**: Possible American-style variants

### Portfolio Architecture (Not in Python POC)

**Design Principle:** Portfolios are views/lenses over positions, not containers. Membership is computed via tag filtering.

**Core Concepts:**

| Concept | Description | Storage |
|---------|-------------|---------|
| Tag | Key-value label on a position | Stored with position |
| Portfolio | Named filter criteria | Configuration/query |
| Membership | Positions matching filter | Computed at query time |

**Tag Dimensions (Extensible):**

| Dimension | Purpose | Example Values |
|-----------|---------|----------------|
| strategy | Trading strategy attribution | "yield_enhancement", "delta_neutral" |
| desk | Organizational unit | "desk_a", "london", "hk" |
| trader | Individual attribution | "john", "sarah" |
| client | Client relationship | "client_abc", "institutional" |
| product_type | Instrument categorization | "brc", "warrant", "vanilla" |
| underlying | Asset exposure | "AAPL", "tech_sector" |
| book | Accounting book | "trading", "banking" |
| regulatory_book | Regulatory classification | "trading_book", "banking_book" |
| risk_category | Risk-based grouping | "high_delta", "expiring_soon" |

**Position Data Model Enhancement:**

```rust
struct Position {
    id: PositionId,
    wallet_id: WalletId,
    instrument: Instrument,
    quantity: Decimal,
    // Flexible tagging for portfolio membership
    tags: HashMap<TagKey, TagValue>,
}

struct Move {
    id: MoveId,
    position_id: PositionId,
    amount: Decimal,
    move_type: MoveType,
    // Inherited tags for attribution
    inherited_tags: HashMap<TagKey, TagValue>,
    // Audit trail to source action and rule
    source_action: ProductActionRef,
    rule_applied: MoveRuleRef,
    context_snapshot: HashMap<String, String>,
}
```

**Portfolio Query Interface:**

```rust
trait LedgerQuery {
    // Get positions matching portfolio criteria
    fn positions_by_portfolio(&self, filter: &PortfolioFilter) -> Vec<Position>;

    // Get moves for positions in portfolio
    fn moves_by_portfolio(&self, filter: &PortfolioFilter) -> Vec<Move>;

    // Aggregate P&L by portfolio
    fn pnl_by_portfolio(&self, filter: &PortfolioFilter) -> PnLSummary;
}

struct PortfolioFilter {
    criteria: Vec<TagCriterion>,  // AND logic
}

enum TagCriterion {
    Equals(TagKey, TagValue),
    In(TagKey, Vec<TagValue>),
    Exists(TagKey),
}
```

**Tag Inheritance:**
- Wallets can have default tags
- Positions inherit wallet tags at creation
- Position-level tags can override wallet defaults
- Moves inherit position tags at creation

### Thin Slice Implementation Strategy (UPDATED)

**CHANGED: Starting with Continuous Barrier Products**

The thin slice has been updated from Reverse Convertible to continuous barrier products because the continuous barrier trigger logic **already exists** in the event framework.

**Thin Slice Products:**
- **Knock-Out Warrant** (PRIMARY)
- **Mini Certificate**

**Rationale:**
- Continuous barrier monitoring already implemented in Event Framework
- Validates the most complex trigger type first
- Exercises real-time price monitoring integration
- Time-based triggers (coupon, expiry) are simpler and can be added after

**Phase 1 - Thin Slice (Continuous Barrier Products):**
- Products: Knock-Out Warrant, Mini Certificate
- Triggers: **Continuous barrier monitoring (PRIMARY)**, Fixing (initial + final), Expiry, Dividends, Stock splits
- Assets needed: Equities, Cash
- Validates:
  - Continuous price monitoring integration
  - Real-time barrier breach detection
  - Two-call pattern with MoveRules
  - Move serialization (JSON + Protobuf)
  - **Event chaining (barrier breach -> settlement)**
- **Include basic portfolio tagging and query support**
- **Implement at least 3 booking patterns (DirectBooking, WashBookRouting, Split)**
- **Demonstrate multi-dimensional context (entity + product type at minimum)**

**Phase 2 - Add Time-Based Triggers:**
- Products: Reverse Convertible, Barrier Reverse Convertible
- New triggers: Coupon payments
- Validates: Scheduled time-based triggers, coupon payment flows

**Phase 3 - Add Discrete Barriers:**
- Products: Discount Certificate, Bonus Certificate
- New trigger: Barrier (discrete observation)
- Validates: Price-based discrete trigger handling, interaction between fixings and barriers

**Phase 4 - Add Periodic Fixings:**
- Products: Factor Certificate, Asian-style variants
- New trigger variant: Periodic/reset fixings
- Validates: Multiple scheduled fixings, averaging logic

**Phase 5 - Add American Exercise:**
- Products: **Vanilla Options**, Capped Warrant (if American-style)
- New trigger: American (early exercise)
- Validates: Exercise decision handling, early termination flows
- **Validates: Event chaining to hedge positions (exercise -> HedgeUnwind)**
- **Note:** Vanilla Options are the PRIMARY use case for this phase

**Phase 6 - Add Index Ledger Integration:**
- Enable Index Ledger alongside Position Ledger
- Implement provisional calculation support
- Validate cross-ledger queries for unified risk views

**Phase 7 - Complete Coverage:**
- Remaining products and edge cases
- All 14 products + 10 asset types fully supported
- **Full portfolio query capabilities and P&L attribution**
- **All booking patterns implemented (including TaxWithholding, Composite, Custom)**
- **Full multi-dimensional rule matching**
- **Full event chaining scenarios including hedge operations**

### Reusability Opportunities

- Product type definitions from Index framework (Rust)
- Python Ledger POC as domain model reference (excluding portfolio concepts, two-call pattern)
- Existing Rust patterns in codebase (DAG, petgraph if used)
- Move/ContractResult patterns from Python POC (concepts, not code)
- **Continuous barrier trigger logic from Event Framework**
- **Protobuf definitions from quant library**

### Scope Boundaries

**In Scope:**
- Smart Contract Adapter library (Rust, stateless)
- Interface contracts for Event Framework integration
- Interface contracts for Quant Lib integration
- Ledger move generation
- Complete trigger taxonomy (8 trigger types including Fixing and American exercise)
- **Knock-Out Warrant + Mini Certificate** for initial thin slice (continuous barriers)
- Implementation roadmap for all 14 products + 10 asset types
- Stub definitions for external dependencies (must simulate all trigger types)
- American (early exercise) trigger type - CONFIRMED IN SCOPE
- Callback payload contract definitions for all trigger types
- Registration contract specification
- Portfolio tagging and query system (GAP in Python POC)
- Position/wallet tagging mechanism
- Portfolio-filtered queries on Ledger
- Two-Call Pattern implementation (Call 1: ProductAction, Call 2: Move Application)
- ProductAction types and interface
- MoveRules types and processing logic
- Booking patterns (DirectBooking, WashBookRouting, Split, TaxWithholding, Composite, Custom)
- Multi-dimensional context for rule matching
- Audit trail (rule applied, context snapshot)
- **Two Ledgers: Position Ledger + Index Ledger**
- **Provisional calculation support for Index integration**
- **Move serialization to JSON and Protobuf**
- **Output model as Rust structs with serialization**
- **Event chaining - Call 2 returns (moves, downstream_actions)**
- **ApplyResult struct with moves and downstream ProductActions**
- **PositionSelector for downstream action targeting**
- **Lineage tracking (source_action_ref) for audit**

**Out of Scope:**
- Event Framework implementation (stub only)
- Quant Lib implementation (stub only)
- Market data service (stub only) - **Note: This library has no direct Market dependency for fixings**
- **Persistence/database layer - how moves get posted to storage**
- Reporting systems
- UI/API layer
- Python code or PyO3 bindings
- Transport layer implementation (REST/Kafka/etc. - abstracted away)
- MoveRules repository/storage (Event Framework responsibility)
- Rule matching logic (Event Framework responsibility - rules are pre-matched)
- Position lookup by trigger (Event Framework responsibility)
- Context building (Event Framework responsibility)
- **Recursion handling for event chains (Event Framework responsibility)**
- **Cycle detection in event chains (Event Framework responsibility)**

### Technical Considerations

**Language & Stack:**
- 100% Rust implementation
- No Python in production path
- Python Ledger POC is reference only
- **Protobuf support** (standard with quant library)

**Architecture Pattern:**
- **"Quantitative Event Engine"** pattern
- Three components: Quant Lib (pricing) | This Library (adapter) | Event Framework (orchestration)
- **Two Ledgers**: Position Ledger + Index Ledger
- Stateless adapter design
- Clear interface boundaries via traits
- **This library has NO direct Market dependency** - Event Framework handles price acquisition for fixings
- **Transport-agnostic design** - core logic separated from message delivery
- **Two-Call Pattern** - separates product logic from accounting logic
- **Move rules are general-purpose, multi-dimensional** - not entity-specific
- **Provisional calculations** for real-time index values
- **Event chaining** - Call 2 returns downstream ProductActions

**Output Model:**
- Moves generated as Rust structs
- Serialization: JSON + Protobuf (protobuf is standard)
- Output returned to Event Framework as ApplyResult
- **ApplyResult contains both moves AND downstream_actions**
- **Persistence is OUT OF SCOPE**

**Integration Points:**
- Event Framework: **Two callback interfaces** (Call 1: product action, Call 2: move application + downstream actions) + trigger registration interface
- Quant Lib: Query interface for product-level calculations - **ONLY called in Call 1**
- Ledger: Move output interface (via Event Framework) + **portfolio query interface**
- ~~Market: Price query interface~~ - **NOT NEEDED: Event Framework passes prices in callbacks**
- **Protobuf**: Serialization format for moves (standard with quant library)

**Event Framework Contract (This Library Defines):**

This library defines the contract that Event Framework must fulfill:

| Contract Element | Definition |
|-----------------|------------|
| Registration API | How to register triggers with payload requirements |
| Trigger conditions | Supported condition types (date, price, etc.) |
| **Call 1 payload schemas** | Required data for product action calculation |
| **Call 2 payload schemas** | ProductAction + positions with context + matched rules |
| **Call 2 response schema** | ApplyResult with moves + downstream_actions |
| Message format | JSON/Protobuf schema for callback messages |
| **Provisional flag** | Support for provisional vs official values |
| **Event chaining contract** | How downstream_actions are processed |

**Event Framework Responsibilities (Stub Must Simulate):**
- Store trigger registrations (condition + payload requirements)
- Detect when trigger conditions are met
- **Call 1:** Send trigger + product data to this library
- **Receive ProductAction from Call 1**
- **Look up affected positions**
- **Build context for each position** (multi-dimensional)
- **Match MoveRules based on context**
- **Call 2:** Send ProductAction + positions with context + matched rules to this library
- **Receive ApplyResult (moves + downstream_actions) from Call 2**
- **Serialize moves** (JSON or Protobuf)
- **Route serialized moves to Ledger**
- **Process downstream_actions recursively**
- **Implement recursion safety (max depth, cycle detection)**
- **Does NOT know Quant Lib internals** (this library handles that)
- **Track provisional vs official status** of price data

**Event Framework Stub Requirements:**
The stub must be capable of simulating all 8 trigger types:
- Time-based: Simple date scheduling
- Fixing: Scheduled price observation - **stub must provide price + provisional flag in callback**
- Discrete barrier: Price + date combination (returns breach boolean)
- **Continuous barrier: Simulated tick stream with barrier breach detection** (PRIMARY for thin slice)
- American exercise: Exercise notification interface - **CONFIRMED production requirement for Vanilla Options**
- Corporate actions: Dividend and split event injection

**Stub must also:**
- Maintain mock position data for Call 2
- Build mock context for positions (multiple dimensions)
- Maintain mock MoveRules repository
- Perform rule matching based on context
- Orchestrate the two-call sequence correctly
- **Serialize moves to JSON and Protobuf**
- Route resulting moves to Ledger (stub)
- **Track provisional vs official value status**
- **Handle downstream_actions from Call 2 responses**
- **Implement event chaining with recursion safety**

**Quality Priorities (in order):**
1. Correctness
2. Maintainability
3. Type safety
4. Performance (eventual, not immediate optimization target)

**Shared Code:**
- Product type definitions must be shared between Ledger and Index framework
- Asset type definitions may need new shared crate or extension of existing
- Trigger type definitions should be in shared module for Event Framework integration
- Callback payload types should be shareable for Event Framework integration
- ProductAction types should be shareable for Event Framework integration
- MoveRules types and BookingPattern types should be shareable
- MoveRuleContext should be shareable for Event Framework integration
- Tag/portfolio types may be shared if other systems need portfolio awareness
- **Protobuf message definitions** should be shared
- **Provisional value wrapper types** should be shared
- **ApplyResult type** should be shareable for Event Framework integration
- **PositionSelector type** should be shareable
- Evaluate if this requires a shared crate or module structure

**Domain Model Distinction:**
- Products = client-facing structured products with complex lifecycle events (14 types including Vanilla Options)
- Assets = tradeable instruments for hedging (simpler lifecycle but varied types)
- Both categories generate ledger moves but may have different event patterns
- Products trigger hedging operations that create asset positions
- Trigger complexity varies significantly by product type (time-only vs continuous monitoring)
- Fixings are fundamental to most products (determine strikes, settlements, coupon amounts)
- **Fixing prices are provided by Event Framework, not queried by this library**
- **American exercise is CONFIRMED IN SCOPE** - Vanilla Options are the primary product requiring this trigger type
- **Portfolios provide organizational views across all positions regardless of product/asset type**
- **Product logic (WHAT happened) is separate from accounting logic (HOW to book)**
- **MoveRules are general-purpose, multi-dimensional configuration** - entity, jurisdiction, accounting standard, business line, client type, etc.
- **Same economic event can have many different booking treatments based on context**
- **Two Ledgers**: Position Ledger (products + hedges) and Index Ledger (constituents)
- **Provisional calculations** enable real-time indicative values while protecting stateful operations
- **Event chaining** enables natural modeling of cascading lifecycle events

### Python POC Gaps Summary

The following features are NOT present in the Python Ledger POC and must be designed fresh for the Rust implementation:

| Gap | Description | Priority |
|-----|-------------|----------|
| **Portfolios** | Tagging, filtering, and multi-dimensional views of positions | High - fundamental for production use |
| **Two-Call Pattern** | Separation of product-level action from position-level booking | High - architectural clarity |
| **Move Rules** | General-purpose, multi-dimensional configurable booking rules | High - multi-context support |
| **Two Ledgers** | Position Ledger + Index Ledger architecture | High - unified vision |
| **Provisional Calculations** | Support for provisional vs official values in DAG | High - Index integration |
| **Protobuf Serialization** | Standard serialization format for moves | Medium - quant library compatibility |
| **Event Chaining** | Call 2 returning downstream ProductActions | High - natural lifecycle modeling |

These gaps represent opportunities to improve upon the POC design rather than simply porting it.
