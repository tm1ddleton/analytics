//! MoveRules Engine for the Quantitative Event Engine.
//!
//! This module implements the multi-dimensional MoveRules engine that translates
//! ProductActions into ledger moves based on configurable booking patterns:
//! - MoveRuleContext: Flexible context for rule matching
//! - RuleCondition: Conditions for matching context attributes
//! - BookingPattern: 6 patterns for generating moves
//! - MoveRule: Combines conditions with booking patterns

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::actions::ProductAction;
use super::types::{Move, MoveId, MoveRuleRef, MoveType, Position};

/// Context for MoveRule matching.
///
/// Contains attributes used to match rules against position/product context.
/// Supports dimensions: legal_entity, product_type, jurisdiction, accounting_standard,
/// business_line, client_type, book_type, and any custom attributes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveRuleContext {
    /// Context attributes for rule matching
    pub attributes: HashMap<String, String>,
}

impl MoveRuleContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        MoveRuleContext {
            attributes: HashMap::new(),
        }
    }

    /// Create a context with initial attributes.
    pub fn with_attributes(attributes: HashMap<String, String>) -> Self {
        MoveRuleContext { attributes }
    }

    /// Set an attribute value.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Builder pattern for setting attributes.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Get an attribute value.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    /// Check if an attribute exists.
    pub fn has(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    /// Standard dimension accessors
    pub fn legal_entity(&self) -> Option<&String> {
        self.get("legal_entity")
    }

    pub fn product_type(&self) -> Option<&String> {
        self.get("product_type")
    }

    pub fn jurisdiction(&self) -> Option<&String> {
        self.get("jurisdiction")
    }

    pub fn accounting_standard(&self) -> Option<&String> {
        self.get("accounting_standard")
    }

    pub fn business_line(&self) -> Option<&String> {
        self.get("business_line")
    }

    pub fn client_type(&self) -> Option<&String> {
        self.get("client_type")
    }

    pub fn book_type(&self) -> Option<&String> {
        self.get("book_type")
    }
}

impl Default for MoveRuleContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Condition for matching against context attributes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Attribute must equal specific value
    Equals {
        attribute: String,
        value: String,
    },

    /// Attribute must be one of the specified values
    In {
        attribute: String,
        values: Vec<String>,
    },

    /// Attribute must exist (any value)
    Exists {
        attribute: String,
    },

    /// Negation of another condition
    Not(Box<RuleCondition>),
}

impl RuleCondition {
    /// Create an Equals condition.
    pub fn equals(attribute: impl Into<String>, value: impl Into<String>) -> Self {
        RuleCondition::Equals {
            attribute: attribute.into(),
            value: value.into(),
        }
    }

    /// Create an In condition.
    pub fn is_in(attribute: impl Into<String>, values: Vec<String>) -> Self {
        RuleCondition::In {
            attribute: attribute.into(),
            values,
        }
    }

    /// Create an Exists condition.
    pub fn exists(attribute: impl Into<String>) -> Self {
        RuleCondition::Exists {
            attribute: attribute.into(),
        }
    }

    /// Create a Not condition.
    pub fn not(condition: RuleCondition) -> Self {
        RuleCondition::Not(Box::new(condition))
    }

    /// Check if the condition matches the given context.
    pub fn matches(&self, context: &MoveRuleContext) -> bool {
        match self {
            RuleCondition::Equals { attribute, value } => {
                context.get(attribute).map(|v| v == value).unwrap_or(false)
            }
            RuleCondition::In { attribute, values } => context
                .get(attribute)
                .map(|v| values.contains(v))
                .unwrap_or(false),
            RuleCondition::Exists { attribute } => context.has(attribute),
            RuleCondition::Not(inner) => !inner.matches(context),
        }
    }
}

/// Entry in a Split booking pattern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitEntry {
    /// Debit account for this split
    pub debit_account: String,
    /// Credit account for this split
    pub credit_account: String,
    /// Percentage allocation (0.0 to 1.0)
    pub percentage: Decimal,
    /// Optional description/label
    pub label: Option<String>,
}

impl SplitEntry {
    /// Create a new split entry.
    pub fn new(
        debit_account: impl Into<String>,
        credit_account: impl Into<String>,
        percentage: Decimal,
    ) -> Self {
        SplitEntry {
            debit_account: debit_account.into(),
            credit_account: credit_account.into(),
            percentage,
            label: None,
        }
    }

    /// Add a label to this entry.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Booking pattern for generating moves from ProductActions.
///
/// 6 patterns covering common accounting scenarios:
/// - DirectBooking: Simple debit/credit
/// - WashBookRouting: 3-leg entry through intermediate account
/// - Split: Multiple entries with percentage allocation
/// - TaxWithholding: Gross -> tax + net pattern
/// - Composite: Combination of multiple patterns
/// - Custom: Extensibility for custom logic
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookingPattern {
    /// Direct booking between two accounts
    DirectBooking {
        debit_account: String,
        credit_account: String,
    },

    /// 3-leg entry routing through a wash/intermediate account
    WashBookRouting {
        /// Source account (first leg debit)
        source: String,
        /// Wash/intermediate book account
        wash_book: String,
        /// Destination account (final leg credit)
        destination: String,
    },

    /// Split into multiple entries with percentage allocation
    Split {
        entries: Vec<SplitEntry>,
    },

    /// Tax withholding pattern (gross -> tax + net)
    TaxWithholding {
        /// Account for gross amount
        gross: String,
        /// Account for tax amount
        tax: String,
        /// Account for net amount
        net: String,
        /// Lookup key for tax rate (e.g., jurisdiction code)
        tax_rate_lookup: String,
    },

    /// Composite pattern combining multiple patterns
    Composite {
        patterns: Vec<BookingPattern>,
    },

    /// Custom pattern with rule ID and parameters
    Custom {
        rule_id: String,
        parameters: HashMap<String, String>,
    },
}

impl BookingPattern {
    /// Create a direct booking pattern.
    pub fn direct(
        debit_account: impl Into<String>,
        credit_account: impl Into<String>,
    ) -> Self {
        BookingPattern::DirectBooking {
            debit_account: debit_account.into(),
            credit_account: credit_account.into(),
        }
    }

    /// Create a wash book routing pattern.
    pub fn wash_book(
        source: impl Into<String>,
        wash_book: impl Into<String>,
        destination: impl Into<String>,
    ) -> Self {
        BookingPattern::WashBookRouting {
            source: source.into(),
            wash_book: wash_book.into(),
            destination: destination.into(),
        }
    }

    /// Create a split pattern.
    pub fn split(entries: Vec<SplitEntry>) -> Self {
        BookingPattern::Split { entries }
    }

    /// Create a tax withholding pattern.
    pub fn tax_withholding(
        gross: impl Into<String>,
        tax: impl Into<String>,
        net: impl Into<String>,
        tax_rate_lookup: impl Into<String>,
    ) -> Self {
        BookingPattern::TaxWithholding {
            gross: gross.into(),
            tax: tax.into(),
            net: net.into(),
            tax_rate_lookup: tax_rate_lookup.into(),
        }
    }

    /// Create a composite pattern.
    pub fn composite(patterns: Vec<BookingPattern>) -> Self {
        BookingPattern::Composite { patterns }
    }

    /// Create a custom pattern.
    pub fn custom(rule_id: impl Into<String>, parameters: HashMap<String, String>) -> Self {
        BookingPattern::Custom {
            rule_id: rule_id.into(),
            parameters,
        }
    }
}

/// MoveRule - combines conditions with a booking pattern.
///
/// A rule matches when ALL conditions are satisfied (AND logic).
/// When matched, the booking pattern is applied to generate moves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveRule {
    /// Unique identifier for this rule
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Conditions that must ALL match (AND logic)
    pub conditions: Vec<RuleCondition>,
    /// Pattern to apply when rule matches
    pub booking_pattern: BookingPattern,
    /// Move type to use for generated moves
    pub move_type: MoveType,
    /// Priority for rule ordering (higher = checked first)
    pub priority: i32,
}

impl MoveRule {
    /// Create a new move rule.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        booking_pattern: BookingPattern,
    ) -> Self {
        MoveRule {
            id: id.into(),
            name: name.into(),
            conditions: Vec::new(),
            booking_pattern,
            move_type: MoveType::Transfer,
            priority: 0,
        }
    }

    /// Add a condition to this rule.
    pub fn with_condition(mut self, condition: RuleCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Set the move type.
    pub fn with_move_type(mut self, move_type: MoveType) -> Self {
        self.move_type = move_type;
        self
    }

    /// Set the priority.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Check if this rule matches the given context.
    /// All conditions must match (AND logic).
    pub fn matches(&self, context: &MoveRuleContext) -> bool {
        if self.conditions.is_empty() {
            return true; // No conditions = always match
        }
        self.conditions.iter().all(|c| c.matches(context))
    }

    /// Get the rule reference for audit trail.
    pub fn as_ref(&self) -> MoveRuleRef {
        MoveRuleRef::new(&self.id)
    }
}

/// Error types for move rule application.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MoveRuleError {
    #[error("No matching rule found for context")]
    NoMatchingRule,

    #[error("Invalid split percentages: must sum to 1.0, got {0}")]
    InvalidSplitPercentages(String),

    #[error("Missing required amount for pattern")]
    MissingAmount,

    #[error("Tax rate not found for lookup: {0}")]
    TaxRateNotFound(String),

    #[error("Custom rule not implemented: {0}")]
    CustomRuleNotImplemented(String),
}

/// Result type for move rule operations.
pub type MoveRuleResult<T> = Result<T, MoveRuleError>;

/// Tax rate provider for TaxWithholding patterns.
pub trait TaxRateProvider {
    /// Get tax rate for a lookup key.
    fn get_rate(&self, lookup_key: &str) -> Option<Decimal>;
}

/// Default tax rate provider with hardcoded rates.
pub struct DefaultTaxRateProvider {
    rates: HashMap<String, Decimal>,
}

impl DefaultTaxRateProvider {
    pub fn new() -> Self {
        let mut rates = HashMap::new();
        // Default rates for common jurisdictions
        rates.insert("US".to_string(), Decimal::new(30, 2)); // 30%
        rates.insert("DE".to_string(), Decimal::new(25, 2)); // 25%
        rates.insert("CH".to_string(), Decimal::new(35, 2)); // 35%
        DefaultTaxRateProvider { rates }
    }

    pub fn with_rate(mut self, key: impl Into<String>, rate: Decimal) -> Self {
        self.rates.insert(key.into(), rate);
        self
    }
}

impl Default for DefaultTaxRateProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TaxRateProvider for DefaultTaxRateProvider {
    fn get_rate(&self, lookup_key: &str) -> Option<Decimal> {
        self.rates.get(lookup_key).copied()
    }
}

/// Applies MoveRules to generate moves from ProductActions.
pub struct MoveRuleApplicator<T: TaxRateProvider = DefaultTaxRateProvider> {
    tax_provider: T,
}

impl<T: TaxRateProvider> MoveRuleApplicator<T> {
    /// Create a new applicator with a tax provider.
    pub fn with_tax_provider(tax_provider: T) -> Self {
        MoveRuleApplicator { tax_provider }
    }

    /// Generate a unique move ID.
    fn generate_move_id(&self) -> MoveId {
        MoveId::new(format!("MOV-{}", Uuid::new_v4()))
    }

    /// Apply a booking pattern to generate moves.
    ///
    /// # Arguments
    /// * `pattern` - The booking pattern to apply
    /// * `position` - The position being affected
    /// * `action` - The product action triggering the move
    /// * `rule` - The rule that matched
    /// * `context` - The context used for matching
    /// * `effective_date` - The date for the moves
    /// * `base_amount` - The base amount (scaled by position quantity)
    pub fn apply_pattern(
        &self,
        pattern: &BookingPattern,
        position: &Position,
        action: &ProductAction,
        rule: &MoveRule,
        context: &MoveRuleContext,
        effective_date: NaiveDate,
        base_amount: Decimal,
    ) -> MoveRuleResult<Vec<Move>> {
        match pattern {
            BookingPattern::DirectBooking {
                debit_account,
                credit_account,
            } => {
                let mov = self.create_move(
                    position,
                    action,
                    rule,
                    context,
                    debit_account,
                    credit_account,
                    base_amount,
                    effective_date,
                );
                Ok(vec![mov])
            }

            BookingPattern::WashBookRouting {
                source,
                wash_book,
                destination,
            } => {
                // 3-leg entry: source -> wash_book -> destination
                // Leg 1: Debit source, credit wash_book
                let leg1 = self.create_move(
                    position,
                    action,
                    rule,
                    context,
                    source,
                    wash_book,
                    base_amount,
                    effective_date,
                );

                // Leg 2: Debit wash_book, credit destination
                let leg2 = self.create_move(
                    position,
                    action,
                    rule,
                    context,
                    wash_book,
                    destination,
                    base_amount,
                    effective_date,
                );

                Ok(vec![leg1, leg2])
            }

            BookingPattern::Split { entries } => {
                // Validate percentages sum to 1.0
                let total: Decimal = entries.iter().map(|e| e.percentage).sum();
                if (total - Decimal::ONE).abs() > Decimal::new(1, 4) {
                    return Err(MoveRuleError::InvalidSplitPercentages(total.to_string()));
                }

                let moves = entries
                    .iter()
                    .map(|entry| {
                        let split_amount = base_amount * entry.percentage;
                        self.create_move(
                            position,
                            action,
                            rule,
                            context,
                            &entry.debit_account,
                            &entry.credit_account,
                            split_amount,
                            effective_date,
                        )
                    })
                    .collect();

                Ok(moves)
            }

            BookingPattern::TaxWithholding {
                gross,
                tax,
                net,
                tax_rate_lookup,
            } => {
                let tax_rate = self
                    .tax_provider
                    .get_rate(tax_rate_lookup)
                    .ok_or_else(|| MoveRuleError::TaxRateNotFound(tax_rate_lookup.clone()))?;

                let tax_amount = base_amount * tax_rate;
                let net_amount = base_amount - tax_amount;

                // Move 1: Gross debit
                let gross_move = self.create_move(
                    position,
                    action,
                    rule,
                    context,
                    gross,
                    tax,
                    tax_amount,
                    effective_date,
                );

                // Move 2: Net credit
                let net_move = self.create_move(
                    position,
                    action,
                    rule,
                    context,
                    gross,
                    net,
                    net_amount,
                    effective_date,
                );

                Ok(vec![gross_move, net_move])
            }

            BookingPattern::Composite { patterns } => {
                let mut all_moves = Vec::new();
                for sub_pattern in patterns {
                    let moves = self.apply_pattern(
                        sub_pattern,
                        position,
                        action,
                        rule,
                        context,
                        effective_date,
                        base_amount,
                    )?;
                    all_moves.extend(moves);
                }
                Ok(all_moves)
            }

            BookingPattern::Custom { rule_id, .. } => {
                Err(MoveRuleError::CustomRuleNotImplemented(rule_id.clone()))
            }
        }
    }

    /// Create a single move entry.
    fn create_move(
        &self,
        position: &Position,
        action: &ProductAction,
        rule: &MoveRule,
        context: &MoveRuleContext,
        debit_account: &str,
        credit_account: &str,
        amount: Decimal,
        effective_date: NaiveDate,
    ) -> Move {
        Move::new(
            self.generate_move_id().as_str(),
            position.id.clone(),
            rule.move_type,
            debit_account,
            credit_account,
            amount,
            effective_date,
            action.id.as_str(),
            rule.id.as_str(),
        )
        .with_context(context.attributes.clone())
        .with_inherited_tags(position.tags.clone())
    }

    /// Apply a matched rule to generate moves.
    ///
    /// This is the main entry point for move generation:
    /// 1. Takes a matched rule and applies its booking pattern
    /// 2. Scales amounts by position quantity
    /// 3. Populates audit trail fields
    pub fn apply_rule(
        &self,
        rule: &MoveRule,
        position: &Position,
        action: &ProductAction,
        context: &MoveRuleContext,
        effective_date: NaiveDate,
    ) -> MoveRuleResult<Vec<Move>> {
        // Get base amount from action, scaled by position quantity
        let amount_per_unit = action.amount_per_unit.unwrap_or(Decimal::ZERO);
        let base_amount = amount_per_unit * position.quantity;

        self.apply_pattern(
            &rule.booking_pattern,
            position,
            action,
            rule,
            context,
            effective_date,
            base_amount,
        )
    }
}

impl MoveRuleApplicator<DefaultTaxRateProvider> {
    /// Create a new applicator with default tax provider.
    pub fn new() -> Self {
        MoveRuleApplicator {
            tax_provider: DefaultTaxRateProvider::new(),
        }
    }
}

impl Default for MoveRuleApplicator<DefaultTaxRateProvider> {
    fn default() -> Self {
        Self::new()
    }
}

/// Find the first matching rule from a list of rules.
pub fn find_matching_rule<'a>(
    rules: &'a [MoveRule],
    context: &MoveRuleContext,
) -> Option<&'a MoveRule> {
    // Sort by priority (descending) and find first match
    let mut sorted_rules: Vec<_> = rules.iter().collect();
    sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    sorted_rules.into_iter().find(|r| r.matches(context))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::{Instrument, Product, ProductDetails, ProductId};
    use crate::ledger::actions::ActionType;

    // Helper to create a test position
    fn create_test_position(id: &str, quantity: Decimal) -> Position {
        let instrument = Instrument::Product(Product::KnockOutWarrant(ProductDetails::new(
            "KO-001",
            "Test Knock-Out",
            "AAPL",
            "USD",
        )));
        Position::new(id, "WALLET-001", instrument, quantity)
            .with_tag("desk", "desk_a")
            .with_tag("strategy", "yield_enhancement")
    }

    // Helper to create a test product action
    fn create_test_action(amount_per_unit: Decimal) -> ProductAction {
        ProductAction::new(
            "ACTION-001",
            ActionType::Settlement,
            ProductId::new("KO-001"),
        )
        .with_amount(amount_per_unit)
    }

    // Test 1: RuleCondition matching (Equals, In, Exists, Not)
    #[test]
    fn test_rule_condition_matching() {
        let context = MoveRuleContext::new()
            .with_attribute("legal_entity", "ENTITY_A")
            .with_attribute("jurisdiction", "US")
            .with_attribute("product_type", "KnockOutWarrant")
            .with_attribute("book_type", "trading");

        // Test Equals - match
        let cond = RuleCondition::equals("legal_entity", "ENTITY_A");
        assert!(cond.matches(&context));

        // Test Equals - no match (wrong value)
        let cond = RuleCondition::equals("legal_entity", "ENTITY_B");
        assert!(!cond.matches(&context));

        // Test Equals - no match (missing attribute)
        let cond = RuleCondition::equals("client_type", "retail");
        assert!(!cond.matches(&context));

        // Test In - match
        let cond = RuleCondition::is_in(
            "jurisdiction",
            vec!["US".to_string(), "DE".to_string(), "CH".to_string()],
        );
        assert!(cond.matches(&context));

        // Test In - no match
        let cond = RuleCondition::is_in(
            "jurisdiction",
            vec!["UK".to_string(), "FR".to_string()],
        );
        assert!(!cond.matches(&context));

        // Test Exists - match
        let cond = RuleCondition::exists("book_type");
        assert!(cond.matches(&context));

        // Test Exists - no match
        let cond = RuleCondition::exists("client_type");
        assert!(!cond.matches(&context));

        // Test Not - negates Equals
        let cond = RuleCondition::not(RuleCondition::equals("legal_entity", "ENTITY_B"));
        assert!(cond.matches(&context)); // Not ENTITY_B, and we have ENTITY_A

        let cond = RuleCondition::not(RuleCondition::equals("legal_entity", "ENTITY_A"));
        assert!(!cond.matches(&context)); // Not ENTITY_A, but we have ENTITY_A

        // Test Not - negates Exists
        let cond = RuleCondition::not(RuleCondition::exists("client_type"));
        assert!(cond.matches(&context)); // client_type doesn't exist
    }

    // Test 2: DirectBooking pattern application
    #[test]
    fn test_direct_booking_pattern() {
        let applicator = MoveRuleApplicator::new();
        let position = create_test_position("POS-001", Decimal::new(100, 0)); // 100 units
        let action = create_test_action(Decimal::new(1050, 2)); // 10.50 per unit
        let context = MoveRuleContext::new()
            .with_attribute("legal_entity", "ENTITY_A");

        let rule = MoveRule::new(
            "RULE-DIRECT-001",
            "Direct Settlement",
            BookingPattern::direct("CASH_ACCOUNT", "PRODUCT_ACCOUNT"),
        )
        .with_move_type(MoveType::Settlement);

        let effective_date = NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
        let moves = applicator
            .apply_rule(&rule, &position, &action, &context, effective_date)
            .unwrap();

        assert_eq!(moves.len(), 1);
        let mov = &moves[0];
        assert_eq!(mov.debit_account.as_str(), "CASH_ACCOUNT");
        assert_eq!(mov.credit_account.as_str(), "PRODUCT_ACCOUNT");
        // 100 units * 10.50 per unit = 1050.00
        assert_eq!(mov.amount, Decimal::new(105000, 2));
        assert_eq!(mov.move_type, MoveType::Settlement);
        assert_eq!(mov.source_action.as_str(), "ACTION-001");
        assert_eq!(mov.rule_applied.as_str(), "RULE-DIRECT-001");
        assert_eq!(mov.effective_date, effective_date);

        // Check audit trail
        assert_eq!(
            mov.context_snapshot.get("legal_entity"),
            Some(&"ENTITY_A".to_string())
        );
        assert_eq!(
            mov.inherited_tags.get("desk"),
            Some(&"desk_a".to_string())
        );
    }

    // Test 3: WashBookRouting pattern (3-leg entry)
    #[test]
    fn test_wash_book_routing_pattern() {
        let applicator = MoveRuleApplicator::new();
        let position = create_test_position("POS-001", Decimal::new(50, 0)); // 50 units
        let action = create_test_action(Decimal::new(2000, 2)); // 20.00 per unit
        let context = MoveRuleContext::new()
            .with_attribute("legal_entity", "ENTITY_A")
            .with_attribute("jurisdiction", "INTERCOMPANY");

        let rule = MoveRule::new(
            "RULE-WASH-001",
            "Intercompany Settlement",
            BookingPattern::wash_book(
                "ENTITY_A_CASH",
                "INTERCOMPANY_WASH",
                "ENTITY_B_CASH",
            ),
        )
        .with_move_type(MoveType::Settlement);

        let effective_date = NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
        let moves = applicator
            .apply_rule(&rule, &position, &action, &context, effective_date)
            .unwrap();

        // Should generate 2 moves (3-leg: source->wash, wash->dest)
        assert_eq!(moves.len(), 2);

        // Leg 1: source -> wash_book
        let leg1 = &moves[0];
        assert_eq!(leg1.debit_account.as_str(), "ENTITY_A_CASH");
        assert_eq!(leg1.credit_account.as_str(), "INTERCOMPANY_WASH");
        // 50 units * 20.00 = 1000.00
        assert_eq!(leg1.amount, Decimal::new(100000, 2));

        // Leg 2: wash_book -> destination
        let leg2 = &moves[1];
        assert_eq!(leg2.debit_account.as_str(), "INTERCOMPANY_WASH");
        assert_eq!(leg2.credit_account.as_str(), "ENTITY_B_CASH");
        assert_eq!(leg2.amount, Decimal::new(100000, 2));

        // Both legs should have same audit trail
        assert_eq!(leg1.source_action.as_str(), leg2.source_action.as_str());
        assert_eq!(leg1.rule_applied.as_str(), leg2.rule_applied.as_str());
    }

    // Test 4: Split pattern with percentage allocation
    #[test]
    fn test_split_pattern_percentage_allocation() {
        let applicator = MoveRuleApplicator::new();
        let position = create_test_position("POS-001", Decimal::new(200, 0)); // 200 units
        let action = create_test_action(Decimal::new(500, 2)); // 5.00 per unit
        let context = MoveRuleContext::new()
            .with_attribute("legal_entity", "ENTITY_A")
            .with_attribute("business_line", "MULTI_DESK");

        // Split 60% to desk_a, 40% to desk_b
        let entries = vec![
            SplitEntry::new("CASH", "DESK_A_PNL", Decimal::new(60, 2))
                .with_label("Desk A allocation"),
            SplitEntry::new("CASH", "DESK_B_PNL", Decimal::new(40, 2))
                .with_label("Desk B allocation"),
        ];

        let rule = MoveRule::new(
            "RULE-SPLIT-001",
            "Multi-Desk P&L Split",
            BookingPattern::split(entries),
        )
        .with_move_type(MoveType::Settlement);

        let effective_date = NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
        let moves = applicator
            .apply_rule(&rule, &position, &action, &context, effective_date)
            .unwrap();

        assert_eq!(moves.len(), 2);

        // Total: 200 units * 5.00 = 1000.00
        // Split 1: 60% = 600.00
        let split1 = &moves[0];
        assert_eq!(split1.debit_account.as_str(), "CASH");
        assert_eq!(split1.credit_account.as_str(), "DESK_A_PNL");
        assert_eq!(split1.amount, Decimal::new(60000, 2));

        // Split 2: 40% = 400.00
        let split2 = &moves[1];
        assert_eq!(split2.debit_account.as_str(), "CASH");
        assert_eq!(split2.credit_account.as_str(), "DESK_B_PNL");
        assert_eq!(split2.amount, Decimal::new(40000, 2));

        // Verify percentages sum to total
        let total = split1.amount + split2.amount;
        assert_eq!(total, Decimal::new(100000, 2));
    }

    // Test 5: Rule matching with multiple conditions (AND logic)
    #[test]
    fn test_rule_matching_and_logic() {
        let context = MoveRuleContext::new()
            .with_attribute("legal_entity", "ENTITY_A")
            .with_attribute("jurisdiction", "US")
            .with_attribute("product_type", "KnockOutWarrant")
            .with_attribute("book_type", "trading");

        // Rule with multiple conditions - all must match
        let rule = MoveRule::new(
            "RULE-MULTI-001",
            "US KnockOut Trading",
            BookingPattern::direct("CASH", "PRODUCT"),
        )
        .with_condition(RuleCondition::equals("jurisdiction", "US"))
        .with_condition(RuleCondition::equals("product_type", "KnockOutWarrant"))
        .with_condition(RuleCondition::equals("book_type", "trading"));

        assert!(rule.matches(&context));

        // Change one attribute - should no longer match
        let context2 = MoveRuleContext::new()
            .with_attribute("legal_entity", "ENTITY_A")
            .with_attribute("jurisdiction", "DE") // Different jurisdiction
            .with_attribute("product_type", "KnockOutWarrant")
            .with_attribute("book_type", "trading");

        assert!(!rule.matches(&context2));

        // Empty conditions = always match
        let rule_empty = MoveRule::new(
            "RULE-EMPTY",
            "Catch-all",
            BookingPattern::direct("DEFAULT_CASH", "DEFAULT_PRODUCT"),
        );
        assert!(rule_empty.matches(&context));
        assert!(rule_empty.matches(&context2));
    }

    // Test 6: Find matching rule from list
    #[test]
    fn test_find_matching_rule_priority() {
        let rules = vec![
            // Low priority catch-all
            MoveRule::new(
                "RULE-DEFAULT",
                "Default",
                BookingPattern::direct("CASH", "DEFAULT"),
            )
            .with_priority(0),
            // High priority specific rule
            MoveRule::new(
                "RULE-US-SPECIFIC",
                "US Specific",
                BookingPattern::direct("CASH", "US_ACCOUNT"),
            )
            .with_condition(RuleCondition::equals("jurisdiction", "US"))
            .with_priority(100),
            // Medium priority rule
            MoveRule::new(
                "RULE-KNOCKOUT",
                "KnockOut Rule",
                BookingPattern::direct("CASH", "KO_ACCOUNT"),
            )
            .with_condition(RuleCondition::equals("product_type", "KnockOutWarrant"))
            .with_priority(50),
        ];

        // Context matching both specific rules
        let context = MoveRuleContext::new()
            .with_attribute("jurisdiction", "US")
            .with_attribute("product_type", "KnockOutWarrant");

        // Should match highest priority rule (US Specific)
        let matched = find_matching_rule(&rules, &context);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().id, "RULE-US-SPECIFIC");

        // Context matching only KnockOut rule
        let context2 = MoveRuleContext::new()
            .with_attribute("jurisdiction", "DE")
            .with_attribute("product_type", "KnockOutWarrant");

        let matched = find_matching_rule(&rules, &context2);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().id, "RULE-KNOCKOUT");

        // Context matching only default
        let context3 = MoveRuleContext::new()
            .with_attribute("jurisdiction", "DE")
            .with_attribute("product_type", "MiniCertificate");

        let matched = find_matching_rule(&rules, &context3);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().id, "RULE-DEFAULT");
    }
}
