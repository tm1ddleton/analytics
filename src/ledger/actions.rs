//! ProductAction and Call 0/1 Interface for the Quantitative Event Engine.
//!
//! This module defines the ProductAction types and the Call 0/1 interfaces:
//!
//! ## Call 0: Product Registration
//! - TriggerSubscription: Minimal criteria for Event Framework to monitor
//! - ProductEventDetails: Verbose event info from QuantLib
//! - SubscriptionTranslator: Transforms QuantLib output to subscriptions
//!
//! ## Call 1: Trigger Processing
//! - ActionType: 9 variants covering product lifecycle actions
//! - ActionDetails: Type-specific data for each action type
//! - ProductAction: The output of Call 1 (unit economics, quantity=1)
//! - PositionSelector: For flexible downstream targeting
//! - ProductActionCalculator: The Call 1 interface trait

use chrono::{DateTime, NaiveDate, Utc};
use prost::Message;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::domain::{Product, ProductId};
use super::triggers::{BarrierDirection, FixingType, TriggerInfo, TriggerType};
use super::types::{PortfolioFilter, PositionId, ProductActionRef};

// =============================================================================
// Call 0: Product Registration Types
// =============================================================================

/// Trigger subscription - minimal criteria for Event Framework to monitor.
///
/// This is the output of Call 0. It contains only what the Event Framework
/// needs to set up monitoring - no quant-specific details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerSubscription {
    /// Product this subscription is for
    pub product_id: ProductId,
    /// Type of trigger to monitor
    pub trigger_type: TriggerType,
    /// Underlying asset to monitor (for price-based triggers)
    pub underlying: Option<String>,
    /// Subscription-specific parameters
    pub criteria: SubscriptionCriteria,
}

impl TriggerSubscription {
    /// Create a new trigger subscription.
    pub fn new(product_id: ProductId, trigger_type: TriggerType) -> Self {
        TriggerSubscription {
            product_id,
            trigger_type,
            underlying: None,
            criteria: SubscriptionCriteria::None,
        }
    }

    /// Set the underlying asset.
    pub fn with_underlying(mut self, underlying: impl Into<String>) -> Self {
        self.underlying = Some(underlying.into());
        self
    }

    /// Set the subscription criteria.
    pub fn with_criteria(mut self, criteria: SubscriptionCriteria) -> Self {
        self.criteria = criteria;
        self
    }

    /// Create a continuous barrier subscription.
    pub fn continuous_barrier(
        product_id: ProductId,
        underlying: impl Into<String>,
        level: Decimal,
        direction: BarrierDirection,
    ) -> Self {
        TriggerSubscription {
            product_id,
            trigger_type: TriggerType::ContinuousBarrier,
            underlying: Some(underlying.into()),
            criteria: SubscriptionCriteria::Barrier { level, direction },
        }
    }

    /// Create a discrete barrier subscription.
    pub fn discrete_barrier(
        product_id: ProductId,
        underlying: impl Into<String>,
        level: Decimal,
        direction: BarrierDirection,
        observation_times: Vec<DateTime<Utc>>,
    ) -> Self {
        TriggerSubscription {
            product_id,
            trigger_type: TriggerType::Barrier,
            underlying: Some(underlying.into()),
            criteria: SubscriptionCriteria::DiscreteBarrier {
                level,
                direction,
                observation_times,
            },
        }
    }

    /// Create a fixing subscription.
    pub fn fixing(
        product_id: ProductId,
        underlying: impl Into<String>,
        fixing_type: FixingType,
        scheduled_time: DateTime<Utc>,
    ) -> Self {
        TriggerSubscription {
            product_id,
            trigger_type: TriggerType::Fixing(fixing_type),
            underlying: Some(underlying.into()),
            criteria: SubscriptionCriteria::Scheduled { time: scheduled_time },
        }
    }

    /// Create an expiry subscription.
    pub fn expiry(product_id: ProductId, expiry_time: DateTime<Utc>) -> Self {
        TriggerSubscription {
            product_id,
            trigger_type: TriggerType::Expiry,
            underlying: None,
            criteria: SubscriptionCriteria::Scheduled { time: expiry_time },
        }
    }

    /// Create a coupon subscription.
    pub fn coupon(product_id: ProductId, payment_time: DateTime<Utc>) -> Self {
        TriggerSubscription {
            product_id,
            trigger_type: TriggerType::CouponPayment,
            underlying: None,
            criteria: SubscriptionCriteria::Scheduled { time: payment_time },
        }
    }
}

/// Subscription criteria - the minimal parameters Event Framework needs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SubscriptionCriteria {
    /// No additional criteria (time-based triggers)
    None,

    /// Scheduled time trigger
    Scheduled {
        /// When to fire the trigger
        time: DateTime<Utc>,
    },

    /// Continuous barrier monitoring criteria
    Barrier {
        /// Barrier level to monitor
        level: Decimal,
        /// Direction (up or down)
        direction: BarrierDirection,
    },

    /// Discrete barrier monitoring criteria
    DiscreteBarrier {
        /// Barrier level to monitor
        level: Decimal,
        /// Direction (up or down)
        direction: BarrierDirection,
        /// Specific times to check
        observation_times: Vec<DateTime<Utc>>,
    },

    /// Multiple scheduled times (for periodic fixings)
    ScheduledSeries {
        /// List of times
        times: Vec<DateTime<Utc>>,
    },
}

impl Default for SubscriptionCriteria {
    fn default() -> Self {
        SubscriptionCriteria::None
    }
}

/// Product event details - verbose output from QuantLib.
///
/// This is what QuantLib returns when asked about a product's events.
/// Contains full quant-specific details that this library transforms
/// into minimal TriggerSubscriptions for the Event Framework.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductEventDetails {
    /// Product this applies to
    pub product_id: ProductId,
    /// Barrier events (if any)
    pub barriers: Vec<BarrierEventDetail>,
    /// Fixing events (if any)
    pub fixings: Vec<FixingEventDetail>,
    /// Expiry event (if any)
    pub expiry: Option<ExpiryEventDetail>,
    /// Coupon events (if any)
    pub coupons: Vec<CouponEventDetail>,
    /// Corporate action sensitivity
    pub corporate_action_sensitive: bool,
}

impl ProductEventDetails {
    /// Create new empty product event details.
    pub fn new(product_id: ProductId) -> Self {
        ProductEventDetails {
            product_id,
            barriers: Vec::new(),
            fixings: Vec::new(),
            expiry: None,
            coupons: Vec::new(),
            corporate_action_sensitive: false,
        }
    }

    /// Add a barrier event.
    pub fn with_barrier(mut self, barrier: BarrierEventDetail) -> Self {
        self.barriers.push(barrier);
        self
    }

    /// Add a fixing event.
    pub fn with_fixing(mut self, fixing: FixingEventDetail) -> Self {
        self.fixings.push(fixing);
        self
    }

    /// Set expiry event.
    pub fn with_expiry(mut self, expiry: ExpiryEventDetail) -> Self {
        self.expiry = Some(expiry);
        self
    }

    /// Add a coupon event.
    pub fn with_coupon(mut self, coupon: CouponEventDetail) -> Self {
        self.coupons.push(coupon);
        self
    }

    /// Set corporate action sensitivity.
    pub fn with_corporate_action_sensitivity(mut self, sensitive: bool) -> Self {
        self.corporate_action_sensitive = sensitive;
        self
    }
}

/// Barrier event detail from QuantLib.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BarrierEventDetail {
    /// Barrier level
    pub level: Decimal,
    /// Direction (up or down)
    pub direction: BarrierDirection,
    /// Observation type
    pub observation_type: BarrierObservationType,
    /// Underlying to monitor
    pub underlying: String,
    /// Observation start date
    pub start_date: NaiveDate,
    /// Observation end date
    pub end_date: NaiveDate,
    /// Is this a knockout (true) or knockin (false)
    pub is_knockout: bool,
    /// Rebate amount if barrier is breached (optional)
    pub rebate: Option<Decimal>,
    /// Specific observation times (for discrete barriers)
    pub observation_times: Vec<DateTime<Utc>>,
}

impl BarrierEventDetail {
    /// Create a new continuous barrier event detail.
    pub fn continuous(
        level: Decimal,
        direction: BarrierDirection,
        underlying: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Self {
        BarrierEventDetail {
            level,
            direction,
            observation_type: BarrierObservationType::Continuous,
            underlying: underlying.into(),
            start_date,
            end_date,
            is_knockout: true,
            rebate: None,
            observation_times: Vec::new(),
        }
    }

    /// Create a new discrete barrier event detail.
    pub fn discrete(
        level: Decimal,
        direction: BarrierDirection,
        underlying: impl Into<String>,
        observation_times: Vec<DateTime<Utc>>,
    ) -> Self {
        let start_date = observation_times.first()
            .map(|t| t.date_naive())
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let end_date = observation_times.last()
            .map(|t| t.date_naive())
            .unwrap_or(start_date);

        BarrierEventDetail {
            level,
            direction,
            observation_type: BarrierObservationType::Discrete,
            underlying: underlying.into(),
            start_date,
            end_date,
            is_knockout: true,
            rebate: None,
            observation_times,
        }
    }

    /// Set knockout flag.
    pub fn with_knockout(mut self, is_knockout: bool) -> Self {
        self.is_knockout = is_knockout;
        self
    }

    /// Set rebate.
    pub fn with_rebate(mut self, rebate: Decimal) -> Self {
        self.rebate = Some(rebate);
        self
    }
}

/// Barrier observation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BarrierObservationType {
    /// Continuous tick-level monitoring
    Continuous,
    /// Discrete observation at specific times
    Discrete,
}

/// Fixing event detail from QuantLib.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixingEventDetail {
    /// Fixing type
    pub fixing_type: FixingType,
    /// Scheduled observation time
    pub scheduled_time: DateTime<Utc>,
    /// Underlying to observe
    pub underlying: String,
    /// Averaging window (for Asian-style fixings)
    pub averaging_window: Option<AveragingWindow>,
    /// Weight for weighted average fixings
    pub weight: Option<Decimal>,
}

impl FixingEventDetail {
    /// Create a new fixing event detail.
    pub fn new(
        fixing_type: FixingType,
        scheduled_time: DateTime<Utc>,
        underlying: impl Into<String>,
    ) -> Self {
        FixingEventDetail {
            fixing_type,
            scheduled_time,
            underlying: underlying.into(),
            averaging_window: None,
            weight: None,
        }
    }

    /// Set averaging window.
    pub fn with_averaging(mut self, window: AveragingWindow) -> Self {
        self.averaging_window = Some(window);
        self
    }

    /// Set weight.
    pub fn with_weight(mut self, weight: Decimal) -> Self {
        self.weight = Some(weight);
        self
    }
}

/// Averaging window for Asian-style fixings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AveragingWindow {
    /// Start of averaging period
    pub start: DateTime<Utc>,
    /// End of averaging period
    pub end: DateTime<Utc>,
    /// Number of observations
    pub observation_count: u32,
}

/// Expiry event detail from QuantLib.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpiryEventDetail {
    /// Expiry time
    pub expiry_time: DateTime<Utc>,
    /// Settlement type
    pub settlement_type: SettlementType,
    /// Settlement delay in business days
    pub settlement_delay_days: u32,
    /// Whether auto-exercise applies
    pub auto_exercise: bool,
    /// Strike for exercise calculation (options)
    pub strike: Option<Decimal>,
}

impl ExpiryEventDetail {
    /// Create a new expiry event detail.
    pub fn new(expiry_time: DateTime<Utc>, settlement_type: SettlementType) -> Self {
        ExpiryEventDetail {
            expiry_time,
            settlement_type,
            settlement_delay_days: 2,
            auto_exercise: true,
            strike: None,
        }
    }

    /// Set settlement delay.
    pub fn with_settlement_delay(mut self, days: u32) -> Self {
        self.settlement_delay_days = days;
        self
    }

    /// Set auto-exercise flag.
    pub fn with_auto_exercise(mut self, auto: bool) -> Self {
        self.auto_exercise = auto;
        self
    }

    /// Set strike.
    pub fn with_strike(mut self, strike: Decimal) -> Self {
        self.strike = Some(strike);
        self
    }
}

/// Coupon event detail from QuantLib.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CouponEventDetail {
    /// Payment date
    pub payment_date: DateTime<Utc>,
    /// Coupon rate (annualized)
    pub coupon_rate: Decimal,
    /// Accrual start date
    pub accrual_start: NaiveDate,
    /// Accrual end date
    pub accrual_end: NaiveDate,
    /// Day count convention
    pub day_count_convention: String,
    /// Conditional (e.g., memory coupon)
    pub is_conditional: bool,
    /// Condition description (if conditional)
    pub condition: Option<String>,
}

impl CouponEventDetail {
    /// Create a new coupon event detail.
    pub fn new(
        payment_date: DateTime<Utc>,
        coupon_rate: Decimal,
        accrual_start: NaiveDate,
        accrual_end: NaiveDate,
    ) -> Self {
        CouponEventDetail {
            payment_date,
            coupon_rate,
            accrual_start,
            accrual_end,
            day_count_convention: "ACT/360".to_string(),
            is_conditional: false,
            condition: None,
        }
    }

    /// Set day count convention.
    pub fn with_day_count(mut self, convention: impl Into<String>) -> Self {
        self.day_count_convention = convention.into();
        self
    }

    /// Set conditional flag with description.
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.is_conditional = true;
        self.condition = Some(condition.into());
        self
    }
}

/// Translates verbose QuantLib event details to minimal Event Framework subscriptions.
///
/// This is the core transformation in Call 0 - taking quant-specific data
/// and producing only what the Event Framework needs to monitor.
pub fn translate_to_subscriptions(events: &ProductEventDetails) -> Vec<TriggerSubscription> {
    let mut subscriptions = Vec::new();

    // Translate barrier events
    for barrier in &events.barriers {
        let subscription = match barrier.observation_type {
            BarrierObservationType::Continuous => {
                TriggerSubscription::continuous_barrier(
                    events.product_id.clone(),
                    &barrier.underlying,
                    barrier.level,
                    barrier.direction,
                )
            }
            BarrierObservationType::Discrete => {
                TriggerSubscription::discrete_barrier(
                    events.product_id.clone(),
                    &barrier.underlying,
                    barrier.level,
                    barrier.direction,
                    barrier.observation_times.clone(),
                )
            }
        };
        subscriptions.push(subscription);
    }

    // Translate fixing events
    for fixing in &events.fixings {
        let subscription = TriggerSubscription::fixing(
            events.product_id.clone(),
            &fixing.underlying,
            fixing.fixing_type,
            fixing.scheduled_time,
        );
        subscriptions.push(subscription);
    }

    // Translate expiry event
    if let Some(expiry) = &events.expiry {
        let subscription = TriggerSubscription::expiry(
            events.product_id.clone(),
            expiry.expiry_time,
        );
        subscriptions.push(subscription);
    }

    // Translate coupon events
    for coupon in &events.coupons {
        let subscription = TriggerSubscription::coupon(
            events.product_id.clone(),
            coupon.payment_date,
        );
        subscriptions.push(subscription);
    }

    // Add corporate action subscriptions if sensitive
    if events.corporate_action_sensitive {
        // Dividend subscription (Event Framework will handle underlying lookup)
        subscriptions.push(TriggerSubscription::new(
            events.product_id.clone(),
            TriggerType::Dividend,
        ));

        // Stock split subscription
        subscriptions.push(TriggerSubscription::new(
            events.product_id.clone(),
            TriggerType::StockSplit,
        ));
    }

    subscriptions
}

// =============================================================================
// Call 1: Trigger Processing Types
// =============================================================================

/// Action type enum with 9 variants covering product lifecycle events.
///
/// These represent the types of actions that can result from processing triggers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum ActionType {
    /// Scheduled coupon payment
    CouponPayment = 1,
    /// Product settlement (maturity or early termination)
    Settlement = 2,
    /// Barrier breach event (knockout/knockin)
    BarrierBreach = 3,
    /// Early exercise of an option
    EarlyExercise = 4,
    /// Dividend adjustment on underlying
    DividendAdjustment = 5,
    /// Stock split adjustment on underlying
    StockSplitAdjustment = 6,
    /// Price fixing observation recorded
    FixingObserved = 7,
    /// Unwind of associated hedges
    HedgeUnwind = 8,
    /// Rebalance of associated hedges
    HedgeRebalance = 9,
}

impl ActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionType::CouponPayment => "CouponPayment",
            ActionType::Settlement => "Settlement",
            ActionType::BarrierBreach => "BarrierBreach",
            ActionType::EarlyExercise => "EarlyExercise",
            ActionType::DividendAdjustment => "DividendAdjustment",
            ActionType::StockSplitAdjustment => "StockSplitAdjustment",
            ActionType::FixingObserved => "FixingObserved",
            ActionType::HedgeUnwind => "HedgeUnwind",
            ActionType::HedgeRebalance => "HedgeRebalance",
        }
    }

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(ActionType::CouponPayment),
            2 => Some(ActionType::Settlement),
            3 => Some(ActionType::BarrierBreach),
            4 => Some(ActionType::EarlyExercise),
            5 => Some(ActionType::DividendAdjustment),
            6 => Some(ActionType::StockSplitAdjustment),
            7 => Some(ActionType::FixingObserved),
            8 => Some(ActionType::HedgeUnwind),
            9 => Some(ActionType::HedgeRebalance),
            _ => None,
        }
    }

    /// Returns whether this action type typically generates downstream actions.
    pub fn has_downstream_actions(&self) -> bool {
        matches!(
            self,
            ActionType::BarrierBreach | ActionType::EarlyExercise | ActionType::Settlement
        )
    }
}

/// Type-specific action details.
///
/// Each variant contains data specific to that action type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionDetails {
    /// Coupon payment details
    Coupon {
        /// Coupon rate (annualized)
        coupon_rate: Decimal,
        /// Period start date
        period_start: NaiveDate,
        /// Period end date
        period_end: NaiveDate,
    },

    /// Settlement details
    Settlement {
        /// Settlement type (cash, physical, or combination)
        settlement_type: SettlementType,
        /// Reason for settlement
        reason: SettlementReason,
    },

    /// Barrier breach details
    Barrier {
        /// Barrier level that was breached
        barrier_level: Decimal,
        /// Price at time of breach
        breach_price: Decimal,
        /// Whether this is a knockout (true) or knockin (false)
        is_knockout: bool,
    },

    /// Exercise details
    Exercise {
        /// Strike price
        strike_price: Decimal,
        /// Spot price at exercise
        spot_price: Decimal,
        /// Whether this is a call (true) or put (false)
        is_call: bool,
    },

    /// Dividend adjustment details
    Dividend {
        /// Dividend amount per share
        dividend_per_share: Decimal,
        /// Adjustment factor applied
        adjustment_factor: Decimal,
        /// Ex-dividend date
        ex_date: NaiveDate,
    },

    /// Stock split adjustment details
    StockSplit {
        /// New shares per old share (e.g., 4 for a 4:1 split)
        new_shares: u32,
        /// Old shares
        old_shares: u32,
        /// Adjustment factor applied
        adjustment_factor: Decimal,
    },

    /// Fixing observation details
    Fixing {
        /// Observed price
        observed_price: Decimal,
        /// Fixing type (initial, final, periodic, reset)
        fixing_type: String,
        /// Index or underlying that was observed
        underlying: String,
    },

    /// Hedge unwind details
    HedgeUnwind {
        /// Reason for unwinding
        reason: String,
        /// Unwind percentage (1.0 for full unwind)
        unwind_percentage: Decimal,
    },

    /// Hedge rebalance details
    HedgeRebalance {
        /// New delta to target
        target_delta: Decimal,
        /// Previous delta
        previous_delta: Decimal,
    },

    /// No additional details needed
    None,
}

impl Default for ActionDetails {
    fn default() -> Self {
        ActionDetails::None
    }
}

/// Settlement type for product termination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettlementType {
    /// Cash settlement only
    Cash,
    /// Physical delivery of underlying
    Physical,
    /// Cash with potential physical component
    CashOrPhysical,
}

impl SettlementType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SettlementType::Cash => "Cash",
            SettlementType::Physical => "Physical",
            SettlementType::CashOrPhysical => "CashOrPhysical",
        }
    }
}

/// Reason for settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettlementReason {
    /// Normal maturity at expiry
    Maturity,
    /// Early termination due to barrier breach
    BarrierBreach,
    /// Early exercise by holder
    EarlyExercise,
    /// Issuer call/redemption
    IssuerCall,
}

impl SettlementReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            SettlementReason::Maturity => "Maturity",
            SettlementReason::BarrierBreach => "BarrierBreach",
            SettlementReason::EarlyExercise => "EarlyExercise",
            SettlementReason::IssuerCall => "IssuerCall",
        }
    }
}

/// Position selector for downstream action targeting.
///
/// Determines which positions a downstream ProductAction should apply to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionSelector {
    /// Target the same position as the source action (default)
    SameAsSource,

    /// Target linked hedge positions for a product
    LinkedHedges {
        /// Product ID to find linked hedges for
        source_product_id: ProductId,
    },

    /// Target specific positions by ID
    Specific {
        /// List of position IDs to target
        position_ids: Vec<PositionId>,
    },

    /// Target positions matching a portfolio query
    ByQuery {
        /// Filter to select positions
        filter: PortfolioFilter,
    },
}

impl Default for PositionSelector {
    fn default() -> Self {
        PositionSelector::SameAsSource
    }
}

impl PositionSelector {
    /// Create a selector for linked hedges.
    pub fn linked_hedges(product_id: ProductId) -> Self {
        PositionSelector::LinkedHedges {
            source_product_id: product_id,
        }
    }

    /// Create a selector for specific positions.
    pub fn specific(position_ids: Vec<PositionId>) -> Self {
        PositionSelector::Specific { position_ids }
    }

    /// Create a selector using a portfolio query.
    pub fn by_query(filter: PortfolioFilter) -> Self {
        PositionSelector::ByQuery { filter }
    }
}

/// ProductAction - the output of Call 1.
///
/// Represents a unit action (quantity=1) with product-level economics.
/// This is what the ProductActionCalculator returns after processing a trigger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductAction {
    /// Unique identifier for this action
    pub id: ProductActionRef,

    /// Type of action
    pub action_type: ActionType,

    /// Product this action applies to
    pub product_id: ProductId,

    /// Amount per unit (e.g., coupon amount per certificate)
    pub amount_per_unit: Option<Decimal>,

    /// Settlement date for the action
    pub settlement_date: Option<NaiveDate>,

    /// Type-specific details
    pub details: ActionDetails,

    /// Reference to source action (for event chaining/lineage)
    pub source_action_ref: Option<ProductActionRef>,

    /// Selector for downstream position targeting
    pub target_selector: Option<PositionSelector>,
}

impl ProductAction {
    /// Create a new ProductAction.
    pub fn new(
        id: impl Into<String>,
        action_type: ActionType,
        product_id: ProductId,
    ) -> Self {
        ProductAction {
            id: ProductActionRef::new(id),
            action_type,
            product_id,
            amount_per_unit: None,
            settlement_date: None,
            details: ActionDetails::None,
            source_action_ref: None,
            target_selector: None,
        }
    }

    /// Set the amount per unit.
    pub fn with_amount(mut self, amount: Decimal) -> Self {
        self.amount_per_unit = Some(amount);
        self
    }

    /// Set the settlement date.
    pub fn with_settlement_date(mut self, date: NaiveDate) -> Self {
        self.settlement_date = Some(date);
        self
    }

    /// Set the action details.
    pub fn with_details(mut self, details: ActionDetails) -> Self {
        self.details = details;
        self
    }

    /// Set the source action reference for event chaining.
    pub fn with_source_action(mut self, source_ref: ProductActionRef) -> Self {
        self.source_action_ref = Some(source_ref);
        self
    }

    /// Set the target position selector.
    pub fn with_target_selector(mut self, selector: PositionSelector) -> Self {
        self.target_selector = Some(selector);
        self
    }

    /// Check if this action has a source action (is part of a chain).
    pub fn is_chained(&self) -> bool {
        self.source_action_ref.is_some()
    }

    /// Get the lineage depth by following source action refs.
    /// Returns 0 for root actions, 1+ for chained actions.
    pub fn lineage_depth(&self) -> usize {
        if self.source_action_ref.is_some() {
            1 // In a real implementation, would need to look up the chain
        } else {
            0
        }
    }
}

/// Protobuf representation for ProductAction.
#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct ProductActionProto {
    /// Action identifier
    #[prost(string, tag = "1")]
    pub id: String,

    /// Action type discriminant
    #[prost(int32, tag = "2")]
    pub action_type: i32,

    /// Product identifier
    #[prost(string, tag = "3")]
    pub product_id: String,

    /// Amount per unit as string (for decimal precision)
    #[prost(string, optional, tag = "4")]
    pub amount_per_unit: Option<String>,

    /// Settlement date as YYYY-MM-DD
    #[prost(string, optional, tag = "5")]
    pub settlement_date: Option<String>,

    /// Source action reference for lineage
    #[prost(string, optional, tag = "6")]
    pub source_action_ref: Option<String>,

    /// Details as JSON string (for flexibility)
    #[prost(string, tag = "7")]
    pub details_json: String,

    /// Target selector type
    #[prost(int32, tag = "8")]
    pub target_selector_type: i32,

    /// Target selector data as JSON
    #[prost(string, optional, tag = "9")]
    pub target_selector_json: Option<String>,
}

impl ProductAction {
    /// Convert to protobuf representation.
    pub fn to_proto(&self) -> ProductActionProto {
        let target_selector_type = match &self.target_selector {
            None => 0,
            Some(PositionSelector::SameAsSource) => 1,
            Some(PositionSelector::LinkedHedges { .. }) => 2,
            Some(PositionSelector::Specific { .. }) => 3,
            Some(PositionSelector::ByQuery { .. }) => 4,
        };

        let target_selector_json = self.target_selector.as_ref().map(|s| {
            serde_json::to_string(s).unwrap_or_default()
        });

        ProductActionProto {
            id: self.id.as_str().to_string(),
            action_type: self.action_type as i32,
            product_id: self.product_id.as_str().to_string(),
            amount_per_unit: self.amount_per_unit.map(|a| a.to_string()),
            settlement_date: self.settlement_date.map(|d| d.format("%Y-%m-%d").to_string()),
            source_action_ref: self.source_action_ref.as_ref().map(|r| r.as_str().to_string()),
            details_json: serde_json::to_string(&self.details).unwrap_or_default(),
            target_selector_type,
            target_selector_json,
        }
    }

    /// Encode to protobuf bytes.
    pub fn encode_proto(&self) -> Vec<u8> {
        let proto = self.to_proto();
        let mut buf = Vec::new();
        proto.encode(&mut buf).expect("encoding should not fail");
        buf
    }
}

/// Error type for ProductAction calculations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ActionError {
    #[error("Trigger type {0} not applicable to product type {1}")]
    TriggerNotApplicable(String, String),

    #[error("Missing required data: {0}")]
    MissingData(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),

    #[error("QuantLib error: {0}")]
    QuantLibError(String),
}

/// Result type for ProductAction calculations.
pub type ActionResult<T> = Result<T, ActionError>;

/// Quant library trait (stub interface).
///
/// This is the interface for the external Quant library that performs
/// actual product calculations. Implementation is out of scope.
pub trait QuantLib: Send + Sync {
    /// Get product event details for trigger registration (Call 0).
    ///
    /// Returns verbose event details that this library transforms
    /// into minimal TriggerSubscriptions for the Event Framework.
    fn get_product_events(&self, product: &Product) -> ActionResult<ProductEventDetails>;

    /// Calculate the action for a product given trigger info (Call 1).
    fn calculate_action(
        &self,
        product: &Product,
        trigger_info: &TriggerInfo,
    ) -> ActionResult<ProductAction>;
}

/// ProductActionCalculator trait - the Call 1 interface.
///
/// This is the main interface for calculating ProductActions from triggers.
/// It is stateless and receives all data it needs as parameters.
pub trait ProductActionCalculator: Send + Sync {
    /// Calculate a ProductAction from trigger info and product.
    ///
    /// This is "Call 1" in the two-call pattern:
    /// - Input: trigger info + product definition
    /// - Calls: QuantLib stub for product-level calculations
    /// - Output: ProductAction with unit economics (quantity=1)
    ///
    /// The returned ProductAction represents what happened to the product,
    /// not how it should be booked (that's Call 2's responsibility).
    fn get_product_action(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
        quant_lib: &dyn QuantLib,
    ) -> ActionResult<ProductAction>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::ProductDetails;
    use crate::ledger::types::TagCriterion;
    use chrono::TimeZone;

    // Test 1: ActionType enum serialization
    #[test]
    fn test_action_type_enum_serialization() {
        // Test all 9 action types
        let action_types = vec![
            ActionType::CouponPayment,
            ActionType::Settlement,
            ActionType::BarrierBreach,
            ActionType::EarlyExercise,
            ActionType::DividendAdjustment,
            ActionType::StockSplitAdjustment,
            ActionType::FixingObserved,
            ActionType::HedgeUnwind,
            ActionType::HedgeRebalance,
        ];

        for action_type in &action_types {
            // JSON roundtrip
            let json = serde_json::to_string(action_type).unwrap();
            let deserialized: ActionType = serde_json::from_str(&json).unwrap();
            assert_eq!(*action_type, deserialized);

            // from_i32 roundtrip
            let value = *action_type as i32;
            assert_eq!(ActionType::from_i32(value), Some(*action_type));

            // as_str is non-empty
            assert!(!action_type.as_str().is_empty());
        }

        // Verify all 9 types have unique discriminants
        let discriminants: Vec<i32> = action_types.iter().map(|a| *a as i32).collect();
        let unique: std::collections::HashSet<i32> = discriminants.iter().cloned().collect();
        assert_eq!(unique.len(), 9, "All 9 action types must have unique discriminants");

        // Invalid from_i32
        assert_eq!(ActionType::from_i32(0), None);
        assert_eq!(ActionType::from_i32(10), None);

        // Test has_downstream_actions
        assert!(ActionType::BarrierBreach.has_downstream_actions());
        assert!(ActionType::EarlyExercise.has_downstream_actions());
        assert!(ActionType::Settlement.has_downstream_actions());
        assert!(!ActionType::CouponPayment.has_downstream_actions());
        assert!(!ActionType::FixingObserved.has_downstream_actions());
    }

    // Test 2: ProductAction construction with all fields
    #[test]
    fn test_product_action_construction_with_all_fields() {
        let product_id = ProductId::new("KO-001");
        let settlement_date = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let amount = Decimal::new(10050, 2); // 100.50

        let details = ActionDetails::Settlement {
            settlement_type: SettlementType::Cash,
            reason: SettlementReason::BarrierBreach,
        };

        let source_ref = ProductActionRef::new("BARRIER-ACTION-001");

        let action = ProductAction::new("SETTLE-001", ActionType::Settlement, product_id.clone())
            .with_amount(amount)
            .with_settlement_date(settlement_date)
            .with_details(details.clone())
            .with_source_action(source_ref.clone())
            .with_target_selector(PositionSelector::SameAsSource);

        assert_eq!(action.id.as_str(), "SETTLE-001");
        assert_eq!(action.action_type, ActionType::Settlement);
        assert_eq!(action.product_id, product_id);
        assert_eq!(action.amount_per_unit, Some(amount));
        assert_eq!(action.settlement_date, Some(settlement_date));
        assert_eq!(action.details, details);
        assert_eq!(action.source_action_ref, Some(source_ref));
        assert_eq!(action.target_selector, Some(PositionSelector::SameAsSource));
        assert!(action.is_chained());
        assert_eq!(action.lineage_depth(), 1);

        // Test action without source (root action)
        let root_action = ProductAction::new("COUPON-001", ActionType::CouponPayment, product_id);
        assert!(!root_action.is_chained());
        assert_eq!(root_action.lineage_depth(), 0);

        // Test JSON roundtrip
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: ProductAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action, deserialized);
    }

    // Test 3: PositionSelector enum variants
    #[test]
    fn test_position_selector_enum_variants() {
        // SameAsSource (default)
        let selector = PositionSelector::default();
        assert_eq!(selector, PositionSelector::SameAsSource);

        // LinkedHedges
        let product_id = ProductId::new("BRC-001");
        let selector = PositionSelector::linked_hedges(product_id.clone());
        if let PositionSelector::LinkedHedges { source_product_id } = &selector {
            assert_eq!(*source_product_id, product_id);
        } else {
            panic!("Expected LinkedHedges variant");
        }

        // Specific positions
        let position_ids = vec![
            PositionId::new("POS-001"),
            PositionId::new("POS-002"),
        ];
        let selector = PositionSelector::specific(position_ids.clone());
        if let PositionSelector::Specific { position_ids: ids } = &selector {
            assert_eq!(*ids, position_ids);
        } else {
            panic!("Expected Specific variant");
        }

        // ByQuery
        let filter = PortfolioFilter::new("Hedge Positions")
            .with_criterion(TagCriterion::Equals("type".to_string(), "hedge".to_string()));
        let selector = PositionSelector::by_query(filter.clone());
        if let PositionSelector::ByQuery { filter: f } = &selector {
            assert_eq!(*f, filter);
        } else {
            panic!("Expected ByQuery variant");
        }

        // JSON serialization roundtrip for all variants
        let selectors = vec![
            PositionSelector::SameAsSource,
            PositionSelector::linked_hedges(ProductId::new("P-001")),
            PositionSelector::specific(vec![PositionId::new("POS-001")]),
            PositionSelector::by_query(PortfolioFilter::new("Test")),
        ];

        for selector in selectors {
            let json = serde_json::to_string(&selector).unwrap();
            let deserialized: PositionSelector = serde_json::from_str(&json).unwrap();
            assert_eq!(selector, deserialized);
        }
    }

    // Test 4: ProductActionRef lineage tracking
    #[test]
    fn test_product_action_ref_lineage_tracking() {
        let product_id = ProductId::new("KO-001");

        // Create a chain of actions: Barrier Breach -> Settlement -> HedgeUnwind
        // Step 1: Root barrier breach action
        let barrier_action = ProductAction::new(
            "ACTION-001-BARRIER",
            ActionType::BarrierBreach,
            product_id.clone(),
        )
        .with_amount(Decimal::ZERO)
        .with_details(ActionDetails::Barrier {
            barrier_level: Decimal::new(9000, 2),
            breach_price: Decimal::new(8950, 2),
            is_knockout: true,
        });

        assert!(!barrier_action.is_chained());
        assert_eq!(barrier_action.lineage_depth(), 0);
        assert!(barrier_action.source_action_ref.is_none());

        // Step 2: Settlement action triggered by barrier breach
        let settlement_action = ProductAction::new(
            "ACTION-002-SETTLE",
            ActionType::Settlement,
            product_id.clone(),
        )
        .with_source_action(barrier_action.id.clone())
        .with_amount(Decimal::new(8950, 2))
        .with_settlement_date(NaiveDate::from_ymd_opt(2024, 12, 20).unwrap())
        .with_details(ActionDetails::Settlement {
            settlement_type: SettlementType::Cash,
            reason: SettlementReason::BarrierBreach,
        })
        .with_target_selector(PositionSelector::SameAsSource);

        assert!(settlement_action.is_chained());
        assert_eq!(settlement_action.lineage_depth(), 1);
        assert_eq!(
            settlement_action.source_action_ref,
            Some(barrier_action.id.clone())
        );

        // Step 3: Hedge unwind triggered by settlement
        let hedge_unwind_action = ProductAction::new(
            "ACTION-003-HEDGE",
            ActionType::HedgeUnwind,
            product_id.clone(),
        )
        .with_source_action(settlement_action.id.clone())
        .with_details(ActionDetails::HedgeUnwind {
            reason: "Product knocked out".to_string(),
            unwind_percentage: Decimal::ONE,
        })
        .with_target_selector(PositionSelector::linked_hedges(product_id.clone()));

        assert!(hedge_unwind_action.is_chained());
        assert_eq!(hedge_unwind_action.lineage_depth(), 1);
        assert_eq!(
            hedge_unwind_action.source_action_ref,
            Some(settlement_action.id.clone())
        );

        // Verify lineage chain integrity
        assert_eq!(barrier_action.id.as_str(), "ACTION-001-BARRIER");
        assert_eq!(
            settlement_action.source_action_ref.as_ref().unwrap().as_str(),
            "ACTION-001-BARRIER"
        );
        assert_eq!(
            hedge_unwind_action.source_action_ref.as_ref().unwrap().as_str(),
            "ACTION-002-SETTLE"
        );
    }

    // Test 5: ActionDetails variants
    #[test]
    fn test_action_details_variants() {
        // Coupon details
        let coupon = ActionDetails::Coupon {
            coupon_rate: Decimal::new(500, 4), // 5%
            period_start: NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(),
        };
        let json = serde_json::to_string(&coupon).unwrap();
        let deserialized: ActionDetails = serde_json::from_str(&json).unwrap();
        assert_eq!(coupon, deserialized);

        // Barrier details
        let barrier = ActionDetails::Barrier {
            barrier_level: Decimal::new(9000, 2),
            breach_price: Decimal::new(8950, 2),
            is_knockout: true,
        };
        let json = serde_json::to_string(&barrier).unwrap();
        let deserialized: ActionDetails = serde_json::from_str(&json).unwrap();
        assert_eq!(barrier, deserialized);

        // Exercise details
        let exercise = ActionDetails::Exercise {
            strike_price: Decimal::new(15000, 2),
            spot_price: Decimal::new(16500, 2),
            is_call: true,
        };
        let json = serde_json::to_string(&exercise).unwrap();
        let deserialized: ActionDetails = serde_json::from_str(&json).unwrap();
        assert_eq!(exercise, deserialized);

        // Dividend details
        let dividend = ActionDetails::Dividend {
            dividend_per_share: Decimal::new(50, 2),
            adjustment_factor: Decimal::new(9950, 4), // 0.995
            ex_date: NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
        };
        let json = serde_json::to_string(&dividend).unwrap();
        let deserialized: ActionDetails = serde_json::from_str(&json).unwrap();
        assert_eq!(dividend, deserialized);

        // Stock split details
        let split = ActionDetails::StockSplit {
            new_shares: 4,
            old_shares: 1,
            adjustment_factor: Decimal::new(25, 2), // 0.25
        };
        let json = serde_json::to_string(&split).unwrap();
        let deserialized: ActionDetails = serde_json::from_str(&json).unwrap();
        assert_eq!(split, deserialized);

        // Default is None
        assert_eq!(ActionDetails::default(), ActionDetails::None);
    }

    // Test 6: ProductAction protobuf serialization
    #[test]
    fn test_product_action_protobuf_serialization() {
        let product_id = ProductId::new("MINI-001");
        let settlement_date = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let amount = Decimal::new(15025, 2); // 150.25

        let details = ActionDetails::Settlement {
            settlement_type: SettlementType::Cash,
            reason: SettlementReason::BarrierBreach,
        };

        let action = ProductAction::new("ACTION-001", ActionType::Settlement, product_id)
            .with_amount(amount)
            .with_settlement_date(settlement_date)
            .with_details(details)
            .with_source_action(ProductActionRef::new("PARENT-001"))
            .with_target_selector(PositionSelector::SameAsSource);

        // Encode to protobuf
        let buf = action.encode_proto();
        assert!(!buf.is_empty());

        // Decode the proto struct
        let proto = ProductActionProto::decode(&buf[..]).unwrap();
        assert_eq!(proto.id, "ACTION-001");
        assert_eq!(proto.action_type, 2); // Settlement
        assert_eq!(proto.product_id, "MINI-001");
        assert_eq!(proto.amount_per_unit, Some("150.25".to_string()));
        assert_eq!(proto.settlement_date, Some("2024-12-20".to_string()));
        assert_eq!(proto.source_action_ref, Some("PARENT-001".to_string()));
        assert_eq!(proto.target_selector_type, 1); // SameAsSource

        // Verify details JSON
        let details_parsed: ActionDetails = serde_json::from_str(&proto.details_json).unwrap();
        if let ActionDetails::Settlement { settlement_type, reason } = details_parsed {
            assert_eq!(settlement_type, SettlementType::Cash);
            assert_eq!(reason, SettlementReason::BarrierBreach);
        } else {
            panic!("Expected Settlement details");
        }
    }

    // =========================================================================
    // Call 0 Tests: Product Registration and Trigger Subscriptions
    // =========================================================================

    // Test 7: TriggerSubscription construction
    #[test]
    fn test_trigger_subscription_construction() {
        let product_id = ProductId::new("KO-001");

        // Continuous barrier subscription
        let barrier_sub = TriggerSubscription::continuous_barrier(
            product_id.clone(),
            "AAPL",
            Decimal::new(9000, 2),
            BarrierDirection::Down,
        );
        assert_eq!(barrier_sub.product_id, product_id);
        assert_eq!(barrier_sub.trigger_type, TriggerType::ContinuousBarrier);
        assert_eq!(barrier_sub.underlying, Some("AAPL".to_string()));
        if let SubscriptionCriteria::Barrier { level, direction } = barrier_sub.criteria {
            assert_eq!(level, Decimal::new(9000, 2));
            assert_eq!(direction, BarrierDirection::Down);
        } else {
            panic!("Expected Barrier criteria");
        }

        // Fixing subscription
        let fixing_time = Utc.with_ymd_and_hms(2024, 12, 15, 9, 0, 0).unwrap();
        let fixing_sub = TriggerSubscription::fixing(
            product_id.clone(),
            "AAPL",
            FixingType::Initial,
            fixing_time,
        );
        assert_eq!(fixing_sub.trigger_type, TriggerType::Fixing(FixingType::Initial));
        if let SubscriptionCriteria::Scheduled { time } = fixing_sub.criteria {
            assert_eq!(time, fixing_time);
        } else {
            panic!("Expected Scheduled criteria");
        }

        // Expiry subscription
        let expiry_time = Utc.with_ymd_and_hms(2024, 12, 20, 17, 30, 0).unwrap();
        let expiry_sub = TriggerSubscription::expiry(product_id.clone(), expiry_time);
        assert_eq!(expiry_sub.trigger_type, TriggerType::Expiry);
        assert!(expiry_sub.underlying.is_none());

        // Coupon subscription
        let coupon_time = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
        let coupon_sub = TriggerSubscription::coupon(product_id.clone(), coupon_time);
        assert_eq!(coupon_sub.trigger_type, TriggerType::CouponPayment);

        // JSON roundtrip
        let json = serde_json::to_string(&barrier_sub).unwrap();
        let deserialized: TriggerSubscription = serde_json::from_str(&json).unwrap();
        assert_eq!(barrier_sub, deserialized);
    }

    // Test 8: ProductEventDetails construction and translation
    #[test]
    fn test_product_event_details_to_subscriptions() {
        let product_id = ProductId::new("KO-001");
        let barrier_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let barrier_end = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let fixing_time = Utc.with_ymd_and_hms(2024, 12, 15, 9, 0, 0).unwrap();
        let expiry_time = Utc.with_ymd_and_hms(2024, 12, 20, 17, 30, 0).unwrap();

        // Build verbose event details (what QuantLib returns)
        let events = ProductEventDetails::new(product_id.clone())
            .with_barrier(BarrierEventDetail::continuous(
                Decimal::new(9000, 2),
                BarrierDirection::Down,
                "AAPL",
                barrier_start,
                barrier_end,
            ))
            .with_fixing(FixingEventDetail::new(
                FixingType::Initial,
                fixing_time,
                "AAPL",
            ))
            .with_expiry(ExpiryEventDetail::new(
                expiry_time,
                SettlementType::Cash,
            ))
            .with_corporate_action_sensitivity(true);

        // Translate to minimal subscriptions
        let subscriptions = translate_to_subscriptions(&events);

        // Should have: barrier, fixing, expiry, dividend, stock_split
        assert_eq!(subscriptions.len(), 5);

        // Verify barrier subscription
        let barrier_sub = subscriptions.iter()
            .find(|s| s.trigger_type == TriggerType::ContinuousBarrier)
            .expect("Should have barrier subscription");
        assert_eq!(barrier_sub.underlying, Some("AAPL".to_string()));
        if let SubscriptionCriteria::Barrier { level, direction } = &barrier_sub.criteria {
            assert_eq!(*level, Decimal::new(9000, 2));
            assert_eq!(*direction, BarrierDirection::Down);
        }

        // Verify fixing subscription
        let fixing_sub = subscriptions.iter()
            .find(|s| matches!(s.trigger_type, TriggerType::Fixing(_)))
            .expect("Should have fixing subscription");
        assert_eq!(fixing_sub.underlying, Some("AAPL".to_string()));

        // Verify expiry subscription
        let expiry_sub = subscriptions.iter()
            .find(|s| s.trigger_type == TriggerType::Expiry)
            .expect("Should have expiry subscription");
        if let SubscriptionCriteria::Scheduled { time } = &expiry_sub.criteria {
            assert_eq!(*time, expiry_time);
        }

        // Verify corporate action subscriptions
        assert!(subscriptions.iter().any(|s| s.trigger_type == TriggerType::Dividend));
        assert!(subscriptions.iter().any(|s| s.trigger_type == TriggerType::StockSplit));
    }

    // Test 9: BarrierEventDetail discrete vs continuous
    #[test]
    fn test_barrier_event_detail_types() {
        // Continuous barrier
        let continuous = BarrierEventDetail::continuous(
            Decimal::new(9000, 2),
            BarrierDirection::Down,
            "AAPL",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        );
        assert_eq!(continuous.observation_type, BarrierObservationType::Continuous);
        assert!(continuous.observation_times.is_empty());

        // Discrete barrier
        let obs_times = vec![
            Utc.with_ymd_and_hms(2024, 3, 15, 17, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 6, 15, 17, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 9, 15, 17, 0, 0).unwrap(),
        ];
        let discrete = BarrierEventDetail::discrete(
            Decimal::new(9000, 2),
            BarrierDirection::Down,
            "AAPL",
            obs_times.clone(),
        );
        assert_eq!(discrete.observation_type, BarrierObservationType::Discrete);
        assert_eq!(discrete.observation_times.len(), 3);

        // Translation of discrete barrier
        let events = ProductEventDetails::new(ProductId::new("KO-001"))
            .with_barrier(discrete);
        let subscriptions = translate_to_subscriptions(&events);

        assert_eq!(subscriptions.len(), 1);
        let sub = &subscriptions[0];
        assert_eq!(sub.trigger_type, TriggerType::Barrier);
        if let SubscriptionCriteria::DiscreteBarrier { observation_times, .. } = &sub.criteria {
            assert_eq!(observation_times.len(), 3);
        } else {
            panic!("Expected DiscreteBarrier criteria");
        }
    }

    // Test 10: Full Call 0 flow simulation
    #[test]
    fn test_call_0_full_flow() {
        let product_id = ProductId::new("MINI-001");

        // Simulate QuantLib returning verbose event details
        let events = ProductEventDetails::new(product_id.clone())
            .with_barrier(
                BarrierEventDetail::continuous(
                    Decimal::new(14000, 2),
                    BarrierDirection::Down,
                    "DAX",
                    NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                )
                .with_knockout(true)
                .with_rebate(Decimal::new(500, 2))
            )
            .with_fixing(
                FixingEventDetail::new(
                    FixingType::Initial,
                    Utc.with_ymd_and_hms(2024, 1, 15, 9, 0, 0).unwrap(),
                    "DAX",
                )
            )
            .with_fixing(
                FixingEventDetail::new(
                    FixingType::Final,
                    Utc.with_ymd_and_hms(2025, 1, 15, 9, 0, 0).unwrap(),
                    "DAX",
                )
            )
            .with_expiry(
                ExpiryEventDetail::new(
                    Utc.with_ymd_and_hms(2025, 1, 15, 17, 30, 0).unwrap(),
                    SettlementType::Cash,
                )
                .with_settlement_delay(2)
                .with_auto_exercise(true)
            )
            .with_corporate_action_sensitivity(true);

        // This library transforms to minimal subscriptions
        let subscriptions = translate_to_subscriptions(&events);

        // Verify we have the right subscriptions
        // 1 barrier + 2 fixings + 1 expiry + 2 corp actions = 6
        assert_eq!(subscriptions.len(), 6);

        // Verify barrier subscription is minimal (no rebate info - Event Framework doesn't need it)
        let barrier_sub = subscriptions.iter()
            .find(|s| s.trigger_type == TriggerType::ContinuousBarrier)
            .unwrap();

        // The subscription only has what Event Framework needs to monitor
        assert_eq!(barrier_sub.underlying, Some("DAX".to_string()));
        if let SubscriptionCriteria::Barrier { level, direction } = &barrier_sub.criteria {
            assert_eq!(*level, Decimal::new(14000, 2));
            assert_eq!(*direction, BarrierDirection::Down);
            // Note: rebate is NOT in the subscription - that's quant detail
        }

        // Verify all subscriptions are JSON serializable
        for sub in &subscriptions {
            let json = serde_json::to_string(sub).unwrap();
            assert!(!json.is_empty());
            let _: TriggerSubscription = serde_json::from_str(&json).unwrap();
        }
    }
}
