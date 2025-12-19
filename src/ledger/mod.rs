//! Quantitative Event Engine - Ledger Domain Model
//!
//! This module provides the core library for processing lifecycle events
//! for structured products and hedges, implementing a three-call pattern:
//! - Call 0: Product registration - translates QuantLib events to subscriptions
//! - Call 1: Trigger processing - produces ProductAction (unit economics)
//! - Call 2: Position application - produces ApplyResult (Moves + downstream)
//!
//! ## Core Library Modules
//!
//! - `domain`: Products (14 types), Assets (10 types), Instruments
//! - `triggers`: 8 trigger types for lifecycle events
//! - `types`: Position, Move, Ledger traits, portfolio tagging
//! - `actions`: ProductAction, ActionType, PositionSelector
//! - `move_rules`: MoveRules engine with 6 booking patterns
//! - `apply_result`: ApplyResult with event chaining support
//! - `in_memory`: InMemoryLedger implementation for research/testing
//!
//! ## Stubs (Testing/Demo Only)
//!
//! - `stubs`: Mock implementations of external systems (EventFramework, QuantLib)
//!   - These are NOT part of the core library
//!   - Used for testing and demonstrating the two-call pattern
//!   - Production would integrate with real external systems

// =============================================================================
// Core Library Modules
// =============================================================================

pub mod actions;
pub mod apply_result;
pub mod domain;
pub mod in_memory;
pub mod move_rules;
pub mod triggers;
pub mod types;

// =============================================================================
// Stubs Module (Testing/Demo Only)
// =============================================================================

/// Stub implementations for external dependencies.
///
/// **Note**: This module is for testing and demonstration purposes only.
/// Production implementations would integrate with real external systems:
/// - EventFramework: Trigger detection, position lookup, rule matching
/// - QuantLib: Product-level pricing and calculations
pub mod stubs;

// =============================================================================
// Integration Tests
// =============================================================================

#[cfg(test)]
mod tests;

// =============================================================================
// Core Library Re-exports
// =============================================================================

pub use domain::{
    Asset, AssetDetails, AssetId, AssetProto, AssetType, Instrument, InstrumentId, Product,
    ProductDetails, ProductId, ProductProto, ProductType,
};

pub use triggers::{
    BarrierDirection, CorporateActionDetails, CorporateActionType, FixingType, TriggerCategory,
    TriggerInfo, TriggerInfoProto, TriggerProductMatrix, TriggerType,
};

pub use types::{
    Account, Ledger, LedgerError, LedgerReader, LedgerResult, LedgerWriter, Move, MoveId,
    MoveProto, MoveRuleRef, MoveType, PortfolioFilter, Position, PositionId, PositionProto,
    ProductActionRef, TagCriterion, TagKey, TagValue, WalletId,
};

pub use actions::{
    // Call 0: Product Registration
    TriggerSubscription, SubscriptionCriteria, ProductEventDetails,
    BarrierEventDetail, BarrierObservationType, FixingEventDetail, ExpiryEventDetail,
    CouponEventDetail, AveragingWindow, translate_to_subscriptions,
    // Call 1: Trigger Processing
    ActionDetails, ActionError, ActionResult, ActionType, PositionSelector, ProductAction,
    ProductActionCalculator, ProductActionProto, QuantLib, SettlementReason, SettlementType,
};

pub use move_rules::{
    BookingPattern, DefaultTaxRateProvider, MoveRule, MoveRuleApplicator, MoveRuleContext,
    MoveRuleError, MoveRuleResult, RuleCondition, SplitEntry, TaxRateProvider,
    find_matching_rule,
};

pub use apply_result::{
    ApplyError, ApplyResult, ApplyResultType, DefaultMoveApplicator, MoveApplicator,
    PositionWithContext,
};

pub use in_memory::{InMemoryLedger, Wallet};

// =============================================================================
// Stub Re-exports (for convenience in tests)
// =============================================================================

pub use stubs::{
    EventFramework, EventFrameworkError, EventFrameworkResult, EventFrameworkStub,
    QuantLibStub, QuantLibTrait, SimulationResult, TriggerRegistration,
    KnockOutWarrantHandler, MiniCertificateHandler, TwoCallFlowResult,
    ApplyResultProto, create_thin_slice_rules, execute_two_call_flow,
};
