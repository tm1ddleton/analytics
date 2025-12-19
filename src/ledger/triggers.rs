//! Trigger Type System for the Quantitative Event Engine.
//!
//! This module defines the trigger types that drive lifecycle events for structured products:
//! - TriggerType: 8 variants covering time-based, price-based, exercise, and corporate actions
//! - FixingType: Subtypes for price observation triggers
//! - TriggerInfo: Payload struct containing all data needed for callbacks
//! - Product-trigger compatibility matrix

use chrono::{DateTime, Utc};
use prost::Message;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::domain::{Product, ProductType};

/// Fixing subtype for price observation triggers.
///
/// Represents different stages in a product's fixing lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum FixingType {
    /// Initial fixing - sets the strike price at product inception
    Initial = 1,
    /// Final fixing - determines settlement value at maturity
    Final = 2,
    /// Periodic fixing - regular observation points (e.g., autocall dates)
    Periodic = 3,
    /// Reset fixing - adjusts product parameters (e.g., factor certificate reset)
    Reset = 4,
}

impl FixingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FixingType::Initial => "Initial",
            FixingType::Final => "Final",
            FixingType::Periodic => "Periodic",
            FixingType::Reset => "Reset",
        }
    }

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(FixingType::Initial),
            2 => Some(FixingType::Final),
            3 => Some(FixingType::Periodic),
            4 => Some(FixingType::Reset),
            _ => None,
        }
    }
}

/// Barrier direction for barrier observation triggers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum BarrierDirection {
    /// Up barrier - triggered when price rises above level
    Up = 1,
    /// Down barrier - triggered when price falls below level
    Down = 2,
}

impl BarrierDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            BarrierDirection::Up => "Up",
            BarrierDirection::Down => "Down",
        }
    }

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(BarrierDirection::Up),
            2 => Some(BarrierDirection::Down),
            _ => None,
        }
    }

    /// Check if a price has breached the barrier level in this direction.
    pub fn is_breached(&self, price: Decimal, barrier_level: Decimal) -> bool {
        match self {
            BarrierDirection::Up => price >= barrier_level,
            BarrierDirection::Down => price <= barrier_level,
        }
    }
}

/// Trigger type enum with 8 variants covering all lifecycle events.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum TriggerType {
    // Time-based triggers
    /// Product expiry/maturity
    Expiry = 1,
    /// Scheduled coupon payment date
    CouponPayment = 2,

    // Price observation triggers
    /// Price fixing observation (with subtype for lifecycle stage)
    Fixing(FixingType) = 3,
    /// Discrete barrier check at specific times
    Barrier = 4,
    /// Continuous barrier monitoring (real-time tick-level)
    ContinuousBarrier = 5,

    // Exercise triggers
    /// American-style early exercise (primarily for vanilla options)
    AmericanExercise = 6,

    // Corporate action triggers
    /// Dividend payment on underlying
    Dividend = 7,
    /// Stock split on underlying
    StockSplit = 8,
}

impl TriggerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TriggerType::Expiry => "Expiry",
            TriggerType::CouponPayment => "CouponPayment",
            TriggerType::Fixing(_) => "Fixing",
            TriggerType::Barrier => "Barrier",
            TriggerType::ContinuousBarrier => "ContinuousBarrier",
            TriggerType::AmericanExercise => "AmericanExercise",
            TriggerType::Dividend => "Dividend",
            TriggerType::StockSplit => "StockSplit",
        }
    }

    /// Returns the category of this trigger type.
    pub fn category(&self) -> TriggerCategory {
        match self {
            TriggerType::Expiry | TriggerType::CouponPayment => TriggerCategory::TimeBased,
            TriggerType::Fixing(_) | TriggerType::Barrier | TriggerType::ContinuousBarrier => {
                TriggerCategory::PriceBased
            }
            TriggerType::AmericanExercise => TriggerCategory::Exercise,
            TriggerType::Dividend | TriggerType::StockSplit => TriggerCategory::CorporateAction,
        }
    }

    /// Returns the discriminant value for serialization.
    pub fn discriminant(&self) -> i32 {
        match self {
            TriggerType::Expiry => 1,
            TriggerType::CouponPayment => 2,
            TriggerType::Fixing(_) => 3,
            TriggerType::Barrier => 4,
            TriggerType::ContinuousBarrier => 5,
            TriggerType::AmericanExercise => 6,
            TriggerType::Dividend => 7,
            TriggerType::StockSplit => 8,
        }
    }
}

/// Trigger category for grouping trigger types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerCategory {
    TimeBased,
    PriceBased,
    Exercise,
    CorporateAction,
}

impl TriggerCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            TriggerCategory::TimeBased => "TimeBased",
            TriggerCategory::PriceBased => "PriceBased",
            TriggerCategory::Exercise => "Exercise",
            TriggerCategory::CorporateAction => "CorporateAction",
        }
    }
}

/// Corporate action details for dividend and stock split triggers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorporateActionDetails {
    /// Type of corporate action
    pub action_type: CorporateActionType,
    /// Ex-date for the corporate action
    pub ex_date: DateTime<Utc>,
    /// Record date for the corporate action
    pub record_date: Option<DateTime<Utc>>,
    /// Payment date for dividends
    pub payment_date: Option<DateTime<Utc>>,
}

/// Type of corporate action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorporateActionType {
    /// Cash dividend with amount per share
    CashDividend { amount: Decimal, currency: [u8; 3] },
    /// Stock dividend with shares per share ratio
    StockDividend { ratio: Decimal },
    /// Stock split with new:old ratio
    StockSplit { new_shares: u32, old_shares: u32 },
    /// Reverse stock split with old:new ratio
    ReverseSplit { old_shares: u32, new_shares: u32 },
}

/// Trigger information payload passed by the Event Framework to this library.
///
/// Contains all data needed for the callback to process a lifecycle event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerInfo {
    /// The type of trigger that fired
    pub trigger_type: TriggerType,

    /// When the trigger fired
    pub trigger_time: DateTime<Utc>,

    /// Observed price for fixings (passed by Event Framework)
    pub observed_price: Option<Decimal>,

    /// Whether this is a provisional calculation (for index integration)
    pub is_provisional: bool,

    /// Barrier level for barrier triggers
    pub barrier_level: Option<Decimal>,

    /// Barrier direction for barrier triggers
    pub barrier_direction: Option<BarrierDirection>,

    /// Corporate action details for dividend/split triggers
    pub corporate_action_details: Option<CorporateActionDetails>,
}

impl TriggerInfo {
    /// Create a new time-based trigger (Expiry or CouponPayment).
    pub fn new_time_based(trigger_type: TriggerType, trigger_time: DateTime<Utc>) -> Self {
        TriggerInfo {
            trigger_type,
            trigger_time,
            observed_price: None,
            is_provisional: false,
            barrier_level: None,
            barrier_direction: None,
            corporate_action_details: None,
        }
    }

    /// Create a new fixing trigger.
    pub fn new_fixing(
        fixing_type: FixingType,
        trigger_time: DateTime<Utc>,
        observed_price: Decimal,
    ) -> Self {
        TriggerInfo {
            trigger_type: TriggerType::Fixing(fixing_type),
            trigger_time,
            observed_price: Some(observed_price),
            is_provisional: false,
            barrier_level: None,
            barrier_direction: None,
            corporate_action_details: None,
        }
    }

    /// Create a new barrier trigger.
    pub fn new_barrier(
        continuous: bool,
        trigger_time: DateTime<Utc>,
        observed_price: Decimal,
        barrier_level: Decimal,
        barrier_direction: BarrierDirection,
    ) -> Self {
        TriggerInfo {
            trigger_type: if continuous {
                TriggerType::ContinuousBarrier
            } else {
                TriggerType::Barrier
            },
            trigger_time,
            observed_price: Some(observed_price),
            is_provisional: false,
            barrier_level: Some(barrier_level),
            barrier_direction: Some(barrier_direction),
            corporate_action_details: None,
        }
    }

    /// Create a new exercise trigger.
    pub fn new_exercise(trigger_time: DateTime<Utc>, observed_price: Decimal) -> Self {
        TriggerInfo {
            trigger_type: TriggerType::AmericanExercise,
            trigger_time,
            observed_price: Some(observed_price),
            is_provisional: false,
            barrier_level: None,
            barrier_direction: None,
            corporate_action_details: None,
        }
    }

    /// Create a new corporate action trigger.
    pub fn new_corporate_action(
        trigger_type: TriggerType,
        trigger_time: DateTime<Utc>,
        details: CorporateActionDetails,
    ) -> Self {
        TriggerInfo {
            trigger_type,
            trigger_time,
            observed_price: None,
            is_provisional: false,
            barrier_level: None,
            barrier_direction: None,
            corporate_action_details: Some(details),
        }
    }

    /// Mark this trigger as provisional (for index integration).
    pub fn with_provisional(mut self, is_provisional: bool) -> Self {
        self.is_provisional = is_provisional;
        self
    }
}

/// Protobuf representation for TriggerInfo.
#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct TriggerInfoProto {
    /// Trigger type discriminant
    #[prost(int32, tag = "1")]
    pub trigger_type: i32,

    /// Fixing subtype (if applicable)
    #[prost(int32, optional, tag = "2")]
    pub fixing_type: Option<i32>,

    /// Trigger time as RFC3339 string
    #[prost(string, tag = "3")]
    pub trigger_time: String,

    /// Observed price as string (for decimal precision)
    #[prost(string, optional, tag = "4")]
    pub observed_price: Option<String>,

    /// Provisional flag
    #[prost(bool, tag = "5")]
    pub is_provisional: bool,

    /// Barrier level as string
    #[prost(string, optional, tag = "6")]
    pub barrier_level: Option<String>,

    /// Barrier direction discriminant
    #[prost(int32, optional, tag = "7")]
    pub barrier_direction: Option<i32>,
}

impl TriggerInfo {
    /// Convert to protobuf representation.
    pub fn to_proto(&self) -> TriggerInfoProto {
        let (trigger_type, fixing_type) = match &self.trigger_type {
            TriggerType::Fixing(ft) => (3, Some(*ft as i32)),
            tt => (tt.discriminant(), None),
        };

        TriggerInfoProto {
            trigger_type,
            fixing_type,
            trigger_time: self.trigger_time.to_rfc3339(),
            observed_price: self.observed_price.map(|p| p.to_string()),
            is_provisional: self.is_provisional,
            barrier_level: self.barrier_level.map(|l| l.to_string()),
            barrier_direction: self.barrier_direction.map(|d| d as i32),
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

/// Product-trigger compatibility matrix.
///
/// Defines which trigger types apply to which products.
#[derive(Debug, Clone)]
pub struct TriggerProductMatrix {
    /// Map from ProductType to applicable trigger types
    triggers_by_product: std::collections::HashMap<ProductType, HashSet<TriggerType>>,
}

impl Default for TriggerProductMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl TriggerProductMatrix {
    /// Create a new matrix with default product-trigger mappings.
    pub fn new() -> Self {
        use ProductType::*;
        use TriggerType::*;

        let mut triggers_by_product = std::collections::HashMap::new();

        // Thin slice products (continuous barrier)
        let continuous_barrier_triggers: HashSet<TriggerType> = [
            Expiry,
            Fixing(FixingType::Initial),
            Fixing(FixingType::Final),
            ContinuousBarrier,
            Dividend,
            StockSplit,
        ]
        .into_iter()
        .collect();

        triggers_by_product.insert(KnockOutWarrant, continuous_barrier_triggers.clone());
        triggers_by_product.insert(MiniCertificate, continuous_barrier_triggers.clone());
        triggers_by_product.insert(OpenEndTurbo, continuous_barrier_triggers.clone());

        // Barrier reverse convertible products (discrete barrier + coupon)
        let brc_triggers: HashSet<TriggerType> = [
            Expiry,
            CouponPayment,
            Fixing(FixingType::Initial),
            Fixing(FixingType::Final),
            Fixing(FixingType::Periodic),
            Barrier,
            Dividend,
            StockSplit,
        ]
        .into_iter()
        .collect();

        triggers_by_product.insert(BarrierReverseConvertible, brc_triggers.clone());
        triggers_by_product.insert(BarrierReverseConvertiblePro, brc_triggers.clone());
        triggers_by_product.insert(ReverseConvertible, brc_triggers.clone());

        // Bonus certificate products (discrete barrier)
        let bonus_triggers: HashSet<TriggerType> = [
            Expiry,
            Fixing(FixingType::Initial),
            Fixing(FixingType::Final),
            Barrier,
            Dividend,
            StockSplit,
        ]
        .into_iter()
        .collect();

        triggers_by_product.insert(BonusCertificate, bonus_triggers.clone());
        triggers_by_product.insert(CappedBonusCertificate, bonus_triggers.clone());
        triggers_by_product.insert(CappedBonusProCertificate, bonus_triggers.clone());
        triggers_by_product.insert(ReverseCappedBonusCertificate, bonus_triggers.clone());

        // Warrant products
        let warrant_triggers: HashSet<TriggerType> = [
            Expiry,
            Fixing(FixingType::Initial),
            Fixing(FixingType::Final),
            Dividend,
            StockSplit,
        ]
        .into_iter()
        .collect();

        triggers_by_product.insert(CappedWarrant, warrant_triggers.clone());
        triggers_by_product.insert(DiscountCertificate, warrant_triggers.clone());

        // Factor certificate (daily reset)
        let factor_triggers: HashSet<TriggerType> = [
            Expiry,
            Fixing(FixingType::Initial),
            Fixing(FixingType::Reset),
            Dividend,
            StockSplit,
        ]
        .into_iter()
        .collect();

        triggers_by_product.insert(FactorCertificate, factor_triggers);

        // Vanilla option (American exercise)
        let option_triggers: HashSet<TriggerType> = [
            Expiry,
            Fixing(FixingType::Initial),
            Fixing(FixingType::Final),
            AmericanExercise,
            Dividend,
            StockSplit,
        ]
        .into_iter()
        .collect();

        triggers_by_product.insert(VanillaOption, option_triggers);

        TriggerProductMatrix { triggers_by_product }
    }

    /// Check if a trigger type is applicable to a product.
    pub fn is_applicable(&self, product_type: ProductType, trigger_type: &TriggerType) -> bool {
        self.triggers_by_product
            .get(&product_type)
            .map(|triggers| triggers.contains(trigger_type))
            .unwrap_or(false)
    }

    /// Get all applicable trigger types for a product.
    pub fn triggers_for_product(&self, product_type: ProductType) -> Vec<TriggerType> {
        self.triggers_by_product
            .get(&product_type)
            .map(|triggers| triggers.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all products that a trigger type applies to.
    pub fn products_for_trigger(&self, trigger_type: &TriggerType) -> Vec<ProductType> {
        self.triggers_by_product
            .iter()
            .filter(|(_, triggers)| triggers.contains(trigger_type))
            .map(|(product_type, _)| *product_type)
            .collect()
    }

    /// Check if a trigger applies to a specific product instance.
    pub fn is_applicable_to_product(&self, product: &Product, trigger_type: &TriggerType) -> bool {
        self.is_applicable(product.product_type_enum(), trigger_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // Test 1: TriggerType enum serialization roundtrip
    #[test]
    fn test_trigger_type_serialization_roundtrip() {
        // Test all 8 trigger type variants
        let trigger_types = vec![
            TriggerType::Expiry,
            TriggerType::CouponPayment,
            TriggerType::Fixing(FixingType::Initial),
            TriggerType::Fixing(FixingType::Final),
            TriggerType::Barrier,
            TriggerType::ContinuousBarrier,
            TriggerType::AmericanExercise,
            TriggerType::Dividend,
            TriggerType::StockSplit,
        ];

        for trigger_type in trigger_types {
            // JSON roundtrip
            let json = serde_json::to_string(&trigger_type).unwrap();
            let deserialized: TriggerType = serde_json::from_str(&json).unwrap();
            assert_eq!(trigger_type, deserialized);

            // Verify discriminant is unique
            assert!(trigger_type.discriminant() >= 1 && trigger_type.discriminant() <= 8);
        }

        // Verify all 8 base types have unique discriminants
        let discriminants: Vec<i32> = vec![
            TriggerType::Expiry.discriminant(),
            TriggerType::CouponPayment.discriminant(),
            TriggerType::Fixing(FixingType::Initial).discriminant(),
            TriggerType::Barrier.discriminant(),
            TriggerType::ContinuousBarrier.discriminant(),
            TriggerType::AmericanExercise.discriminant(),
            TriggerType::Dividend.discriminant(),
            TriggerType::StockSplit.discriminant(),
        ];
        let unique: HashSet<i32> = discriminants.iter().cloned().collect();
        assert_eq!(unique.len(), 8, "All 8 trigger types must have unique discriminants");
    }

    // Test 2: TriggerInfo construction for each category
    #[test]
    fn test_trigger_info_construction_by_category() {
        let trigger_time = Utc.with_ymd_and_hms(2024, 12, 15, 10, 0, 0).unwrap();
        let price = Decimal::new(15025, 2); // 150.25

        // Time-based: Expiry
        let expiry_info = TriggerInfo::new_time_based(TriggerType::Expiry, trigger_time);
        assert_eq!(expiry_info.trigger_type.category(), TriggerCategory::TimeBased);
        assert_eq!(expiry_info.trigger_time, trigger_time);
        assert!(expiry_info.observed_price.is_none());

        // Time-based: CouponPayment
        let coupon_info = TriggerInfo::new_time_based(TriggerType::CouponPayment, trigger_time);
        assert_eq!(coupon_info.trigger_type.category(), TriggerCategory::TimeBased);

        // Price-based: Fixing
        let fixing_info = TriggerInfo::new_fixing(FixingType::Final, trigger_time, price);
        assert_eq!(fixing_info.trigger_type.category(), TriggerCategory::PriceBased);
        assert_eq!(fixing_info.observed_price, Some(price));
        if let TriggerType::Fixing(ft) = fixing_info.trigger_type {
            assert_eq!(ft, FixingType::Final);
        } else {
            panic!("Expected Fixing trigger type");
        }

        // Price-based: Barrier
        let barrier_info = TriggerInfo::new_barrier(
            false,
            trigger_time,
            price,
            Decimal::new(14000, 2),
            BarrierDirection::Down,
        );
        assert_eq!(barrier_info.trigger_type.category(), TriggerCategory::PriceBased);
        assert_eq!(barrier_info.trigger_type, TriggerType::Barrier);
        assert_eq!(barrier_info.barrier_level, Some(Decimal::new(14000, 2)));
        assert_eq!(barrier_info.barrier_direction, Some(BarrierDirection::Down));

        // Price-based: ContinuousBarrier
        let continuous_info = TriggerInfo::new_barrier(
            true,
            trigger_time,
            price,
            Decimal::new(16000, 2),
            BarrierDirection::Up,
        );
        assert_eq!(continuous_info.trigger_type, TriggerType::ContinuousBarrier);

        // Exercise: AmericanExercise
        let exercise_info = TriggerInfo::new_exercise(trigger_time, price);
        assert_eq!(exercise_info.trigger_type.category(), TriggerCategory::Exercise);
        assert_eq!(exercise_info.trigger_type, TriggerType::AmericanExercise);
        assert_eq!(exercise_info.observed_price, Some(price));

        // Corporate Action: Dividend
        let div_details = CorporateActionDetails {
            action_type: CorporateActionType::CashDividend {
                amount: Decimal::new(50, 2),
                currency: *b"USD",
            },
            ex_date: trigger_time,
            record_date: Some(trigger_time),
            payment_date: Some(trigger_time),
        };
        let dividend_info =
            TriggerInfo::new_corporate_action(TriggerType::Dividend, trigger_time, div_details);
        assert_eq!(dividend_info.trigger_type.category(), TriggerCategory::CorporateAction);
        assert!(dividend_info.corporate_action_details.is_some());

        // Corporate Action: StockSplit
        let split_details = CorporateActionDetails {
            action_type: CorporateActionType::StockSplit {
                new_shares: 4,
                old_shares: 1,
            },
            ex_date: trigger_time,
            record_date: None,
            payment_date: None,
        };
        let split_info =
            TriggerInfo::new_corporate_action(TriggerType::StockSplit, trigger_time, split_details);
        assert_eq!(split_info.trigger_type.category(), TriggerCategory::CorporateAction);
    }

    // Test 3: Fixing subtypes (initial, final, periodic, reset)
    #[test]
    fn test_fixing_subtypes() {
        let trigger_time = Utc::now();
        let price = Decimal::new(10000, 2);

        // Test all 4 fixing subtypes
        let fixing_types = vec![
            FixingType::Initial,
            FixingType::Final,
            FixingType::Periodic,
            FixingType::Reset,
        ];

        for fixing_type in fixing_types {
            let info = TriggerInfo::new_fixing(fixing_type, trigger_time, price);

            // Verify trigger type contains correct subtype
            if let TriggerType::Fixing(ft) = info.trigger_type {
                assert_eq!(ft, fixing_type);

                // Test as_str
                match ft {
                    FixingType::Initial => assert_eq!(ft.as_str(), "Initial"),
                    FixingType::Final => assert_eq!(ft.as_str(), "Final"),
                    FixingType::Periodic => assert_eq!(ft.as_str(), "Periodic"),
                    FixingType::Reset => assert_eq!(ft.as_str(), "Reset"),
                }

                // Test from_i32 roundtrip
                let value = ft as i32;
                assert_eq!(FixingType::from_i32(value), Some(ft));
            } else {
                panic!("Expected Fixing trigger type");
            }
        }

        // Verify JSON serialization preserves subtype
        let initial_info = TriggerInfo::new_fixing(FixingType::Initial, trigger_time, price);
        let json = serde_json::to_string(&initial_info).unwrap();
        let deserialized: TriggerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(
            initial_info.trigger_type,
            deserialized.trigger_type,
            "Fixing subtype should survive JSON roundtrip"
        );
    }

    // Test 4: BarrierDirection (Up, Down) behavior
    #[test]
    fn test_barrier_direction_behavior() {
        let barrier_level = Decimal::new(10000, 2); // 100.00

        // Test Down barrier
        let down = BarrierDirection::Down;
        assert_eq!(down.as_str(), "Down");
        assert_eq!(BarrierDirection::from_i32(2), Some(BarrierDirection::Down));

        // Down barrier breached when price <= level
        assert!(down.is_breached(Decimal::new(9999, 2), barrier_level)); // 99.99 <= 100.00
        assert!(down.is_breached(Decimal::new(10000, 2), barrier_level)); // 100.00 <= 100.00
        assert!(!down.is_breached(Decimal::new(10001, 2), barrier_level)); // 100.01 > 100.00

        // Test Up barrier
        let up = BarrierDirection::Up;
        assert_eq!(up.as_str(), "Up");
        assert_eq!(BarrierDirection::from_i32(1), Some(BarrierDirection::Up));

        // Up barrier breached when price >= level
        assert!(up.is_breached(Decimal::new(10001, 2), barrier_level)); // 100.01 >= 100.00
        assert!(up.is_breached(Decimal::new(10000, 2), barrier_level)); // 100.00 >= 100.00
        assert!(!up.is_breached(Decimal::new(9999, 2), barrier_level)); // 99.99 < 100.00

        // Test invalid from_i32
        assert_eq!(BarrierDirection::from_i32(0), None);
        assert_eq!(BarrierDirection::from_i32(3), None);
    }

    // Test 5: TriggerInfo protobuf serialization
    #[test]
    fn test_trigger_info_protobuf_serialization() {
        let trigger_time = Utc.with_ymd_and_hms(2024, 6, 15, 14, 30, 0).unwrap();
        let price = Decimal::new(15025, 2);
        let barrier_level = Decimal::new(14000, 2);

        // Create a barrier trigger with all fields populated
        let info = TriggerInfo::new_barrier(
            true,
            trigger_time,
            price,
            barrier_level,
            BarrierDirection::Down,
        )
        .with_provisional(true);

        // Encode to protobuf
        let buf = info.encode_proto();
        assert!(!buf.is_empty());

        // Decode the proto struct
        let proto = TriggerInfoProto::decode(&buf[..]).unwrap();

        // Verify fields
        assert_eq!(proto.trigger_type, 5); // ContinuousBarrier
        assert!(proto.fixing_type.is_none());
        assert_eq!(proto.observed_price, Some("150.25".to_string()));
        assert_eq!(proto.barrier_level, Some("140.00".to_string()));
        assert_eq!(proto.barrier_direction, Some(2)); // Down
        assert!(proto.is_provisional);
    }

    // Test 6: Product-trigger compatibility matrix
    #[test]
    fn test_product_trigger_matrix() {
        let matrix = TriggerProductMatrix::new();

        // Knock-Out Warrant should have ContinuousBarrier, not Barrier
        assert!(matrix.is_applicable(ProductType::KnockOutWarrant, &TriggerType::ContinuousBarrier));
        assert!(!matrix.is_applicable(ProductType::KnockOutWarrant, &TriggerType::Barrier));
        assert!(matrix.is_applicable(
            ProductType::KnockOutWarrant,
            &TriggerType::Fixing(FixingType::Initial)
        ));
        assert!(matrix.is_applicable(ProductType::KnockOutWarrant, &TriggerType::Expiry));
        assert!(matrix.is_applicable(ProductType::KnockOutWarrant, &TriggerType::Dividend));

        // Barrier Reverse Convertible should have discrete Barrier and CouponPayment
        assert!(matrix.is_applicable(ProductType::BarrierReverseConvertible, &TriggerType::Barrier));
        assert!(!matrix.is_applicable(
            ProductType::BarrierReverseConvertible,
            &TriggerType::ContinuousBarrier
        ));
        assert!(
            matrix.is_applicable(ProductType::BarrierReverseConvertible, &TriggerType::CouponPayment)
        );

        // Vanilla Option should have AmericanExercise
        assert!(matrix.is_applicable(ProductType::VanillaOption, &TriggerType::AmericanExercise));
        assert!(!matrix.is_applicable(ProductType::KnockOutWarrant, &TriggerType::AmericanExercise));

        // Factor Certificate should have Reset fixing
        assert!(matrix.is_applicable(
            ProductType::FactorCertificate,
            &TriggerType::Fixing(FixingType::Reset)
        ));
        assert!(!matrix.is_applicable(
            ProductType::FactorCertificate,
            &TriggerType::Fixing(FixingType::Periodic)
        ));

        // All products should have Expiry
        let products_with_expiry = matrix.products_for_trigger(&TriggerType::Expiry);
        assert!(products_with_expiry.len() >= 10, "Most products should have Expiry trigger");

        // Test triggers_for_product
        let ko_triggers = matrix.triggers_for_product(ProductType::KnockOutWarrant);
        assert!(ko_triggers.contains(&TriggerType::ContinuousBarrier));
        assert!(ko_triggers.contains(&TriggerType::Expiry));
    }
}
