//! Integration Tests for the Quantitative Event Engine.
//!
//! This module contains comprehensive integration tests that validate end-to-end
//! workflows across multiple components:
//! - End-to-end two-call pattern flows
//! - Event chaining scenarios with lineage tracking
//! - Multi-pattern booking scenarios
//! - Portfolio-scoped move queries
//! - Serialization roundtrips (JSON and Protobuf)
//! - Stub orchestration flows

use crate::ledger::actions::{ActionDetails, ActionType, PositionSelector, SettlementReason, SettlementType};
use crate::ledger::apply_result::{ApplyResult, PositionWithContext};
use crate::ledger::domain::{Instrument, Product, ProductDetails};
use crate::ledger::in_memory::{InMemoryLedger, Wallet};
use crate::ledger::move_rules::{BookingPattern, MoveRule, MoveRuleContext, RuleCondition};
use crate::ledger::stubs::{
    EventFramework, EventFrameworkStub, QuantLibStub, QuantLibTrait,
    create_thin_slice_rules, execute_two_call_flow, ApplyResultProto,
};
use crate::ledger::triggers::{BarrierDirection, FixingType, TriggerInfo, TriggerType};
use crate::ledger::types::{
    LedgerReader, LedgerWriter, Move, MoveType, PortfolioFilter, Position, TagCriterion,
};
use chrono::{NaiveDate, TimeZone, Utc};
use prost::Message;
use rust_decimal::Decimal;

// =========================================================================
// Helper Functions
// =========================================================================

fn create_ko_product(id: &str) -> Product {
    Product::KnockOutWarrant(ProductDetails::new(id, "Test KO Warrant", "AAPL", "USD"))
}

fn create_mini_product(id: &str) -> Product {
    Product::MiniCertificate(ProductDetails::new(id, "Test Mini Cert", "DAX", "EUR"))
}

fn create_test_position(
    id: &str,
    wallet_id: &str,
    product: &Product,
    quantity: Decimal,
    tags: Vec<(&str, &str)>,
) -> Position {
    let instrument = Instrument::Product(product.clone());
    let mut position = Position::new(id, wallet_id, instrument, quantity);
    for (key, value) in tags {
        position = position.with_tag(key, value);
    }
    position
}

fn create_barrier_trigger(breach_price: Decimal, barrier_level: Decimal) -> TriggerInfo {
    TriggerInfo::new_barrier(
        true, // continuous
        Utc.with_ymd_and_hms(2024, 12, 15, 10, 30, 0).unwrap(),
        breach_price,
        barrier_level,
        BarrierDirection::Down,
    )
}

fn create_fixing_trigger(fixing_type: FixingType, observed_price: Decimal) -> TriggerInfo {
    TriggerInfo::new_fixing(
        fixing_type,
        Utc.with_ymd_and_hms(2024, 12, 15, 9, 0, 0).unwrap(),
        observed_price,
    )
}

fn create_expiry_trigger(settlement_price: Decimal) -> TriggerInfo {
    let mut trigger = TriggerInfo::new_time_based(
        TriggerType::Expiry,
        Utc.with_ymd_and_hms(2024, 12, 20, 17, 30, 0).unwrap(),
    );
    trigger.observed_price = Some(settlement_price);
    trigger
}

// =========================================================================
// Integration Test 1: End-to-end Continuous Barrier Breach -> Moves + Settlement
// =========================================================================

/// Test the complete flow: continuous barrier breach triggers moves and
/// downstream settlement action, validating the full event chain.
#[test]
fn test_e2e_continuous_barrier_breach_with_settlement_chain() {
    let product = create_ko_product("KO-INTEG-001");

    // Create position with comprehensive tags for portfolio tracking
    let position = create_test_position(
        "POS-INTEG-001",
        "WALLET-001",
        &product,
        Decimal::new(100, 0),
        vec![
            ("desk", "desk_a"),
            ("strategy", "leverage"),
            ("client", "institutional"),
            ("booking_type", "standard"),
        ],
    );

    // Setup context for rule matching
    let context = MoveRuleContext::new()
        .with_attribute("product_type", "KnockOutWarrant")
        .with_attribute("booking_type", "standard")
        .with_attribute("desk", "desk_a");

    let rules = create_thin_slice_rules();
    let pos_with_ctx = PositionWithContext::new(position.clone(), context);

    // Barrier at 90.00, breached at 89.50
    let trigger = create_barrier_trigger(Decimal::new(8950, 2), Decimal::new(9000, 2));

    // Execute full two-call flow with event chaining
    let result = execute_two_call_flow(&trigger, &product, &[pos_with_ctx], &rules, 2)
        .expect("Two-call flow should succeed");

    // Validate Call 1: BarrierBreach action produced
    assert_eq!(
        result.product_action.action_type,
        ActionType::BarrierBreach,
        "Call 1 should produce BarrierBreach action"
    );
    assert_eq!(
        result.product_action.amount_per_unit,
        Some(Decimal::new(8950, 2)),
        "Settlement amount should be breach price"
    );

    // Validate barrier details
    if let ActionDetails::Barrier {
        barrier_level,
        breach_price,
        is_knockout,
    } = &result.product_action.details
    {
        assert_eq!(*barrier_level, Decimal::new(9000, 2));
        assert_eq!(*breach_price, Decimal::new(8950, 2));
        assert!(*is_knockout);
    } else {
        panic!("Expected Barrier details in ProductAction");
    }

    // Validate Call 2: Moves generated
    assert!(
        result.apply_result.move_count() >= 1,
        "Call 2 should generate moves"
    );

    // Validate move amounts: 100 units * 89.50 = 8950.00
    let barrier_moves = &result.apply_result.moves;
    let total_barrier_amount: Decimal = barrier_moves.iter().map(|m| m.amount).sum();
    assert_eq!(
        total_barrier_amount,
        Decimal::new(895000, 2),
        "Barrier moves should total 8950.00"
    );

    // Validate event chaining: downstream settlement
    assert!(
        result.apply_result.has_downstream_actions(),
        "BarrierBreach should trigger downstream Settlement"
    );
    assert_eq!(result.apply_result.downstream_action_count(), 1);

    let settlement_action = &result.apply_result.downstream_actions[0];
    assert_eq!(settlement_action.action_type, ActionType::Settlement);
    assert!(
        settlement_action.source_action_ref.is_some(),
        "Settlement should have lineage reference"
    );

    // Validate downstream results processed
    assert!(
        !result.downstream_results.is_empty(),
        "Downstream settlement should be processed"
    );

    // Validate total moves across entire chain
    let total_moves = result.total_moves();
    assert!(
        total_moves >= 2,
        "Should have moves from both barrier and settlement"
    );

    // Validate audit trail on all moves
    for mov in result.all_moves() {
        assert!(
            !mov.source_action.as_str().is_empty(),
            "Move should have source action reference"
        );
        assert!(
            !mov.rule_applied.as_str().is_empty(),
            "Move should have rule reference"
        );
    }
}

// =========================================================================
// Integration Test 2: End-to-end Fixing -> Settlement Flow
// =========================================================================

/// Test the complete fixing to settlement lifecycle flow.
#[test]
fn test_e2e_fixing_to_settlement_flow() {
    let product = create_ko_product("KO-FIXING-001");

    let position = create_test_position(
        "POS-FIXING-001",
        "WALLET-001",
        &product,
        Decimal::new(200, 0),
        vec![("desk", "desk_b"), ("booking_type", "standard")],
    );

    let context = MoveRuleContext::new()
        .with_attribute("product_type", "KnockOutWarrant")
        .with_attribute("booking_type", "standard");

    let rules = create_thin_slice_rules();
    let pos_with_ctx = PositionWithContext::new(position.clone(), context.clone());

    // Step 1: Initial Fixing at 150.00 (sets strike)
    let initial_fixing = create_fixing_trigger(FixingType::Initial, Decimal::new(15000, 2));

    let initial_result =
        execute_two_call_flow(&initial_fixing, &product, &[pos_with_ctx.clone()], &rules, 1)
            .expect("Initial fixing should succeed");

    assert_eq!(
        initial_result.product_action.action_type,
        ActionType::FixingObserved
    );
    if let ActionDetails::Fixing {
        observed_price,
        fixing_type,
        underlying,
    } = &initial_result.product_action.details
    {
        assert_eq!(*observed_price, Decimal::new(15000, 2));
        assert_eq!(fixing_type, "Initial");
        assert_eq!(underlying, "AAPL");
    } else {
        panic!("Expected Fixing details");
    }

    // Step 2: Final Settlement at expiry with price 175.00
    let expiry_trigger = create_expiry_trigger(Decimal::new(17500, 2));

    let expiry_result =
        execute_two_call_flow(&expiry_trigger, &product, &[pos_with_ctx], &rules, 1)
            .expect("Expiry settlement should succeed");

    // Validate Settlement action
    assert_eq!(
        expiry_result.product_action.action_type,
        ActionType::Settlement
    );
    assert_eq!(
        expiry_result.product_action.amount_per_unit,
        Some(Decimal::new(17500, 2))
    );

    // Validate settlement details
    if let ActionDetails::Settlement {
        settlement_type,
        reason,
    } = &expiry_result.product_action.details
    {
        assert_eq!(*settlement_type, SettlementType::Cash);
        assert_eq!(*reason, SettlementReason::Maturity);
    } else {
        panic!("Expected Settlement details");
    }

    // Validate moves: 200 units * 175.00 = 35000.00
    assert!(expiry_result.apply_result.move_count() >= 1);
    let settlement_mov = &expiry_result.apply_result.moves[0];
    assert_eq!(settlement_mov.amount, Decimal::new(3500000, 2));
}

// =========================================================================
// Integration Test 3: Two-Call Pattern with 3 Different Booking Patterns
// =========================================================================

/// Test that all 3 booking patterns (DirectBooking, WashBookRouting, Split)
/// work correctly in a single integrated scenario.
#[test]
fn test_two_call_pattern_with_three_booking_patterns() {
    let product = create_ko_product("KO-PATTERNS-001");
    let trigger = create_barrier_trigger(Decimal::new(8950, 2), Decimal::new(9000, 2));
    let rules = create_thin_slice_rules();

    // Position 1: DirectBooking (standard)
    let pos1 = create_test_position(
        "POS-DIRECT-001",
        "WALLET-001",
        &product,
        Decimal::new(100, 0),
        vec![("booking_type", "standard")],
    );
    let ctx1 = MoveRuleContext::new()
        .with_attribute("product_type", "KnockOutWarrant")
        .with_attribute("booking_type", "standard");
    let pos_ctx1 = PositionWithContext::new(pos1, ctx1);

    // Position 2: WashBookRouting (intercompany)
    let pos2 = create_test_position(
        "POS-WASH-001",
        "WALLET-001",
        &product,
        Decimal::new(50, 0),
        vec![("booking_type", "intercompany")],
    );
    let ctx2 = MoveRuleContext::new()
        .with_attribute("product_type", "KnockOutWarrant")
        .with_attribute("booking_type", "intercompany");
    let pos_ctx2 = PositionWithContext::new(pos2, ctx2);

    // Position 3: Split (multi_desk)
    let pos3 = create_test_position(
        "POS-SPLIT-001",
        "WALLET-001",
        &product,
        Decimal::new(200, 0),
        vec![("booking_type", "multi_desk")],
    );
    let ctx3 = MoveRuleContext::new()
        .with_attribute("product_type", "KnockOutWarrant")
        .with_attribute("booking_type", "multi_desk");
    let pos_ctx3 = PositionWithContext::new(pos3, ctx3);

    // Execute with all 3 positions
    let all_positions = vec![pos_ctx1, pos_ctx2, pos_ctx3];
    let result = execute_two_call_flow(&trigger, &product, &all_positions, &rules, 1)
        .expect("Multi-pattern flow should succeed");

    // Count moves by pattern
    // DirectBooking: 1 move (position 1)
    // WashBookRouting: 2 moves (position 2)
    // Split: 2 moves (position 3)
    // Total: 5 moves minimum
    assert!(
        result.apply_result.move_count() >= 5,
        "Should have at least 5 moves from 3 patterns"
    );

    // Validate amounts by position
    let moves = &result.apply_result.moves;

    // DirectBooking: 100 * 89.50 = 8950.00
    let direct_moves: Vec<_> = moves
        .iter()
        .filter(|m| m.position_id.as_str() == "POS-DIRECT-001")
        .collect();
    assert_eq!(direct_moves.len(), 1, "DirectBooking should produce 1 move");
    assert_eq!(direct_moves[0].amount, Decimal::new(895000, 2));

    // WashBookRouting: 50 * 89.50 = 4475.00 (2 legs, same amount)
    let wash_moves: Vec<_> = moves
        .iter()
        .filter(|m| m.position_id.as_str() == "POS-WASH-001")
        .collect();
    assert_eq!(
        wash_moves.len(),
        2,
        "WashBookRouting should produce 2 moves"
    );
    assert_eq!(wash_moves[0].amount, Decimal::new(447500, 2));
    assert_eq!(wash_moves[1].amount, Decimal::new(447500, 2));

    // Validate WashBook accounts
    assert_eq!(wash_moves[0].debit_account.as_str(), "ENTITY_A_CASH");
    assert_eq!(wash_moves[0].credit_account.as_str(), "INTERCOMPANY_WASH");
    assert_eq!(wash_moves[1].debit_account.as_str(), "INTERCOMPANY_WASH");
    assert_eq!(wash_moves[1].credit_account.as_str(), "ENTITY_B_CASH");

    // Split: 200 * 89.50 = 17900.00 split 60/40
    let split_moves: Vec<_> = moves
        .iter()
        .filter(|m| m.position_id.as_str() == "POS-SPLIT-001")
        .collect();
    assert_eq!(split_moves.len(), 2, "Split should produce 2 moves");
    let split_total: Decimal = split_moves.iter().map(|m| m.amount).sum();
    assert_eq!(
        split_total,
        Decimal::new(1790000, 2),
        "Split total should be 17900.00"
    );
}

// =========================================================================
// Integration Test 4: Event Chaining with Lineage Tracking
// =========================================================================

/// Test complete event chaining with full lineage verification:
/// BarrierBreach -> Settlement -> (HedgeUnwind if positions exist)
#[test]
fn test_event_chaining_with_lineage_tracking() {
    let product = create_ko_product("KO-LINEAGE-001");
    let position = create_test_position(
        "POS-LINEAGE-001",
        "WALLET-001",
        &product,
        Decimal::new(100, 0),
        vec![("desk", "desk_a"), ("booking_type", "standard")],
    );

    let context = MoveRuleContext::new()
        .with_attribute("product_type", "KnockOutWarrant")
        .with_attribute("booking_type", "standard");

    let rules = create_thin_slice_rules();
    let pos_with_ctx = PositionWithContext::new(position, context);

    let trigger = create_barrier_trigger(Decimal::new(8950, 2), Decimal::new(9000, 2));

    let result = execute_two_call_flow(&trigger, &product, &[pos_with_ctx], &rules, 3)
        .expect("Lineage flow should succeed");

    // Level 0: Root action (BarrierBreach)
    let root_action = &result.product_action;
    assert_eq!(root_action.action_type, ActionType::BarrierBreach);
    assert!(
        root_action.source_action_ref.is_none(),
        "Root action should have no source"
    );
    assert_eq!(root_action.lineage_depth(), 0);

    // Level 1: Downstream Settlement
    assert!(result.apply_result.has_downstream_actions());
    let settlement_action = &result.apply_result.downstream_actions[0];

    assert_eq!(settlement_action.action_type, ActionType::Settlement);
    assert!(
        settlement_action.source_action_ref.is_some(),
        "Settlement should reference barrier action"
    );
    assert!(settlement_action.is_chained());
    assert_eq!(settlement_action.lineage_depth(), 1);

    // Verify Settlement details
    if let ActionDetails::Settlement {
        settlement_type,
        reason,
    } = &settlement_action.details
    {
        assert_eq!(*settlement_type, SettlementType::Cash);
        assert_eq!(*reason, SettlementReason::BarrierBreach);
    }

    // Verify PositionSelector on downstream
    assert_eq!(
        settlement_action.target_selector,
        Some(PositionSelector::SameAsSource)
    );

    // Verify all moves have lineage
    for mov in result.all_moves() {
        assert!(
            !mov.source_action.as_str().is_empty(),
            "All moves should have source action for audit"
        );
    }
}

// =========================================================================
// Integration Test 5: Portfolio-Scoped Move Queries
// =========================================================================

/// Test that moves can be queried and attributed by portfolio dimensions
/// after being recorded in the ledger.
#[test]
fn test_portfolio_scoped_move_queries() {
    let mut ledger = InMemoryLedger::new();

    // Use a consistent wallet ID for all positions
    let wallet_id = "WALLET-PORTFOLIO";

    // Create wallet with default tags
    let wallet = Wallet::new(wallet_id, "Portfolio Test Wallet")
        .with_default_tag("book", "trading")
        .with_default_tag("regulatory_book", "FRTB");
    ledger.create_wallet(wallet).unwrap();

    // Create products
    let ko_product = create_ko_product("KO-PORT-001");
    let mini_product = create_mini_product("MINI-PORT-001");

    // Create positions with portfolio wallet (to inherit tags)
    let pos1 = create_test_position(
        "POS-PORT-001",
        wallet_id,
        &ko_product,
        Decimal::new(100, 0),
        vec![("desk", "desk_a"), ("strategy", "leverage")],
    );
    let pos2 = create_test_position(
        "POS-PORT-002",
        wallet_id,
        &ko_product,
        Decimal::new(200, 0),
        vec![("desk", "desk_b"), ("strategy", "leverage")],
    );
    let pos3 = create_test_position(
        "POS-PORT-003",
        wallet_id,
        &mini_product,
        Decimal::new(150, 0),
        vec![("desk", "desk_a"), ("strategy", "income")],
    );

    // Create positions with wallet tag inheritance
    let pos1_id = ledger.create_position_with_inheritance(pos1).unwrap();
    let pos2_id = ledger.create_position_with_inheritance(pos2).unwrap();
    let pos3_id = ledger.create_position_with_inheritance(pos3).unwrap();

    // Verify positions have inherited tags
    let retrieved_pos1 = ledger.get_position(&pos1_id).unwrap();
    assert_eq!(
        retrieved_pos1.tags.get("book"),
        Some(&"trading".to_string()),
        "Position should inherit 'book' tag from wallet"
    );

    // Create and record moves with tag inheritance
    let effective_date = NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
    let moves = vec![
        Move::new(
            "MOV-001",
            pos1_id.clone(),
            MoveType::Settlement,
            "CASH",
            "PRODUCT",
            Decimal::new(100000, 2), // 1000.00
            effective_date,
            "ACTION-001",
            "RULE-001",
        ),
        Move::new(
            "MOV-002",
            pos2_id.clone(),
            MoveType::Settlement,
            "CASH",
            "PRODUCT",
            Decimal::new(200000, 2), // 2000.00
            effective_date,
            "ACTION-002",
            "RULE-001",
        ),
        Move::new(
            "MOV-003",
            pos3_id.clone(),
            MoveType::Settlement,
            "CASH",
            "PRODUCT",
            Decimal::new(150000, 2), // 1500.00
            effective_date,
            "ACTION-003",
            "RULE-001",
        ),
    ];

    ledger.record_moves_with_inheritance(moves).unwrap();

    // Query by desk
    let desk_a_filter = PortfolioFilter::new("Desk A P&L")
        .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()));
    let desk_a_moves = ledger.moves_by_portfolio(&desk_a_filter).unwrap();
    assert_eq!(desk_a_moves.len(), 2, "Desk A should have 2 moves");
    let desk_a_total: Decimal = desk_a_moves.iter().map(|m| m.amount).sum();
    assert_eq!(desk_a_total, Decimal::new(250000, 2)); // 1000 + 1500 = 2500

    // Query by strategy
    let leverage_filter = PortfolioFilter::new("Leverage Strategy P&L").with_criterion(
        TagCriterion::Equals("strategy".to_string(), "leverage".to_string()),
    );
    let leverage_moves = ledger.moves_by_portfolio(&leverage_filter).unwrap();
    assert_eq!(leverage_moves.len(), 2, "Leverage strategy should have 2 moves");
    let leverage_total: Decimal = leverage_moves.iter().map(|m| m.amount).sum();
    assert_eq!(leverage_total, Decimal::new(300000, 2)); // 1000 + 2000 = 3000

    // Query by inherited book tag
    let trading_book_filter = PortfolioFilter::new("Trading Book P&L")
        .with_criterion(TagCriterion::Equals("book".to_string(), "trading".to_string()));
    let trading_moves = ledger.moves_by_portfolio(&trading_book_filter).unwrap();
    assert_eq!(
        trading_moves.len(),
        3,
        "All moves should have inherited trading book tag"
    );

    // Combined filter: desk_a AND leverage
    let combined_filter = PortfolioFilter::new("Desk A Leverage")
        .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()))
        .with_criterion(TagCriterion::Equals(
            "strategy".to_string(),
            "leverage".to_string(),
        ));
    let combined_moves = ledger.moves_by_portfolio(&combined_filter).unwrap();
    assert_eq!(combined_moves.len(), 1);
    assert_eq!(combined_moves[0].amount, Decimal::new(100000, 2));
}

// =========================================================================
// Integration Test 6: Full ApplyResult JSON Roundtrip
// =========================================================================

/// Test complete JSON serialization and deserialization of ApplyResult
/// with moves and downstream actions.
#[test]
fn test_apply_result_json_roundtrip() {
    let product = create_ko_product("KO-JSON-001");
    let position = create_test_position(
        "POS-JSON-001",
        "WALLET-001",
        &product,
        Decimal::new(100, 0),
        vec![
            ("desk", "desk_a"),
            ("strategy", "leverage"),
            ("booking_type", "standard"),
        ],
    );

    let context = MoveRuleContext::new()
        .with_attribute("product_type", "KnockOutWarrant")
        .with_attribute("booking_type", "standard");

    let rules = create_thin_slice_rules();
    let pos_with_ctx = PositionWithContext::new(position, context);

    let trigger = create_barrier_trigger(Decimal::new(8950, 2), Decimal::new(9000, 2));

    let result = execute_two_call_flow(&trigger, &product, &[pos_with_ctx], &rules, 1)
        .expect("Flow should succeed");

    let original = &result.apply_result;

    // Serialize to JSON
    let json = original.to_json().expect("JSON serialization should succeed");
    assert!(!json.is_empty());
    assert!(json.contains("moves"));
    assert!(json.contains("downstream_actions"));

    // Verify JSON contains expected data - Decimal serializes as "8950.00" not "895000"
    assert!(
        json.contains("POSITION_CASH") || json.contains("CASH"),
        "JSON should contain account references"
    );
    // Check for the decimal value in JSON (Decimal serializes with decimal point)
    assert!(
        json.contains("8950.00") || json.contains("8950"),
        "JSON should contain move amount in some form"
    );

    // Deserialize back
    let deserialized: ApplyResult =
        serde_json::from_str(&json).expect("JSON deserialization should succeed");

    // Validate structure preserved
    assert_eq!(
        deserialized.move_count(),
        original.move_count(),
        "Move count should be preserved"
    );
    assert_eq!(
        deserialized.downstream_action_count(),
        original.downstream_action_count(),
        "Downstream action count should be preserved"
    );

    // Validate move details preserved
    for (orig_mov, deser_mov) in original.moves.iter().zip(deserialized.moves.iter()) {
        assert_eq!(orig_mov.id, deser_mov.id);
        assert_eq!(orig_mov.position_id, deser_mov.position_id);
        assert_eq!(orig_mov.amount, deser_mov.amount);
        assert_eq!(orig_mov.move_type, deser_mov.move_type);
        assert_eq!(orig_mov.debit_account, deser_mov.debit_account);
        assert_eq!(orig_mov.credit_account, deser_mov.credit_account);
    }

    // Validate downstream actions preserved
    for (orig_action, deser_action) in original
        .downstream_actions
        .iter()
        .zip(deserialized.downstream_actions.iter())
    {
        assert_eq!(orig_action.action_type, deser_action.action_type);
        assert_eq!(orig_action.product_id, deser_action.product_id);
        assert_eq!(orig_action.source_action_ref, deser_action.source_action_ref);
    }

    // Test pretty JSON
    let pretty_json = original
        .to_json_pretty()
        .expect("Pretty JSON should succeed");
    assert!(pretty_json.contains('\n'), "Pretty JSON should be formatted");
}

// =========================================================================
// Integration Test 7: Full ApplyResult Protobuf Roundtrip
// =========================================================================

/// Test complete Protobuf serialization and deserialization of ApplyResult.
#[test]
fn test_apply_result_protobuf_roundtrip() {
    let product = create_ko_product("KO-PROTO-001");
    let position = create_test_position(
        "POS-PROTO-001",
        "WALLET-001",
        &product,
        Decimal::new(100, 0),
        vec![("desk", "desk_a"), ("booking_type", "standard")],
    );

    let context = MoveRuleContext::new()
        .with_attribute("product_type", "KnockOutWarrant")
        .with_attribute("booking_type", "standard");

    let rules = create_thin_slice_rules();
    let pos_with_ctx = PositionWithContext::new(position, context);

    let trigger = create_barrier_trigger(Decimal::new(8950, 2), Decimal::new(9000, 2));

    let result = execute_two_call_flow(&trigger, &product, &[pos_with_ctx], &rules, 1)
        .expect("Flow should succeed");

    let original = &result.apply_result;

    // Encode to Protobuf
    let proto_bytes = original.encode_proto();
    assert!(!proto_bytes.is_empty(), "Protobuf encoding should produce bytes");

    // Decode Protobuf
    let decoded =
        ApplyResultProto::decode(&proto_bytes[..]).expect("Protobuf decoding should succeed");

    // Validate structure
    assert_eq!(
        decoded.moves.len(),
        original.move_count(),
        "Move count should match"
    );
    assert_eq!(
        decoded.downstream_action_count as usize,
        original.downstream_action_count(),
        "Downstream action count should match"
    );

    // Validate move proto fields
    for (orig_mov, proto_mov) in original.moves.iter().zip(decoded.moves.iter()) {
        assert_eq!(proto_mov.id, orig_mov.id.as_str());
        assert_eq!(proto_mov.position_id, orig_mov.position_id.as_str());
        assert_eq!(proto_mov.move_type, orig_mov.move_type as i32);
        assert_eq!(proto_mov.debit_account, orig_mov.debit_account.as_str());
        assert_eq!(proto_mov.credit_account, orig_mov.credit_account.as_str());
        assert_eq!(proto_mov.amount, orig_mov.amount.to_string());
    }
}

// =========================================================================
// Integration Test 8: Complete Stub Orchestration Trigger -> Moves Flow
// =========================================================================

/// Test the complete EventFrameworkStub orchestration from trigger to moves,
/// validating the full two-call pattern through the stub infrastructure.
#[test]
fn test_stub_orchestration_complete_flow() {
    let product = create_ko_product("KO-STUB-001");
    let position = create_test_position(
        "POS-STUB-001",
        "WALLET-001",
        &product,
        Decimal::new(100, 0),
        vec![("desk", "desk_a"), ("booking_type", "standard")],
    );

    // Create comprehensive move rules
    let ko_rule = MoveRule::new(
        "RULE-KO-STUB",
        "KO Standard Settlement",
        BookingPattern::direct("POSITION_CASH", "SETTLEMENT_ACCOUNT"),
    )
    .with_condition(RuleCondition::equals("product_type", "KnockOutWarrant"))
    .with_move_type(MoveType::Settlement)
    .with_priority(100);

    let default_rule = MoveRule::new(
        "RULE-DEFAULT-STUB",
        "Default Settlement",
        BookingPattern::direct("CASH", "SETTLEMENT"),
    )
    .with_move_type(MoveType::Settlement)
    .with_priority(0);

    let default_context = MoveRuleContext::new()
        .with_attribute("booking_type", "standard")
        .with_attribute("legal_entity", "ENTITY_A");

    // Build the EventFrameworkStub
    let framework = EventFrameworkStub::new()
        .with_max_depth(3)
        .with_product(product.clone())
        .with_position(position)
        .with_rule(ko_rule)
        .with_rule(default_rule)
        .with_default_context(default_context);

    // Create barrier trigger
    let trigger_info = TriggerInfo::new_barrier(
        true,
        Utc.with_ymd_and_hms(2024, 12, 15, 10, 30, 0).unwrap(),
        Decimal::new(8950, 2),
        Decimal::new(9000, 2),
        BarrierDirection::Down,
    );

    // Execute simulation
    let result = framework
        .simulate_trigger(trigger_info)
        .expect("Simulation should succeed");

    // Validate simulation result structure
    assert!(result.total_moves > 0, "Should have generated moves");
    assert!(
        !result.actions_by_depth.is_empty(),
        "Should have actions at depth 0"
    );
    assert!(!result.results_by_depth.is_empty(), "Should have results");

    // Validate depth 0 (BarrierBreach)
    let depth0_actions = &result.actions_by_depth[0];
    assert_eq!(depth0_actions.len(), 1);
    assert_eq!(depth0_actions[0].action_type, ActionType::BarrierBreach);

    // Validate moves generated
    let all_moves = result.all_moves();
    assert!(!all_moves.is_empty());

    // Validate move details
    let first_move = &all_moves[0];
    assert_eq!(first_move.debit_account.as_str(), "POSITION_CASH");
    assert_eq!(first_move.credit_account.as_str(), "SETTLEMENT_ACCOUNT");
    assert_eq!(first_move.amount, Decimal::new(895000, 2)); // 100 * 89.50

    // Validate event chaining occurred (depth > 0)
    assert!(
        result.max_depth_reached >= 1,
        "Should have processed downstream actions"
    );

    // Validate no recursion limit hit (with max_depth=3, barrier->settlement should be fine)
    assert!(
        !result.recursion_limited || result.max_depth_reached < 3,
        "Should not hit recursion limit with simple chain"
    );

    // Test QuantLibStub directly for all trigger types
    let quant_lib = QuantLibStub::new();

    // Test fixing trigger
    let fixing_trigger = create_fixing_trigger(FixingType::Final, Decimal::new(17500, 2));
    let fixing_action = quant_lib
        .get_action(&product, &fixing_trigger)
        .expect("Fixing should succeed");
    assert_eq!(fixing_action.action_type, ActionType::FixingObserved);

    // Test expiry trigger
    let expiry_trigger = create_expiry_trigger(Decimal::new(17500, 2));
    let expiry_action = quant_lib
        .get_action(&product, &expiry_trigger)
        .expect("Expiry should succeed");
    assert_eq!(expiry_action.action_type, ActionType::Settlement);

    // Test coupon trigger
    let coupon_trigger = TriggerInfo::new_time_based(
        TriggerType::CouponPayment,
        Utc.with_ymd_and_hms(2024, 6, 15, 17, 0, 0).unwrap(),
    );
    let coupon_action = quant_lib
        .get_action(&product, &coupon_trigger)
        .expect("Coupon should succeed");
    assert_eq!(coupon_action.action_type, ActionType::CouponPayment);
}
