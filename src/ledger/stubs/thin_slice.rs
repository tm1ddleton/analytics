//! Thin Slice Implementation for Continuous Barrier Products.
//!
//! This module implements the thin slice validation of the entire architecture
//! using Knock-Out Warrants and Mini Certificates with continuous barrier monitoring:
//! - KnockOutWarrantHandler: Processes continuous barrier, fixing, and expiry triggers
//! - MiniCertificateHandler: Similar logic with Mini-specific variations
//! - Full two-call flow demonstration
//! - Event chaining (barrier breach -> settlement)
//! - JSON and Protobuf serialization

use prost::Message;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ledger::actions::{
    ActionDetails, ActionError, ActionResult, ActionType, PositionSelector, ProductAction,
    ProductActionCalculator, ProductEventDetails, QuantLib, SettlementReason, SettlementType,
};
use crate::ledger::apply_result::{
    ApplyError, ApplyResult, DefaultMoveApplicator, MoveApplicator, PositionWithContext,
};
use crate::ledger::domain::{Product, ProductType};
use crate::ledger::move_rules::{BookingPattern, MoveRule, RuleCondition, SplitEntry};
use crate::ledger::triggers::{FixingType, TriggerInfo, TriggerType};
use crate::ledger::types::{Move, MoveProto, MoveType};

/// Knock-Out Warrant product handler implementing ProductActionCalculator.
///
/// Processes:
/// - ContinuousBarrier trigger -> BarrierBreach action -> Settlement downstream
/// - Fixing trigger -> FixingObserved action
/// - Expiry trigger -> Settlement action
pub struct KnockOutWarrantHandler;

impl KnockOutWarrantHandler {
    /// Create a new handler.
    pub fn new() -> Self {
        KnockOutWarrantHandler
    }

    /// Generate a unique action ID.
    fn generate_action_id(&self) -> String {
        format!("ACTION-KO-{}", Uuid::new_v4())
    }

    /// Process a continuous barrier trigger.
    fn process_barrier_breach(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
    ) -> ActionResult<ProductAction> {
        let barrier_level = trigger_info.barrier_level.ok_or_else(|| {
            ActionError::MissingData("barrier_level required for ContinuousBarrier".to_string())
        })?;

        let breach_price = trigger_info.observed_price.ok_or_else(|| {
            ActionError::MissingData("observed_price required for barrier breach".to_string())
        })?;

        let settlement_date = trigger_info.trigger_time.date_naive();

        // Create the BarrierBreach action
        let action = ProductAction::new(
            self.generate_action_id(),
            ActionType::BarrierBreach,
            product.id(),
        )
        .with_amount(breach_price) // Settlement amount is breach price for KO warrants
        .with_settlement_date(settlement_date)
        .with_details(ActionDetails::Barrier {
            barrier_level,
            breach_price,
            is_knockout: true,
        })
        .with_target_selector(PositionSelector::SameAsSource);

        Ok(action)
    }

    /// Process a fixing trigger.
    fn process_fixing(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
        fixing_type: FixingType,
    ) -> ActionResult<ProductAction> {
        let observed_price = trigger_info.observed_price.ok_or_else(|| {
            ActionError::MissingData("observed_price required for fixing".to_string())
        })?;

        let action = ProductAction::new(
            self.generate_action_id(),
            ActionType::FixingObserved,
            product.id(),
        )
        .with_details(ActionDetails::Fixing {
            observed_price,
            fixing_type: fixing_type.as_str().to_string(),
            underlying: product.details().underlying.clone(),
        })
        .with_target_selector(PositionSelector::SameAsSource);

        Ok(action)
    }

    /// Process an expiry trigger.
    fn process_expiry(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
    ) -> ActionResult<ProductAction> {
        let settlement_price = trigger_info.observed_price.ok_or_else(|| {
            ActionError::MissingData("observed_price required for expiry settlement".to_string())
        })?;

        let settlement_date = trigger_info.trigger_time.date_naive();

        let action = ProductAction::new(
            self.generate_action_id(),
            ActionType::Settlement,
            product.id(),
        )
        .with_amount(settlement_price)
        .with_settlement_date(settlement_date)
        .with_details(ActionDetails::Settlement {
            settlement_type: SettlementType::Cash,
            reason: SettlementReason::Maturity,
        })
        .with_target_selector(PositionSelector::SameAsSource);

        Ok(action)
    }
}

impl Default for KnockOutWarrantHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProductActionCalculator for KnockOutWarrantHandler {
    fn get_product_action(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
        _quant_lib: &dyn QuantLib,
    ) -> ActionResult<ProductAction> {
        // Verify product type
        if product.product_type_enum() != ProductType::KnockOutWarrant {
            return Err(ActionError::TriggerNotApplicable(
                trigger_info.trigger_type.as_str().to_string(),
                product.product_type().to_string(),
            ));
        }

        match &trigger_info.trigger_type {
            TriggerType::ContinuousBarrier => self.process_barrier_breach(trigger_info, product),
            TriggerType::Fixing(fixing_type) => {
                self.process_fixing(trigger_info, product, *fixing_type)
            }
            TriggerType::Expiry => self.process_expiry(trigger_info, product),
            _ => Err(ActionError::TriggerNotApplicable(
                trigger_info.trigger_type.as_str().to_string(),
                product.product_type().to_string(),
            )),
        }
    }
}

/// Mini Certificate product handler implementing ProductActionCalculator.
///
/// Similar to KnockOutWarrant with Mini-specific logic:
/// - Continuous barrier monitoring
/// - Event chaining on knockout
/// - Different settlement calculation for Mini products
pub struct MiniCertificateHandler;

impl MiniCertificateHandler {
    /// Create a new handler.
    pub fn new() -> Self {
        MiniCertificateHandler
    }

    /// Generate a unique action ID.
    fn generate_action_id(&self) -> String {
        format!("ACTION-MINI-{}", Uuid::new_v4())
    }

    /// Process a continuous barrier trigger for Mini Certificate.
    ///
    /// Mini certificates have a stop-loss level (barrier) that when breached
    /// results in a residual value settlement.
    fn process_barrier_breach(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
    ) -> ActionResult<ProductAction> {
        let barrier_level = trigger_info.barrier_level.ok_or_else(|| {
            ActionError::MissingData("barrier_level required for ContinuousBarrier".to_string())
        })?;

        let breach_price = trigger_info.observed_price.ok_or_else(|| {
            ActionError::MissingData("observed_price required for barrier breach".to_string())
        })?;

        // Mini certificate residual value = max(0, breach_price - barrier_level)
        // For simplicity, we use breach_price as the settlement amount
        let settlement_amount = if breach_price > barrier_level {
            breach_price - barrier_level
        } else {
            Decimal::ZERO
        };

        let settlement_date = trigger_info.trigger_time.date_naive();

        let action = ProductAction::new(
            self.generate_action_id(),
            ActionType::BarrierBreach,
            product.id(),
        )
        .with_amount(settlement_amount)
        .with_settlement_date(settlement_date)
        .with_details(ActionDetails::Barrier {
            barrier_level,
            breach_price,
            is_knockout: true,
        })
        .with_target_selector(PositionSelector::SameAsSource);

        Ok(action)
    }

    /// Process a fixing trigger.
    fn process_fixing(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
        fixing_type: FixingType,
    ) -> ActionResult<ProductAction> {
        let observed_price = trigger_info.observed_price.ok_or_else(|| {
            ActionError::MissingData("observed_price required for fixing".to_string())
        })?;

        let action = ProductAction::new(
            self.generate_action_id(),
            ActionType::FixingObserved,
            product.id(),
        )
        .with_details(ActionDetails::Fixing {
            observed_price,
            fixing_type: fixing_type.as_str().to_string(),
            underlying: product.details().underlying.clone(),
        })
        .with_target_selector(PositionSelector::SameAsSource);

        Ok(action)
    }

    /// Process an expiry trigger.
    fn process_expiry(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
    ) -> ActionResult<ProductAction> {
        let settlement_price = trigger_info.observed_price.ok_or_else(|| {
            ActionError::MissingData("observed_price required for expiry settlement".to_string())
        })?;

        let settlement_date = trigger_info.trigger_time.date_naive();

        let action = ProductAction::new(
            self.generate_action_id(),
            ActionType::Settlement,
            product.id(),
        )
        .with_amount(settlement_price)
        .with_settlement_date(settlement_date)
        .with_details(ActionDetails::Settlement {
            settlement_type: SettlementType::Cash,
            reason: SettlementReason::Maturity,
        })
        .with_target_selector(PositionSelector::SameAsSource);

        Ok(action)
    }
}

impl Default for MiniCertificateHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProductActionCalculator for MiniCertificateHandler {
    fn get_product_action(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
        _quant_lib: &dyn QuantLib,
    ) -> ActionResult<ProductAction> {
        // Verify product type
        if product.product_type_enum() != ProductType::MiniCertificate {
            return Err(ActionError::TriggerNotApplicable(
                trigger_info.trigger_type.as_str().to_string(),
                product.product_type().to_string(),
            ));
        }

        match &trigger_info.trigger_type {
            TriggerType::ContinuousBarrier => self.process_barrier_breach(trigger_info, product),
            TriggerType::Fixing(fixing_type) => {
                self.process_fixing(trigger_info, product, *fixing_type)
            }
            TriggerType::Expiry => self.process_expiry(trigger_info, product),
            _ => Err(ActionError::TriggerNotApplicable(
                trigger_info.trigger_type.as_str().to_string(),
                product.product_type().to_string(),
            )),
        }
    }
}

/// Creates the standard set of MoveRules for the thin slice.
///
/// Returns at least 3 patterns:
/// - DirectBooking: Standard knockout/settlement
/// - WashBookRouting: Intercompany settlement
/// - Split: Multi-desk P&L allocation
pub fn create_thin_slice_rules() -> Vec<MoveRule> {
    vec![
        // Pattern 1: DirectBooking - Standard knockout/settlement
        MoveRule::new(
            "RULE-DIRECT-KO",
            "Direct Knockout Settlement",
            BookingPattern::direct("POSITION_CASH", "SETTLEMENT_ACCOUNT"),
        )
        .with_condition(RuleCondition::equals("product_type", "KnockOutWarrant"))
        .with_condition(RuleCondition::equals("booking_type", "standard"))
        .with_move_type(MoveType::Settlement)
        .with_priority(100),
        // Pattern 1b: DirectBooking - Standard settlement for Mini Certificate
        MoveRule::new(
            "RULE-DIRECT-MINI",
            "Direct Mini Settlement",
            BookingPattern::direct("POSITION_CASH", "SETTLEMENT_ACCOUNT"),
        )
        .with_condition(RuleCondition::equals("product_type", "MiniCertificate"))
        .with_condition(RuleCondition::equals("booking_type", "standard"))
        .with_move_type(MoveType::Settlement)
        .with_priority(100),
        // Pattern 2: WashBookRouting - Intercompany settlement
        MoveRule::new(
            "RULE-WASH-INTERCO",
            "Intercompany Settlement via Wash Book",
            BookingPattern::wash_book(
                "ENTITY_A_CASH",
                "INTERCOMPANY_WASH",
                "ENTITY_B_CASH",
            ),
        )
        .with_condition(RuleCondition::equals("booking_type", "intercompany"))
        .with_move_type(MoveType::Settlement)
        .with_priority(150),
        // Pattern 3: Split - Multi-desk P&L allocation
        MoveRule::new(
            "RULE-SPLIT-DESK",
            "Multi-Desk P&L Split",
            BookingPattern::split(vec![
                SplitEntry::new("CASH", "DESK_A_PNL", Decimal::new(60, 2))
                    .with_label("Desk A (60%)"),
                SplitEntry::new("CASH", "DESK_B_PNL", Decimal::new(40, 2))
                    .with_label("Desk B (40%)"),
            ]),
        )
        .with_condition(RuleCondition::equals("booking_type", "multi_desk"))
        .with_move_type(MoveType::Settlement)
        .with_priority(120),
        // Default catch-all rule
        MoveRule::new(
            "RULE-DEFAULT",
            "Default Settlement",
            BookingPattern::direct("CASH", "SETTLEMENT"),
        )
        .with_move_type(MoveType::Settlement)
        .with_priority(0),
    ]
}

/// Stub implementation of QuantLib for thin slice testing.
pub struct QuantLibStub;

impl QuantLib for QuantLibStub {
    fn get_product_events(&self, product: &Product) -> ActionResult<ProductEventDetails> {
        use crate::ledger::actions::{
            BarrierEventDetail, ExpiryEventDetail, FixingEventDetail, ProductEventDetails,
        };
        use crate::ledger::triggers::BarrierDirection;
        use chrono::{TimeZone, Utc};

        let product_id = product.id();
        let underlying = product.details().underlying.clone();

        // Build product-specific event details
        let mut events = ProductEventDetails::new(product_id)
            .with_corporate_action_sensitivity(true);

        // Add barrier for KnockOut and Mini products
        match product.product_type_enum() {
            ProductType::KnockOutWarrant | ProductType::MiniCertificate => {
                // Default barrier at 90% of notional
                let barrier = BarrierEventDetail::continuous(
                    Decimal::new(9000, 2), // Default barrier level
                    BarrierDirection::Down,
                    &underlying,
                    chrono::Utc::now().date_naive(),
                    chrono::Utc::now().date_naive() + chrono::Duration::days(365),
                );
                events = events.with_barrier(barrier);

                // Add initial and final fixings
                let initial_fixing = FixingEventDetail::new(
                    crate::ledger::triggers::FixingType::Initial,
                    Utc::now(),
                    &underlying,
                );
                let final_fixing = FixingEventDetail::new(
                    crate::ledger::triggers::FixingType::Final,
                    Utc::now() + chrono::Duration::days(365),
                    &underlying,
                );
                events = events.with_fixing(initial_fixing).with_fixing(final_fixing);

                // Add expiry
                let expiry = ExpiryEventDetail::new(
                    Utc::now() + chrono::Duration::days(365),
                    crate::ledger::actions::SettlementType::Cash,
                );
                events = events.with_expiry(expiry);
            }
            _ => {}
        }

        Ok(events)
    }

    fn calculate_action(
        &self,
        product: &Product,
        trigger_info: &TriggerInfo,
    ) -> ActionResult<ProductAction> {
        // For the thin slice, delegate to the appropriate handler
        match product.product_type_enum() {
            ProductType::KnockOutWarrant => {
                KnockOutWarrantHandler::new().get_product_action(trigger_info, product, self)
            }
            ProductType::MiniCertificate => {
                MiniCertificateHandler::new().get_product_action(trigger_info, product, self)
            }
            _ => Err(ActionError::TriggerNotApplicable(
                trigger_info.trigger_type.as_str().to_string(),
                product.product_type().to_string(),
            )),
        }
    }
}

/// Result of the full two-call flow.
///
/// Contains all generated moves and any pending downstream actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoCallFlowResult {
    /// ProductAction from Call 1
    pub product_action: ProductAction,
    /// ApplyResult from Call 2
    pub apply_result: ApplyResult,
    /// Results from processing downstream actions (simulated recursive processing)
    pub downstream_results: Vec<ApplyResult>,
}

impl TwoCallFlowResult {
    /// Get total number of moves across all results.
    pub fn total_moves(&self) -> usize {
        self.apply_result.move_count()
            + self
                .downstream_results
                .iter()
                .map(|r| r.move_count())
                .sum::<usize>()
    }

    /// Get all moves flattened into a single vector.
    pub fn all_moves(&self) -> Vec<Move> {
        let mut moves = self.apply_result.moves.clone();
        for result in &self.downstream_results {
            moves.extend(result.moves.clone());
        }
        moves
    }
}

/// Execute the full two-call flow for a trigger.
///
/// Call 1: TriggerInfo + Product -> ProductAction
/// Call 2: ProductAction + Positions + Rules -> ApplyResult
/// Event chaining: Process downstream_actions recursively (simulated)
pub fn execute_two_call_flow(
    trigger_info: &TriggerInfo,
    product: &Product,
    positions: &[PositionWithContext],
    move_rules: &[MoveRule],
    max_depth: usize,
) -> Result<TwoCallFlowResult, ApplyError> {
    let quant_lib = QuantLibStub;
    let applicator = DefaultMoveApplicator::new();

    // Call 1: Get ProductAction from trigger + product
    let handler: Box<dyn ProductActionCalculator> = match product.product_type_enum() {
        ProductType::KnockOutWarrant => Box::new(KnockOutWarrantHandler::new()),
        ProductType::MiniCertificate => Box::new(MiniCertificateHandler::new()),
        _ => {
            return Err(ApplyError::MissingData(format!(
                "No handler for product type: {}",
                product.product_type()
            )))
        }
    };

    let product_action = handler
        .get_product_action(trigger_info, product, &quant_lib)
        .map_err(|e| ApplyError::MissingData(e.to_string()))?;

    // Call 2: Apply ProductAction to positions
    let apply_result = applicator.apply_to_positions(&product_action, positions, move_rules)?;

    // Process downstream actions (simulated recursive processing)
    let mut downstream_results = Vec::new();

    if max_depth > 0 && apply_result.has_downstream_actions() {
        for downstream_action in &apply_result.downstream_actions {
            // For simulation, we apply downstream actions to the same positions
            let downstream_result =
                applicator.apply_to_positions(downstream_action, positions, move_rules)?;
            downstream_results.push(downstream_result);
        }
    }

    Ok(TwoCallFlowResult {
        product_action,
        apply_result,
        downstream_results,
    })
}

/// Protobuf representation for ApplyResult.
#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct ApplyResultProto {
    /// Moves as protobuf messages
    #[prost(message, repeated, tag = "1")]
    pub moves: Vec<MoveProto>,

    /// Number of downstream actions (actions serialized separately)
    #[prost(uint32, tag = "2")]
    pub downstream_action_count: u32,
}

impl ApplyResult {
    /// Convert to protobuf representation.
    pub fn to_proto(&self) -> ApplyResultProto {
        ApplyResultProto {
            moves: self.moves.iter().map(|m| m.to_proto()).collect(),
            downstream_action_count: self.downstream_actions.len() as u32,
        }
    }

    /// Encode to protobuf bytes.
    pub fn encode_proto(&self) -> Vec<u8> {
        let proto = self.to_proto();
        let mut buf = Vec::new();
        proto.encode(&mut buf).expect("encoding should not fail");
        buf
    }

    /// Convert to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Convert to pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::{Instrument, ProductDetails};
    use crate::ledger::move_rules::MoveRuleContext;
    use crate::ledger::triggers::BarrierDirection;
    use crate::ledger::types::Position;
    use chrono::{NaiveDate, TimeZone, Utc};

    // Helper to create a test KnockOutWarrant product
    fn create_ko_product(id: &str) -> Product {
        Product::KnockOutWarrant(ProductDetails::new(id, "Test KO Warrant", "AAPL", "USD"))
    }

    // Helper to create a test MiniCertificate product
    fn create_mini_product(id: &str) -> Product {
        Product::MiniCertificate(ProductDetails::new(id, "Test Mini Cert", "DAX", "EUR"))
    }

    // Helper to create a test position
    fn create_test_position(id: &str, product: &Product, quantity: Decimal) -> Position {
        let instrument = Instrument::Product(product.clone());
        Position::new(id, "WALLET-001", instrument, quantity)
            .with_tag("desk", "desk_a")
            .with_tag("strategy", "leverage")
    }

    // Helper to create barrier trigger
    fn create_barrier_trigger(breach_price: Decimal, barrier_level: Decimal) -> TriggerInfo {
        TriggerInfo::new_barrier(
            true, // continuous
            Utc.with_ymd_and_hms(2024, 12, 15, 10, 30, 0).unwrap(),
            breach_price,
            barrier_level,
            BarrierDirection::Down,
        )
    }

    #[test]
    fn test_knockout_warrant_barrier_breach() {
        let product = create_ko_product("KO-001");
        let handler = KnockOutWarrantHandler::new();
        let quant_lib = QuantLibStub;

        let trigger = create_barrier_trigger(Decimal::new(8950, 2), Decimal::new(9000, 2));

        let action = handler
            .get_product_action(&trigger, &product, &quant_lib)
            .expect("Should produce ProductAction");

        assert_eq!(action.action_type, ActionType::BarrierBreach);
        assert_eq!(action.product_id, product.id());
        assert_eq!(action.amount_per_unit, Some(Decimal::new(8950, 2)));
    }

    #[test]
    fn test_mini_certificate_barrier_breach() {
        let product = create_mini_product("MINI-001");
        let position = create_test_position("POS-001", &product, Decimal::new(50, 0));

        // Mini with barrier at 14000, breached at 13950
        let trigger = create_barrier_trigger(Decimal::new(1395000, 2), Decimal::new(1400000, 2));

        let context = MoveRuleContext::new()
            .with_attribute("product_type", "MiniCertificate")
            .with_attribute("booking_type", "standard");

        let rules = create_thin_slice_rules();
        let pos_with_ctx = PositionWithContext::new(position, context);

        let result = execute_two_call_flow(&trigger, &product, &[pos_with_ctx], &rules, 1)
            .expect("Should succeed");

        assert_eq!(result.product_action.action_type, ActionType::BarrierBreach);
        assert!(result.apply_result.move_count() >= 1);
    }

    #[test]
    fn test_booking_patterns() {
        let product = create_ko_product("KO-001");
        let trigger = create_barrier_trigger(Decimal::new(8950, 2), Decimal::new(9000, 2));
        let rules = create_thin_slice_rules();

        // Test DirectBooking
        let position1 = create_test_position("POS-001", &product, Decimal::new(100, 0));
        let context1 = MoveRuleContext::new()
            .with_attribute("product_type", "KnockOutWarrant")
            .with_attribute("booking_type", "standard");
        let pos_ctx1 = PositionWithContext::new(position1, context1);

        let result1 = execute_two_call_flow(&trigger, &product, &[pos_ctx1], &rules, 1)
            .expect("DirectBooking should succeed");

        assert!(result1.apply_result.move_count() >= 1);
        let mov1 = &result1.apply_result.moves[0];
        assert_eq!(mov1.debit_account.as_str(), "POSITION_CASH");
        assert_eq!(mov1.credit_account.as_str(), "SETTLEMENT_ACCOUNT");
    }

    #[test]
    fn test_json_serialization() {
        let product = create_ko_product("KO-001");
        let position = create_test_position("POS-001", &product, Decimal::new(100, 0));
        let trigger = create_barrier_trigger(Decimal::new(8950, 2), Decimal::new(9000, 2));

        let context = MoveRuleContext::new()
            .with_attribute("product_type", "KnockOutWarrant")
            .with_attribute("booking_type", "standard");

        let rules = create_thin_slice_rules();
        let pos_with_ctx = PositionWithContext::new(position, context);

        let result = execute_two_call_flow(&trigger, &product, &[pos_with_ctx], &rules, 1)
            .expect("Should succeed");

        let json = result
            .apply_result
            .to_json()
            .expect("JSON serialization should succeed");
        assert!(!json.is_empty());
        assert!(json.contains("moves"));

        let deserialized: ApplyResult =
            serde_json::from_str(&json).expect("JSON deserialization should succeed");
        assert_eq!(deserialized.move_count(), result.apply_result.move_count());
    }

    #[test]
    fn test_protobuf_serialization() {
        let product = create_ko_product("KO-001");
        let position = create_test_position("POS-001", &product, Decimal::new(100, 0));
        let trigger = create_barrier_trigger(Decimal::new(8950, 2), Decimal::new(9000, 2));

        let context = MoveRuleContext::new()
            .with_attribute("product_type", "KnockOutWarrant")
            .with_attribute("booking_type", "standard");

        let rules = create_thin_slice_rules();
        let pos_with_ctx = PositionWithContext::new(position, context);

        let result = execute_two_call_flow(&trigger, &product, &[pos_with_ctx], &rules, 1)
            .expect("Should succeed");

        let proto_bytes = result.apply_result.encode_proto();
        assert!(!proto_bytes.is_empty());

        let decoded = ApplyResultProto::decode(&proto_bytes[..])
            .expect("Protobuf decoding should succeed");
        assert_eq!(decoded.moves.len(), result.apply_result.move_count());
    }
}
