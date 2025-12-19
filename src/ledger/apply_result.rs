//! Call 2 Interface and ApplyResult for the Quantitative Event Engine.
//!
//! This module implements the Call 2 interface that transforms ProductActions
//! into ledger moves with support for event chaining:
//! - PositionWithContext: Position with pre-matched rules and context
//! - ApplyResult: Bundles moves with downstream ProductActions
//! - MoveApplicator: The Call 2 interface trait and implementation

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::actions::{
    ActionDetails, ActionType, PositionSelector, ProductAction, SettlementReason, SettlementType,
};
use super::move_rules::{
    find_matching_rule, MoveRule, MoveRuleApplicator, MoveRuleContext, MoveRuleError,
};
use super::types::{Move, Position};

/// Position with context and pre-matched rules for Call 2 processing.
///
/// The Event Framework pre-builds the context and matches applicable rules
/// before calling the MoveApplicator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionWithContext {
    /// The position being processed
    pub position: Position,
    /// Context for rule matching (built by Event Framework)
    pub context: MoveRuleContext,
    /// Pre-matched rules from Event Framework
    pub applicable_rules: Vec<MoveRule>,
}

impl PositionWithContext {
    /// Create a new PositionWithContext.
    pub fn new(position: Position, context: MoveRuleContext) -> Self {
        PositionWithContext {
            position,
            context,
            applicable_rules: Vec::new(),
        }
    }

    /// Add pre-matched rules.
    pub fn with_applicable_rules(mut self, rules: Vec<MoveRule>) -> Self {
        self.applicable_rules = rules;
        self
    }

    /// Add a single applicable rule.
    pub fn with_rule(mut self, rule: MoveRule) -> Self {
        self.applicable_rules.push(rule);
        self
    }
}

/// Result of applying a ProductAction to positions (Call 2 output).
///
/// Contains both the generated moves and any downstream ProductActions
/// for event chaining (e.g., barrier breach -> settlement).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplyResult {
    /// Generated ledger moves
    pub moves: Vec<Move>,
    /// Downstream ProductActions for event chaining
    pub downstream_actions: Vec<ProductAction>,
}

impl ApplyResult {
    /// Create a new empty ApplyResult.
    pub fn new() -> Self {
        ApplyResult {
            moves: Vec::new(),
            downstream_actions: Vec::new(),
        }
    }

    /// Create an ApplyResult with moves only.
    pub fn with_moves(moves: Vec<Move>) -> Self {
        ApplyResult {
            moves,
            downstream_actions: Vec::new(),
        }
    }

    /// Create an ApplyResult with moves and downstream actions.
    pub fn with_moves_and_actions(moves: Vec<Move>, downstream_actions: Vec<ProductAction>) -> Self {
        ApplyResult {
            moves,
            downstream_actions,
        }
    }

    /// Add moves to the result.
    pub fn add_moves(&mut self, moves: Vec<Move>) {
        self.moves.extend(moves);
    }

    /// Add a downstream action.
    pub fn add_downstream_action(&mut self, action: ProductAction) {
        self.downstream_actions.push(action);
    }

    /// Check if there are any downstream actions.
    pub fn has_downstream_actions(&self) -> bool {
        !self.downstream_actions.is_empty()
    }

    /// Get the total number of moves.
    pub fn move_count(&self) -> usize {
        self.moves.len()
    }

    /// Get the total number of downstream actions.
    pub fn downstream_action_count(&self) -> usize {
        self.downstream_actions.len()
    }
}

impl Default for ApplyResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Error types for MoveApplicator operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApplyError {
    #[error("No matching rule found for position {0}")]
    NoMatchingRule(String),

    #[error("Move rule error: {0}")]
    MoveRuleError(#[from] MoveRuleError),

    #[error("Missing required data: {0}")]
    MissingData(String),

    #[error("Invalid position quantity for position {0}: {1}")]
    InvalidQuantity(String, String),
}

/// Result type for MoveApplicator operations.
pub type ApplyResultType<T> = Result<T, ApplyError>;

/// Trait for applying ProductActions to positions (Call 2 interface).
///
/// This is the main interface for the two-call pattern's second call:
/// - Input: ProductAction + positions with context + MoveRules
/// - Output: ApplyResult containing moves and downstream actions
pub trait MoveApplicator: Send + Sync {
    /// Apply a ProductAction to positions and generate moves.
    ///
    /// This is "Call 2" in the two-call pattern:
    /// - Applies ProductAction to each position
    /// - Scales amount_per_unit by position quantity
    /// - Applies matched MoveRules to generate moves
    /// - Identifies downstream ProductActions based on action_type
    /// - Sets source_action_ref for lineage
    fn apply_to_positions(
        &self,
        product_action: &ProductAction,
        positions: &[PositionWithContext],
        move_rules: &[MoveRule],
    ) -> ApplyResultType<ApplyResult>;
}

/// Default implementation of MoveApplicator.
pub struct DefaultMoveApplicator {
    rule_applicator: MoveRuleApplicator,
}

impl DefaultMoveApplicator {
    /// Create a new DefaultMoveApplicator.
    pub fn new() -> Self {
        DefaultMoveApplicator {
            rule_applicator: MoveRuleApplicator::new(),
        }
    }

    /// Generate a unique action ID for downstream actions.
    fn generate_action_id(&self) -> String {
        format!("ACTION-{}", Uuid::new_v4())
    }

    /// Get the effective date for moves.
    ///
    /// Uses the settlement date from the action if available,
    /// otherwise uses today's date.
    fn get_effective_date(&self, action: &ProductAction) -> NaiveDate {
        action
            .settlement_date
            .unwrap_or_else(|| chrono::Utc::now().date_naive())
    }

    /// Create downstream Settlement action for BarrierBreach.
    ///
    /// When a barrier is breached, a settlement action is generated
    /// to close out the position.
    fn create_barrier_breach_settlement(
        &self,
        source_action: &ProductAction,
        settlement_amount: Option<Decimal>,
    ) -> ProductAction {
        let settlement_date = source_action
            .settlement_date
            .unwrap_or_else(|| chrono::Utc::now().date_naive());

        let mut downstream = ProductAction::new(
            self.generate_action_id(),
            ActionType::Settlement,
            source_action.product_id.clone(),
        )
        .with_source_action(source_action.id.clone())
        .with_settlement_date(settlement_date)
        .with_target_selector(PositionSelector::SameAsSource)
        .with_details(ActionDetails::Settlement {
            settlement_type: SettlementType::Cash,
            reason: SettlementReason::BarrierBreach,
        });

        // Use breach price from barrier details as settlement amount if available
        if let Some(amount) = settlement_amount {
            downstream = downstream.with_amount(amount);
        } else if let ActionDetails::Barrier { breach_price, .. } = &source_action.details {
            downstream = downstream.with_amount(*breach_price);
        }

        downstream
    }

    /// Generate downstream actions based on action type.
    ///
    /// Implements event chaining for:
    /// - BarrierBreach -> Settlement
    fn generate_downstream_actions(
        &self,
        product_action: &ProductAction,
    ) -> Vec<ProductAction> {
        let mut downstream = Vec::new();

        match product_action.action_type {
            ActionType::BarrierBreach => {
                // Barrier breach triggers settlement
                let settlement_amount = match &product_action.details {
                    ActionDetails::Barrier { breach_price, .. } => Some(*breach_price),
                    _ => product_action.amount_per_unit,
                };
                downstream.push(self.create_barrier_breach_settlement(
                    product_action,
                    settlement_amount,
                ));
            }
            // Other action types can be extended here
            _ => {}
        }

        downstream
    }

    /// Apply the ProductAction to a single position.
    fn apply_to_position(
        &self,
        product_action: &ProductAction,
        pos_ctx: &PositionWithContext,
        all_rules: &[MoveRule],
    ) -> ApplyResultType<Vec<Move>> {
        // Find matching rule: first check pre-matched rules, then fall back to all rules
        let rule = if !pos_ctx.applicable_rules.is_empty() {
            find_matching_rule(&pos_ctx.applicable_rules, &pos_ctx.context)
        } else {
            find_matching_rule(all_rules, &pos_ctx.context)
        };

        let rule = rule.ok_or_else(|| {
            ApplyError::NoMatchingRule(pos_ctx.position.id.as_str().to_string())
        })?;

        let effective_date = self.get_effective_date(product_action);

        // Apply the rule to generate moves
        let moves = self.rule_applicator.apply_rule(
            rule,
            &pos_ctx.position,
            product_action,
            &pos_ctx.context,
            effective_date,
        )?;

        Ok(moves)
    }
}

impl Default for DefaultMoveApplicator {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveApplicator for DefaultMoveApplicator {
    fn apply_to_positions(
        &self,
        product_action: &ProductAction,
        positions: &[PositionWithContext],
        move_rules: &[MoveRule],
    ) -> ApplyResultType<ApplyResult> {
        let mut result = ApplyResult::new();

        // Apply action to each position
        for pos_ctx in positions {
            let moves = self.apply_to_position(product_action, pos_ctx, move_rules)?;
            result.add_moves(moves);
        }

        // Generate downstream actions based on action type
        let downstream = self.generate_downstream_actions(product_action);
        for action in downstream {
            result.add_downstream_action(action);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::{Instrument, Product, ProductDetails, ProductId};
    use crate::ledger::move_rules::{BookingPattern, RuleCondition};
    use crate::ledger::types::{MoveType, ProductActionRef};

    // Helper to create a test position
    fn create_test_position(id: &str, quantity: Decimal) -> Position {
        let instrument = Instrument::Product(Product::KnockOutWarrant(ProductDetails::new(
            "KO-001",
            "Test Knock-Out Warrant",
            "AAPL",
            "USD",
        )));
        Position::new(id, "WALLET-001", instrument, quantity)
            .with_tag("desk", "desk_a")
            .with_tag("strategy", "yield_enhancement")
    }

    // Helper to create test context
    fn create_test_context() -> MoveRuleContext {
        MoveRuleContext::new()
            .with_attribute("legal_entity", "ENTITY_A")
            .with_attribute("jurisdiction", "US")
            .with_attribute("product_type", "KnockOutWarrant")
    }

    // Helper to create DirectBooking rule
    fn create_direct_booking_rule() -> MoveRule {
        MoveRule::new(
            "RULE-DIRECT-001",
            "Direct Settlement",
            BookingPattern::direct("CASH_ACCOUNT", "PRODUCT_ACCOUNT"),
        )
        .with_move_type(MoveType::Settlement)
        .with_priority(100)
    }

    // Helper to create WashBookRouting rule
    fn create_wash_book_rule() -> MoveRule {
        MoveRule::new(
            "RULE-WASH-001",
            "Intercompany Settlement",
            BookingPattern::wash_book("SOURCE_CASH", "WASH_BOOK", "DEST_CASH"),
        )
        .with_condition(RuleCondition::equals("jurisdiction", "INTERCOMPANY"))
        .with_move_type(MoveType::Settlement)
        .with_priority(150)
    }

    // Test 1: ApplyResult construction with moves and downstream_actions
    #[test]
    fn test_apply_result_construction() {
        // Empty result
        let result = ApplyResult::new();
        assert_eq!(result.move_count(), 0);
        assert_eq!(result.downstream_action_count(), 0);
        assert!(!result.has_downstream_actions());

        // Result with moves only
        let position = create_test_position("POS-001", Decimal::new(100, 0));
        let mov = Move::new(
            "MOV-001",
            position.id.clone(),
            MoveType::Settlement,
            "CASH",
            "PRODUCT",
            Decimal::new(10000, 2),
            NaiveDate::from_ymd_opt(2024, 12, 15).unwrap(),
            "ACTION-001",
            "RULE-001",
        );
        let result = ApplyResult::with_moves(vec![mov.clone()]);
        assert_eq!(result.move_count(), 1);
        assert!(!result.has_downstream_actions());

        // Result with moves and downstream actions
        let downstream_action = ProductAction::new(
            "ACTION-002",
            ActionType::Settlement,
            ProductId::new("KO-001"),
        )
        .with_source_action(ProductActionRef::new("ACTION-001"));

        let result = ApplyResult::with_moves_and_actions(
            vec![mov.clone()],
            vec![downstream_action.clone()],
        );
        assert_eq!(result.move_count(), 1);
        assert_eq!(result.downstream_action_count(), 1);
        assert!(result.has_downstream_actions());

        // Test add methods
        let mut result = ApplyResult::new();
        result.add_moves(vec![mov]);
        result.add_downstream_action(downstream_action);
        assert_eq!(result.move_count(), 1);
        assert_eq!(result.downstream_action_count(), 1);
    }

    // Test 2: MoveApplicator with DirectBooking (single position)
    #[test]
    fn test_move_applicator_direct_booking_single_position() {
        let applicator = DefaultMoveApplicator::new();

        let position = create_test_position("POS-001", Decimal::new(100, 0));
        let context = create_test_context();
        let rule = create_direct_booking_rule();

        let pos_with_ctx = PositionWithContext::new(position, context).with_rule(rule.clone());

        let product_action = ProductAction::new(
            "ACTION-001",
            ActionType::Settlement,
            ProductId::new("KO-001"),
        )
        .with_amount(Decimal::new(1050, 2)) // 10.50 per unit
        .with_settlement_date(NaiveDate::from_ymd_opt(2024, 12, 20).unwrap());

        let result = applicator
            .apply_to_positions(&product_action, &[pos_with_ctx], &[rule])
            .expect("Should succeed");

        // Should generate 1 move for DirectBooking
        assert_eq!(result.move_count(), 1);

        let mov = &result.moves[0];
        assert_eq!(mov.debit_account.as_str(), "CASH_ACCOUNT");
        assert_eq!(mov.credit_account.as_str(), "PRODUCT_ACCOUNT");
        // 100 units * 10.50 = 1050.00
        assert_eq!(mov.amount, Decimal::new(105000, 2));
        assert_eq!(mov.move_type, MoveType::Settlement);
        assert_eq!(mov.source_action.as_str(), "ACTION-001");
        assert_eq!(mov.rule_applied.as_str(), "RULE-DIRECT-001");
        assert_eq!(
            mov.effective_date,
            NaiveDate::from_ymd_opt(2024, 12, 20).unwrap()
        );

        // Verify inherited tags
        assert_eq!(mov.inherited_tags.get("desk"), Some(&"desk_a".to_string()));
    }

    // Test 3: MoveApplicator with WashBookRouting pattern
    #[test]
    fn test_move_applicator_wash_book_routing() {
        let applicator = DefaultMoveApplicator::new();

        let position = create_test_position("POS-001", Decimal::new(50, 0));
        let context = MoveRuleContext::new()
            .with_attribute("legal_entity", "ENTITY_A")
            .with_attribute("jurisdiction", "INTERCOMPANY")
            .with_attribute("product_type", "KnockOutWarrant");

        let wash_rule = create_wash_book_rule();
        let direct_rule = create_direct_booking_rule();

        let pos_with_ctx = PositionWithContext::new(position, context)
            .with_rule(wash_rule.clone())
            .with_rule(direct_rule.clone());

        let product_action = ProductAction::new(
            "ACTION-002",
            ActionType::Settlement,
            ProductId::new("KO-001"),
        )
        .with_amount(Decimal::new(2000, 2)) // 20.00 per unit
        .with_settlement_date(NaiveDate::from_ymd_opt(2024, 12, 15).unwrap());

        let result = applicator
            .apply_to_positions(&product_action, &[pos_with_ctx], &[wash_rule, direct_rule])
            .expect("Should succeed");

        // WashBookRouting generates 2 moves (3-leg entry: source->wash, wash->dest)
        assert_eq!(result.move_count(), 2);

        // Leg 1: Source -> Wash
        let leg1 = &result.moves[0];
        assert_eq!(leg1.debit_account.as_str(), "SOURCE_CASH");
        assert_eq!(leg1.credit_account.as_str(), "WASH_BOOK");
        // 50 units * 20.00 = 1000.00
        assert_eq!(leg1.amount, Decimal::new(100000, 2));

        // Leg 2: Wash -> Destination
        let leg2 = &result.moves[1];
        assert_eq!(leg2.debit_account.as_str(), "WASH_BOOK");
        assert_eq!(leg2.credit_account.as_str(), "DEST_CASH");
        assert_eq!(leg2.amount, Decimal::new(100000, 2));

        // Both legs should reference the same source action and rule
        assert_eq!(leg1.source_action.as_str(), "ACTION-002");
        assert_eq!(leg2.source_action.as_str(), "ACTION-002");
        assert_eq!(leg1.rule_applied.as_str(), "RULE-WASH-001");
        assert_eq!(leg2.rule_applied.as_str(), "RULE-WASH-001");
    }

    // Test 4: Downstream action generation (barrier breach -> settlement)
    #[test]
    fn test_downstream_action_barrier_breach_to_settlement() {
        let applicator = DefaultMoveApplicator::new();

        let position = create_test_position("POS-001", Decimal::new(100, 0));
        let context = create_test_context();
        let rule = create_direct_booking_rule();

        let pos_with_ctx = PositionWithContext::new(position, context).with_rule(rule.clone());

        // Create BarrierBreach action with barrier details
        let barrier_action = ProductAction::new(
            "ACTION-BARRIER-001",
            ActionType::BarrierBreach,
            ProductId::new("KO-001"),
        )
        .with_amount(Decimal::ZERO) // No direct payment on breach
        .with_settlement_date(NaiveDate::from_ymd_opt(2024, 12, 15).unwrap())
        .with_details(ActionDetails::Barrier {
            barrier_level: Decimal::new(9000, 2),   // 90.00
            breach_price: Decimal::new(8950, 2),    // 89.50
            is_knockout: true,
        });

        let result = applicator
            .apply_to_positions(&barrier_action, &[pos_with_ctx], &[rule])
            .expect("Should succeed");

        // Should have moves from the barrier action processing
        assert!(result.move_count() >= 1);

        // Should have downstream settlement action
        assert!(result.has_downstream_actions());
        assert_eq!(result.downstream_action_count(), 1);

        let downstream = &result.downstream_actions[0];
        assert_eq!(downstream.action_type, ActionType::Settlement);
        assert_eq!(downstream.product_id, ProductId::new("KO-001"));

        // Check source action reference for lineage
        assert!(downstream.source_action_ref.is_some());
        assert_eq!(
            downstream.source_action_ref.as_ref().unwrap().as_str(),
            "ACTION-BARRIER-001"
        );

        // Check target selector
        assert_eq!(
            downstream.target_selector,
            Some(PositionSelector::SameAsSource)
        );

        // Check settlement details
        if let ActionDetails::Settlement { settlement_type, reason } = &downstream.details {
            assert_eq!(*settlement_type, SettlementType::Cash);
            assert_eq!(*reason, SettlementReason::BarrierBreach);
        } else {
            panic!("Expected Settlement details");
        }

        // Settlement amount should be the breach price
        assert_eq!(downstream.amount_per_unit, Some(Decimal::new(8950, 2)));
    }

    // Test 5: Multiple positions processing
    #[test]
    fn test_multiple_positions_processing() {
        let applicator = DefaultMoveApplicator::new();

        let position1 = create_test_position("POS-001", Decimal::new(100, 0));
        let position2 = create_test_position("POS-002", Decimal::new(200, 0));

        let context1 = create_test_context();
        let context2 = create_test_context();

        let rule = create_direct_booking_rule();

        let positions_with_ctx = vec![
            PositionWithContext::new(position1, context1).with_rule(rule.clone()),
            PositionWithContext::new(position2, context2).with_rule(rule.clone()),
        ];

        let product_action = ProductAction::new(
            "ACTION-001",
            ActionType::Settlement,
            ProductId::new("KO-001"),
        )
        .with_amount(Decimal::new(500, 2)) // 5.00 per unit
        .with_settlement_date(NaiveDate::from_ymd_opt(2024, 12, 20).unwrap());

        let result = applicator
            .apply_to_positions(&product_action, &positions_with_ctx, &[rule])
            .expect("Should succeed");

        // Should generate 1 move per position (DirectBooking)
        assert_eq!(result.move_count(), 2);

        // Position 1: 100 units * 5.00 = 500.00
        let mov1 = &result.moves[0];
        assert_eq!(mov1.position_id.as_str(), "POS-001");
        assert_eq!(mov1.amount, Decimal::new(50000, 2));

        // Position 2: 200 units * 5.00 = 1000.00
        let mov2 = &result.moves[1];
        assert_eq!(mov2.position_id.as_str(), "POS-002");
        assert_eq!(mov2.amount, Decimal::new(100000, 2));
    }

    // Test 6: Lineage tracking with source_action_ref
    #[test]
    fn test_lineage_tracking_source_action_ref() {
        let applicator = DefaultMoveApplicator::new();

        let position = create_test_position("POS-001", Decimal::new(100, 0));
        let context = create_test_context();
        let rule = create_direct_booking_rule();

        let pos_with_ctx = PositionWithContext::new(position, context).with_rule(rule.clone());

        // Create a barrier breach action
        let barrier_action = ProductAction::new(
            "ACTION-ROOT-BARRIER",
            ActionType::BarrierBreach,
            ProductId::new("KO-001"),
        )
        .with_amount(Decimal::ZERO)
        .with_settlement_date(NaiveDate::from_ymd_opt(2024, 12, 15).unwrap())
        .with_details(ActionDetails::Barrier {
            barrier_level: Decimal::new(10000, 2),
            breach_price: Decimal::new(9800, 2),
            is_knockout: true,
        });

        let result = applicator
            .apply_to_positions(&barrier_action, &[pos_with_ctx], &[rule])
            .expect("Should succeed");

        // Verify moves have correct source action reference
        for mov in &result.moves {
            assert_eq!(mov.source_action.as_str(), "ACTION-ROOT-BARRIER");
        }

        // Verify downstream action has source_action_ref for lineage
        assert_eq!(result.downstream_action_count(), 1);
        let downstream = &result.downstream_actions[0];

        // The downstream settlement should reference the barrier action
        assert!(downstream.is_chained());
        assert_eq!(
            downstream.source_action_ref.as_ref().unwrap().as_str(),
            "ACTION-ROOT-BARRIER"
        );

        // Verify settlement date is preserved
        assert_eq!(
            downstream.settlement_date,
            Some(NaiveDate::from_ymd_opt(2024, 12, 15).unwrap())
        );
    }
}
