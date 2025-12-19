//! Stub implementation of the Event Framework for testing.
//!
//! The Event Framework is responsible for:
//! - Trigger registration and detection
//! - Position lookup
//! - MoveRule matching
//! - Orchestrating the two-call pattern
//!
//! This stub provides a test/demo implementation that simulates the full flow.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::ledger::actions::{ActionType, ProductAction};
use crate::ledger::apply_result::{ApplyResult, DefaultMoveApplicator, MoveApplicator, PositionWithContext};
use crate::ledger::domain::{Product, ProductId};
use crate::ledger::move_rules::{MoveRule, MoveRuleContext};
use crate::ledger::triggers::{TriggerInfo, TriggerType};
use crate::ledger::types::Position;

use super::quant_lib::{QuantLibStub, QuantLibTrait};

/// Trigger registration for the Event Framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerRegistration {
    /// Product ID this trigger is for
    pub product_id: ProductId,
    /// Type of trigger
    pub trigger_type: TriggerType,
    /// Trigger parameters (e.g., barrier level)
    pub parameters: HashMap<String, String>,
}

impl TriggerRegistration {
    /// Create a new trigger registration.
    pub fn new(product_id: ProductId, trigger_type: TriggerType) -> Self {
        TriggerRegistration {
            product_id,
            trigger_type,
            parameters: HashMap::new(),
        }
    }

    /// Add a parameter.
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

/// Error type for Event Framework operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EventFrameworkError {
    #[error("Product not found: {0}")]
    ProductNotFound(String),

    #[error("No positions found for product: {0}")]
    NoPositionsFound(String),

    #[error("Max recursion depth exceeded: {0}")]
    MaxRecursionDepth(usize),

    #[error("Cycle detected in action chain: {action_type} for product {product_id}")]
    CycleDetected { action_type: String, product_id: String },

    #[error("Action error: {0}")]
    ActionError(String),

    #[error("Apply error: {0}")]
    ApplyError(String),
}

/// Result type for Event Framework operations.
pub type EventFrameworkResult<T> = Result<T, EventFrameworkError>;

/// Event Framework trait for trigger management and orchestration.
///
/// The Event Framework is responsible for:
/// - Trigger registration and detection
/// - Position lookup
/// - MoveRule matching
/// - Orchestrating the two-call pattern
pub trait EventFramework: Send + Sync {
    /// Register a trigger for a product.
    fn register_trigger(&mut self, registration: TriggerRegistration) -> EventFrameworkResult<()>;

    /// Simulate a trigger firing and process the full two-call flow.
    fn simulate_trigger(&self, trigger_info: TriggerInfo) -> EventFrameworkResult<SimulationResult>;
}

/// Result of simulating a trigger through the Event Framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Product actions generated at each depth level
    pub actions_by_depth: Vec<Vec<ProductAction>>,
    /// Apply results at each depth level
    pub results_by_depth: Vec<ApplyResult>,
    /// Total number of moves generated
    pub total_moves: usize,
    /// Maximum depth reached
    pub max_depth_reached: usize,
    /// Whether recursion limit was hit
    pub recursion_limited: bool,
}

impl SimulationResult {
    /// Create a new empty simulation result.
    pub fn new() -> Self {
        SimulationResult {
            actions_by_depth: Vec::new(),
            results_by_depth: Vec::new(),
            total_moves: 0,
            max_depth_reached: 0,
            recursion_limited: false,
        }
    }

    /// Get all moves flattened.
    pub fn all_moves(&self) -> Vec<crate::ledger::types::Move> {
        self.results_by_depth
            .iter()
            .flat_map(|r| r.moves.clone())
            .collect()
    }
}

impl Default for SimulationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Stub implementation of the Event Framework for testing.
///
/// Provides:
/// - Trigger registration storage
/// - Mock position lookup
/// - MoveRuleContext building
/// - Two-call pattern orchestration
/// - Recursive downstream action processing with safety limits
pub struct EventFrameworkStub {
    /// Registered triggers by product ID
    pub(crate) trigger_registrations: HashMap<ProductId, Vec<TriggerRegistration>>,
    /// Products available for simulation
    products: HashMap<ProductId, Product>,
    /// Positions available for simulation
    positions: Vec<Position>,
    /// Move rules for pattern matching
    move_rules: Vec<MoveRule>,
    /// Default context attributes
    default_context: MoveRuleContext,
    /// Maximum recursion depth for event chaining
    max_depth: usize,
    /// QuantLib stub for Call 1
    quant_lib: QuantLibStub,
}

impl EventFrameworkStub {
    /// Create a new EventFrameworkStub with default max depth of 10.
    pub fn new() -> Self {
        EventFrameworkStub {
            trigger_registrations: HashMap::new(),
            products: HashMap::new(),
            positions: Vec::new(),
            move_rules: Vec::new(),
            default_context: MoveRuleContext::new(),
            max_depth: 10,
            quant_lib: QuantLibStub::new(),
        }
    }

    /// Set the maximum recursion depth.
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Add a product for simulation.
    pub fn with_product(mut self, product: Product) -> Self {
        self.products.insert(product.id(), product);
        self
    }

    /// Add a position for simulation.
    pub fn with_position(mut self, position: Position) -> Self {
        self.positions.push(position);
        self
    }

    /// Add multiple positions for simulation.
    pub fn with_positions(mut self, positions: Vec<Position>) -> Self {
        self.positions.extend(positions);
        self
    }

    /// Add a move rule for pattern matching.
    pub fn with_rule(mut self, rule: MoveRule) -> Self {
        self.move_rules.push(rule);
        self
    }

    /// Add multiple move rules.
    pub fn with_rules(mut self, rules: Vec<MoveRule>) -> Self {
        self.move_rules.extend(rules);
        self
    }

    /// Set default context attributes.
    pub fn with_default_context(mut self, context: MoveRuleContext) -> Self {
        self.default_context = context;
        self
    }

    /// Set the QuantLib stub.
    pub fn with_quant_lib(mut self, quant_lib: QuantLibStub) -> Self {
        self.quant_lib = quant_lib;
        self
    }

    /// Build MoveRuleContext for a position.
    fn build_context(&self, position: &Position) -> MoveRuleContext {
        let mut context = self.default_context.clone();

        // Add position tags to context
        for (key, value) in &position.tags {
            context.set(key, value);
        }

        // Add instrument type
        context.set("instrument_type", position.instrument.instrument_type());
        context.set("specific_type", position.instrument.specific_type());

        // If it's a product, add product_type
        if let crate::ledger::domain::Instrument::Product(product) = &position.instrument {
            context.set("product_type", product.product_type());
        }

        context
    }

    /// Pre-match rules for a position based on context.
    fn pre_match_rules(&self, context: &MoveRuleContext) -> Vec<MoveRule> {
        self.move_rules
            .iter()
            .filter(|rule| rule.matches(context))
            .cloned()
            .collect()
    }

    /// Find product for a trigger.
    fn find_product(&self, _trigger_info: &TriggerInfo) -> Option<&Product> {
        // In a real implementation, the trigger would reference the product
        // For the stub, we return the first product of a matching type
        self.products.values().next()
    }

    /// Find positions that should be affected by an action.
    fn find_positions_for_action(
        &self,
        action: &ProductAction,
        _source_positions: &[&Position],
    ) -> Vec<Position> {
        use crate::ledger::actions::PositionSelector;

        match &action.target_selector {
            Some(PositionSelector::SameAsSource) | None => {
                // Return positions that match the product
                self.positions
                    .iter()
                    .filter(|p| {
                        if let crate::ledger::domain::Instrument::Product(prod) = &p.instrument {
                            prod.id() == action.product_id
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect()
            }
            Some(PositionSelector::Specific { position_ids }) => {
                self.positions
                    .iter()
                    .filter(|p| position_ids.contains(&p.id))
                    .cloned()
                    .collect()
            }
            Some(PositionSelector::LinkedHedges { source_product_id }) => {
                // For stub, return positions tagged as hedges
                self.positions
                    .iter()
                    .filter(|p| p.tags.get("hedge_for").map(|v| v == source_product_id.as_str()).unwrap_or(false))
                    .cloned()
                    .collect()
            }
            Some(PositionSelector::ByQuery { filter }) => {
                self.positions
                    .iter()
                    .filter(|p| filter.matches(&p.tags))
                    .cloned()
                    .collect()
            }
        }
    }

    /// Process actions recursively with cycle detection.
    fn process_actions_recursive(
        &self,
        actions: Vec<ProductAction>,
        current_depth: usize,
        visited: &mut HashSet<(String, String)>,
        result: &mut SimulationResult,
    ) -> EventFrameworkResult<()> {
        if actions.is_empty() {
            return Ok(());
        }

        // Check max depth
        if current_depth >= self.max_depth {
            result.recursion_limited = true;
            return Ok(());
        }

        result.max_depth_reached = result.max_depth_reached.max(current_depth);

        let applicator = DefaultMoveApplicator::new();
        let mut depth_actions = Vec::new();
        let mut depth_result = ApplyResult::new();
        let mut downstream_actions = Vec::new();

        for action in actions {
            // Cycle detection
            let action_key = (action.action_type.as_str().to_string(), action.product_id.as_str().to_string());
            if visited.contains(&action_key) {
                return Err(EventFrameworkError::CycleDetected {
                    action_type: action_key.0,
                    product_id: action_key.1,
                });
            }
            visited.insert(action_key);

            depth_actions.push(action.clone());

            // Find positions for this action
            let positions = self.find_positions_for_action(&action, &[]);
            if positions.is_empty() {
                continue;
            }

            // Build PositionWithContext for each position
            let positions_with_ctx: Vec<PositionWithContext> = positions
                .iter()
                .map(|p| {
                    let context = self.build_context(p);
                    let rules = self.pre_match_rules(&context);
                    PositionWithContext::new(p.clone(), context).with_applicable_rules(rules)
                })
                .collect();

            // Call 2: Apply action to positions
            let apply_result = applicator
                .apply_to_positions(&action, &positions_with_ctx, &self.move_rules)
                .map_err(|e| EventFrameworkError::ApplyError(e.to_string()))?;

            result.total_moves += apply_result.move_count();
            depth_result.moves.extend(apply_result.moves);

            // Collect downstream actions
            for downstream in apply_result.downstream_actions {
                downstream_actions.push(downstream);
            }
        }

        result.actions_by_depth.push(depth_actions);
        result.results_by_depth.push(depth_result);

        // Process downstream actions recursively
        if !downstream_actions.is_empty() {
            self.process_actions_recursive(downstream_actions, current_depth + 1, visited, result)?;
        }

        Ok(())
    }
}

impl Default for EventFrameworkStub {
    fn default() -> Self {
        Self::new()
    }
}

impl EventFramework for EventFrameworkStub {
    fn register_trigger(&mut self, registration: TriggerRegistration) -> EventFrameworkResult<()> {
        let product_id = registration.product_id.clone();
        self.trigger_registrations
            .entry(product_id)
            .or_default()
            .push(registration);
        Ok(())
    }

    fn simulate_trigger(&self, trigger_info: TriggerInfo) -> EventFrameworkResult<SimulationResult> {
        let mut result = SimulationResult::new();

        // Find the product for this trigger
        let product = self.find_product(&trigger_info).ok_or_else(|| {
            EventFrameworkError::ProductNotFound("No product found for trigger".to_string())
        })?;

        // Call 1: Get ProductAction from QuantLib
        let product_action = self
            .quant_lib
            .get_action(product, &trigger_info)
            .map_err(|e| EventFrameworkError::ActionError(e.to_string()))?;

        // Initialize visited set for cycle detection
        let mut visited = HashSet::new();

        // Process the initial action and any downstream actions recursively
        self.process_actions_recursive(vec![product_action], 0, &mut visited, &mut result)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::{Instrument, ProductDetails};
    use crate::ledger::move_rules::BookingPattern;
    use crate::ledger::triggers::BarrierDirection;
    use crate::ledger::types::MoveType;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;

    fn create_ko_product(id: &str) -> Product {
        Product::KnockOutWarrant(ProductDetails::new(id, "Test KO Warrant", "AAPL", "USD"))
    }

    fn create_test_position(id: &str, product: &Product, quantity: Decimal) -> Position {
        let instrument = Instrument::Product(product.clone());
        Position::new(id, "WALLET-001", instrument, quantity)
            .with_tag("desk", "desk_a")
            .with_tag("booking_type", "standard")
    }

    fn create_default_rule() -> MoveRule {
        MoveRule::new(
            "RULE-DEFAULT",
            "Default Settlement",
            BookingPattern::direct("CASH", "SETTLEMENT"),
        )
        .with_move_type(MoveType::Settlement)
        .with_priority(0)
    }

    #[test]
    fn test_event_framework_stub_trigger_registration() {
        let mut framework = EventFrameworkStub::new();

        let product_id = ProductId::new("KO-001");
        let registration = TriggerRegistration::new(product_id.clone(), TriggerType::ContinuousBarrier)
            .with_parameter("barrier_level", "90.00")
            .with_parameter("direction", "down");

        let result = framework.register_trigger(registration.clone());
        assert!(result.is_ok());

        assert!(framework.trigger_registrations.contains_key(&product_id));
        let registrations = framework.trigger_registrations.get(&product_id).unwrap();
        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].trigger_type, TriggerType::ContinuousBarrier);
    }

    #[test]
    fn test_event_framework_stub_simulate_trigger() {
        let product = create_ko_product("KO-001");
        let position = create_test_position("POS-001", &product, Decimal::new(100, 0));
        let rule = create_default_rule();

        let framework = EventFrameworkStub::new()
            .with_product(product.clone())
            .with_position(position)
            .with_rule(rule);

        let trigger_info = TriggerInfo::new_barrier(
            true,
            Utc.with_ymd_and_hms(2024, 12, 15, 10, 30, 0).unwrap(),
            Decimal::new(8950, 2),
            Decimal::new(9000, 2),
            BarrierDirection::Down,
        );

        let result = framework.simulate_trigger(trigger_info).expect("Should succeed");

        assert!(result.total_moves > 0, "Should have generated moves");
        assert!(!result.actions_by_depth.is_empty(), "Should have actions");

        let first_actions = &result.actions_by_depth[0];
        assert_eq!(first_actions.len(), 1);
        assert_eq!(first_actions[0].action_type, ActionType::BarrierBreach);
    }

    #[test]
    fn test_event_framework_stub_max_depth() {
        let product = create_ko_product("KO-001");
        let position = create_test_position("POS-001", &product, Decimal::new(100, 0));
        let rule = create_default_rule();

        let framework = EventFrameworkStub::new()
            .with_max_depth(2)
            .with_product(product.clone())
            .with_position(position)
            .with_rule(rule);

        let trigger_info = TriggerInfo::new_barrier(
            true,
            Utc.with_ymd_and_hms(2024, 12, 15, 10, 30, 0).unwrap(),
            Decimal::new(8950, 2),
            Decimal::new(9000, 2),
            BarrierDirection::Down,
        );

        let result = framework.simulate_trigger(trigger_info).expect("Should succeed");
        assert!(result.max_depth_reached <= 2);
    }
}
