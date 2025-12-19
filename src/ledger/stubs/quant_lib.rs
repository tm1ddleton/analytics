//! Stub implementation of the Quant Library for testing.
//!
//! This module provides a mock QuantLib that returns realistic ProductActions
//! for thin slice products without requiring actual quant library integration.

use rust_decimal::Decimal;
use uuid::Uuid;

use crate::ledger::actions::{
    ActionDetails, ActionError, ActionResult, ActionType, PositionSelector, ProductAction,
    SettlementReason, SettlementType,
};
use crate::ledger::domain::{Product, ProductType};
use crate::ledger::triggers::{FixingType, TriggerInfo, TriggerType};

/// Quant Library trait for product-level calculations.
///
/// This is the interface for the external Quant library that performs
/// actual product calculations. In production, this would call the real
/// quant library. For testing, use QuantLibStub.
pub trait QuantLibTrait: Send + Sync {
    /// Calculate the action for a product given trigger info.
    ///
    /// This is the core interface for Call 1 - given a trigger and product,
    /// determine what action (if any) should be taken.
    fn get_action(
        &self,
        product: &Product,
        trigger_data: &TriggerInfo,
    ) -> ActionResult<ProductAction>;
}

/// Stub implementation of QuantLib for testing.
///
/// Returns realistic ProductActions for thin slice products:
/// - KnockOutWarrant
/// - MiniCertificate
///
/// Supports all 8 trigger types needed for testing.
#[derive(Debug, Default)]
pub struct QuantLibStub {
    /// Optional override for settlement amounts
    settlement_amount_override: Option<Decimal>,
}

impl QuantLibStub {
    /// Create a new QuantLibStub.
    pub fn new() -> Self {
        QuantLibStub {
            settlement_amount_override: None,
        }
    }

    /// Create a stub with a specific settlement amount override.
    pub fn with_settlement_amount(mut self, amount: Decimal) -> Self {
        self.settlement_amount_override = Some(amount);
        self
    }

    /// Generate a unique action ID.
    fn generate_action_id(&self, prefix: &str) -> String {
        format!("{}-{}", prefix, Uuid::new_v4())
    }

    /// Process a continuous barrier trigger.
    fn process_continuous_barrier(
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

        // Calculate settlement amount based on product type
        let settlement_amount = self.settlement_amount_override.unwrap_or_else(|| {
            match product.product_type_enum() {
                ProductType::MiniCertificate => {
                    // Mini certificate: residual value = max(0, breach - barrier)
                    if breach_price > barrier_level {
                        breach_price - barrier_level
                    } else {
                        Decimal::ZERO
                    }
                }
                _ => breach_price, // Default: breach price is settlement
            }
        });

        Ok(ProductAction::new(
            self.generate_action_id("BARRIER"),
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
        .with_target_selector(PositionSelector::SameAsSource))
    }

    /// Process a discrete barrier trigger.
    fn process_discrete_barrier(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
    ) -> ActionResult<ProductAction> {
        // Same logic as continuous for the stub
        self.process_continuous_barrier(trigger_info, product)
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

        Ok(ProductAction::new(
            self.generate_action_id("FIXING"),
            ActionType::FixingObserved,
            product.id(),
        )
        .with_details(ActionDetails::Fixing {
            observed_price,
            fixing_type: fixing_type.as_str().to_string(),
            underlying: product.details().underlying.clone(),
        })
        .with_target_selector(PositionSelector::SameAsSource))
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
        let settlement_amount = self.settlement_amount_override.unwrap_or(settlement_price);

        Ok(ProductAction::new(
            self.generate_action_id("EXPIRY"),
            ActionType::Settlement,
            product.id(),
        )
        .with_amount(settlement_amount)
        .with_settlement_date(settlement_date)
        .with_details(ActionDetails::Settlement {
            settlement_type: SettlementType::Cash,
            reason: SettlementReason::Maturity,
        })
        .with_target_selector(PositionSelector::SameAsSource))
    }

    /// Process a coupon payment trigger.
    fn process_coupon(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
    ) -> ActionResult<ProductAction> {
        let settlement_date = trigger_info.trigger_time.date_naive();
        let coupon_amount = self.settlement_amount_override.unwrap_or(Decimal::new(500, 2)); // 5.00 default

        Ok(ProductAction::new(
            self.generate_action_id("COUPON"),
            ActionType::CouponPayment,
            product.id(),
        )
        .with_amount(coupon_amount)
        .with_settlement_date(settlement_date)
        .with_details(ActionDetails::Coupon {
            coupon_rate: Decimal::new(500, 4), // 5%
            period_start: settlement_date - chrono::Duration::days(180),
            period_end: settlement_date,
        })
        .with_target_selector(PositionSelector::SameAsSource))
    }

    /// Process an American exercise trigger.
    fn process_exercise(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
    ) -> ActionResult<ProductAction> {
        let spot_price = trigger_info.observed_price.ok_or_else(|| {
            ActionError::MissingData("observed_price required for exercise".to_string())
        })?;

        let settlement_date = trigger_info.trigger_time.date_naive();
        let strike_price = Decimal::new(10000, 2); // 100.00 default strike
        let intrinsic_value = (spot_price - strike_price).max(Decimal::ZERO);
        let settlement_amount = self.settlement_amount_override.unwrap_or(intrinsic_value);

        Ok(ProductAction::new(
            self.generate_action_id("EXERCISE"),
            ActionType::EarlyExercise,
            product.id(),
        )
        .with_amount(settlement_amount)
        .with_settlement_date(settlement_date)
        .with_details(ActionDetails::Exercise {
            strike_price,
            spot_price,
            is_call: true,
        })
        .with_target_selector(PositionSelector::SameAsSource))
    }

    /// Process a dividend trigger.
    fn process_dividend(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
    ) -> ActionResult<ProductAction> {
        let settlement_date = trigger_info.trigger_time.date_naive();
        let dividend_per_share = Decimal::new(50, 2); // 0.50 default
        let adjustment_factor = Decimal::ONE - (dividend_per_share / Decimal::new(10000, 2));

        Ok(ProductAction::new(
            self.generate_action_id("DIVIDEND"),
            ActionType::DividendAdjustment,
            product.id(),
        )
        .with_details(ActionDetails::Dividend {
            dividend_per_share,
            adjustment_factor,
            ex_date: settlement_date,
        })
        .with_target_selector(PositionSelector::SameAsSource))
    }

    /// Process a stock split trigger.
    fn process_stock_split(
        &self,
        trigger_info: &TriggerInfo,
        product: &Product,
    ) -> ActionResult<ProductAction> {
        let (new_shares, old_shares) = match &trigger_info.corporate_action_details {
            Some(details) => match &details.action_type {
                crate::ledger::triggers::CorporateActionType::StockSplit { new_shares, old_shares } => {
                    (*new_shares, *old_shares)
                }
                crate::ledger::triggers::CorporateActionType::ReverseSplit { old_shares, new_shares } => {
                    (*new_shares, *old_shares)
                }
                _ => (4, 1), // Default 4:1 split
            },
            None => (4, 1),
        };

        let adjustment_factor = Decimal::new(old_shares as i64, 0) / Decimal::new(new_shares as i64, 0);

        Ok(ProductAction::new(
            self.generate_action_id("SPLIT"),
            ActionType::StockSplitAdjustment,
            product.id(),
        )
        .with_details(ActionDetails::StockSplit {
            new_shares,
            old_shares,
            adjustment_factor,
        })
        .with_target_selector(PositionSelector::SameAsSource))
    }
}

impl QuantLibTrait for QuantLibStub {
    fn get_action(
        &self,
        product: &Product,
        trigger_data: &TriggerInfo,
    ) -> ActionResult<ProductAction> {
        match &trigger_data.trigger_type {
            TriggerType::ContinuousBarrier => self.process_continuous_barrier(trigger_data, product),
            TriggerType::Barrier => self.process_discrete_barrier(trigger_data, product),
            TriggerType::Fixing(fixing_type) => {
                self.process_fixing(trigger_data, product, *fixing_type)
            }
            TriggerType::Expiry => self.process_expiry(trigger_data, product),
            TriggerType::CouponPayment => self.process_coupon(trigger_data, product),
            TriggerType::AmericanExercise => self.process_exercise(trigger_data, product),
            TriggerType::Dividend => self.process_dividend(trigger_data, product),
            TriggerType::StockSplit => self.process_stock_split(trigger_data, product),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::ProductDetails;
    use crate::ledger::triggers::BarrierDirection;
    use chrono::{TimeZone, Utc};

    fn create_ko_product(id: &str) -> Product {
        Product::KnockOutWarrant(ProductDetails::new(id, "Test KO Warrant", "AAPL", "USD"))
    }

    fn create_mini_product(id: &str) -> Product {
        Product::MiniCertificate(ProductDetails::new(id, "Test Mini Cert", "DAX", "EUR"))
    }

    #[test]
    fn test_quant_lib_stub_barrier_breach() {
        let quant_lib = QuantLibStub::new();
        let ko_product = create_ko_product("KO-001");

        let barrier_trigger = TriggerInfo::new_barrier(
            true,
            Utc.with_ymd_and_hms(2024, 12, 15, 10, 30, 0).unwrap(),
            Decimal::new(8950, 2),
            Decimal::new(9000, 2),
            BarrierDirection::Down,
        );

        let action = quant_lib.get_action(&ko_product, &barrier_trigger).expect("Should succeed");
        assert_eq!(action.action_type, ActionType::BarrierBreach);
        assert_eq!(action.product_id, ko_product.id());
        assert_eq!(action.amount_per_unit, Some(Decimal::new(8950, 2)));
    }

    #[test]
    fn test_quant_lib_stub_fixing() {
        let quant_lib = QuantLibStub::new();
        let ko_product = create_ko_product("KO-001");

        let fixing_trigger = TriggerInfo::new_fixing(
            FixingType::Initial,
            Utc.with_ymd_and_hms(2024, 12, 15, 9, 0, 0).unwrap(),
            Decimal::new(15000, 2),
        );

        let action = quant_lib.get_action(&ko_product, &fixing_trigger).expect("Should succeed");
        assert_eq!(action.action_type, ActionType::FixingObserved);
    }

    #[test]
    fn test_quant_lib_stub_mini_certificate_barrier() {
        let quant_lib = QuantLibStub::new();
        let mini_product = create_mini_product("MINI-001");

        let barrier_trigger = TriggerInfo::new_barrier(
            true,
            Utc.with_ymd_and_hms(2024, 12, 15, 10, 30, 0).unwrap(),
            Decimal::new(10500, 2),  // breach at 105
            Decimal::new(10000, 2),  // barrier at 100
            BarrierDirection::Down,
        );

        let action = quant_lib.get_action(&mini_product, &barrier_trigger).expect("Should succeed");
        assert_eq!(action.action_type, ActionType::BarrierBreach);
        // Mini: residual = breach - barrier = 105 - 100 = 5.00
        assert_eq!(action.amount_per_unit, Some(Decimal::new(500, 2)));
    }
}
