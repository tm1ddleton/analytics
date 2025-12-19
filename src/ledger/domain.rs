//! Shared domain model for structured products and assets.
//!
//! This module defines the core type system for the Quantitative Event Engine:
//! - Product: 14 variants representing structured products
//! - Asset: 10 variants representing tradeable assets/hedges
//! - Instrument: Union type enabling unified position handling

use prost::Message;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a Product.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductId(String);

impl ProductId {
    pub fn new(id: impl Into<String>) -> Self {
        ProductId(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProductId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for an Asset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(String);

impl AssetId {
    pub fn new(id: impl Into<String>) -> Self {
        AssetId(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for an Instrument (either Product or Asset).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstrumentId {
    Product(ProductId),
    Asset(AssetId),
}

impl fmt::Display for InstrumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstrumentId::Product(id) => write!(f, "product:{}", id),
            InstrumentId::Asset(id) => write!(f, "asset:{}", id),
        }
    }
}

/// Product type discriminator for serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum ProductType {
    BarrierReverseConvertible = 1,
    BarrierReverseConvertiblePro = 2,
    BonusCertificate = 3,
    CappedBonusCertificate = 4,
    CappedBonusProCertificate = 5,
    CappedWarrant = 6,
    DiscountCertificate = 7,
    KnockOutWarrant = 8,
    ReverseCappedBonusCertificate = 9,
    ReverseConvertible = 10,
    MiniCertificate = 11,
    OpenEndTurbo = 12,
    FactorCertificate = 13,
    VanillaOption = 14,
}

impl ProductType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProductType::BarrierReverseConvertible => "BarrierReverseConvertible",
            ProductType::BarrierReverseConvertiblePro => "BarrierReverseConvertiblePro",
            ProductType::BonusCertificate => "BonusCertificate",
            ProductType::CappedBonusCertificate => "CappedBonusCertificate",
            ProductType::CappedBonusProCertificate => "CappedBonusProCertificate",
            ProductType::CappedWarrant => "CappedWarrant",
            ProductType::DiscountCertificate => "DiscountCertificate",
            ProductType::KnockOutWarrant => "KnockOutWarrant",
            ProductType::ReverseCappedBonusCertificate => "ReverseCappedBonusCertificate",
            ProductType::ReverseConvertible => "ReverseConvertible",
            ProductType::MiniCertificate => "MiniCertificate",
            ProductType::OpenEndTurbo => "OpenEndTurbo",
            ProductType::FactorCertificate => "FactorCertificate",
            ProductType::VanillaOption => "VanillaOption",
        }
    }

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(ProductType::BarrierReverseConvertible),
            2 => Some(ProductType::BarrierReverseConvertiblePro),
            3 => Some(ProductType::BonusCertificate),
            4 => Some(ProductType::CappedBonusCertificate),
            5 => Some(ProductType::CappedBonusProCertificate),
            6 => Some(ProductType::CappedWarrant),
            7 => Some(ProductType::DiscountCertificate),
            8 => Some(ProductType::KnockOutWarrant),
            9 => Some(ProductType::ReverseCappedBonusCertificate),
            10 => Some(ProductType::ReverseConvertible),
            11 => Some(ProductType::MiniCertificate),
            12 => Some(ProductType::OpenEndTurbo),
            13 => Some(ProductType::FactorCertificate),
            14 => Some(ProductType::VanillaOption),
            _ => None,
        }
    }
}

/// Common details for a product (protobuf-serializable).
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Message)]
pub struct ProductDetails {
    /// Product identifier
    #[prost(string, tag = "1")]
    pub id: String,

    /// Product name/description
    #[prost(string, tag = "2")]
    pub name: String,

    /// Underlying asset identifier
    #[prost(string, tag = "3")]
    pub underlying: String,

    /// Currency code (e.g., "USD", "EUR")
    #[prost(string, tag = "4")]
    pub currency: String,
}

impl ProductDetails {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        underlying: impl Into<String>,
        currency: impl Into<String>,
    ) -> Self {
        ProductDetails {
            id: id.into(),
            name: name.into(),
            underlying: underlying.into(),
            currency: currency.into(),
        }
    }
}

/// Product wrapper for protobuf serialization.
/// Encodes product_type as an integer and details as a nested message.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Message)]
pub struct ProductProto {
    /// Product type discriminator
    #[prost(int32, tag = "1")]
    pub product_type: i32,

    /// Product details
    #[prost(message, optional, tag = "2")]
    pub details: Option<ProductDetails>,
}

/// Structured product types (14 variants).
///
/// These represent the structured products that can be traded and have lifecycle events.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Product {
    /// Barrier Reverse Convertible - pays coupon unless barrier breached
    BarrierReverseConvertible(ProductDetails),

    /// Barrier Reverse Convertible Pro - enhanced version with additional features
    BarrierReverseConvertiblePro(ProductDetails),

    /// Bonus Certificate - provides bonus return if barrier not breached
    BonusCertificate(ProductDetails),

    /// Capped Bonus Certificate - bonus with upside cap
    CappedBonusCertificate(ProductDetails),

    /// Capped Bonus Pro Certificate - enhanced capped bonus
    CappedBonusProCertificate(ProductDetails),

    /// Capped Warrant - warrant with capped upside
    CappedWarrant(ProductDetails),

    /// Discount Certificate - buy underlying at discount, capped upside
    DiscountCertificate(ProductDetails),

    /// Knock-Out Warrant - leverage product with knockout barrier (thin slice)
    KnockOutWarrant(ProductDetails),

    /// Reverse Capped Bonus Certificate - inverse bonus structure
    ReverseCappedBonusCertificate(ProductDetails),

    /// Reverse Convertible - simpler version of BRC without barrier
    ReverseConvertible(ProductDetails),

    /// Mini Certificate - leverage product with continuous barrier (thin slice)
    MiniCertificate(ProductDetails),

    /// Open End Turbo - perpetual leverage product
    OpenEndTurbo(ProductDetails),

    /// Factor Certificate - daily leverage reset product
    FactorCertificate(ProductDetails),

    /// Vanilla Option - standard option (call/put)
    VanillaOption(ProductDetails),
}

impl Default for Product {
    fn default() -> Self {
        Product::KnockOutWarrant(ProductDetails::default())
    }
}

impl Product {
    /// Returns the product type enum value.
    pub fn product_type_enum(&self) -> ProductType {
        match self {
            Product::BarrierReverseConvertible(_) => ProductType::BarrierReverseConvertible,
            Product::BarrierReverseConvertiblePro(_) => ProductType::BarrierReverseConvertiblePro,
            Product::BonusCertificate(_) => ProductType::BonusCertificate,
            Product::CappedBonusCertificate(_) => ProductType::CappedBonusCertificate,
            Product::CappedBonusProCertificate(_) => ProductType::CappedBonusProCertificate,
            Product::CappedWarrant(_) => ProductType::CappedWarrant,
            Product::DiscountCertificate(_) => ProductType::DiscountCertificate,
            Product::KnockOutWarrant(_) => ProductType::KnockOutWarrant,
            Product::ReverseCappedBonusCertificate(_) => ProductType::ReverseCappedBonusCertificate,
            Product::ReverseConvertible(_) => ProductType::ReverseConvertible,
            Product::MiniCertificate(_) => ProductType::MiniCertificate,
            Product::OpenEndTurbo(_) => ProductType::OpenEndTurbo,
            Product::FactorCertificate(_) => ProductType::FactorCertificate,
            Product::VanillaOption(_) => ProductType::VanillaOption,
        }
    }

    /// Returns the product type as a string for discrimination.
    pub fn product_type(&self) -> &'static str {
        self.product_type_enum().as_str()
    }

    /// Returns the product details.
    pub fn details(&self) -> &ProductDetails {
        match self {
            Product::BarrierReverseConvertible(d) => d,
            Product::BarrierReverseConvertiblePro(d) => d,
            Product::BonusCertificate(d) => d,
            Product::CappedBonusCertificate(d) => d,
            Product::CappedBonusProCertificate(d) => d,
            Product::CappedWarrant(d) => d,
            Product::DiscountCertificate(d) => d,
            Product::KnockOutWarrant(d) => d,
            Product::ReverseCappedBonusCertificate(d) => d,
            Product::ReverseConvertible(d) => d,
            Product::MiniCertificate(d) => d,
            Product::OpenEndTurbo(d) => d,
            Product::FactorCertificate(d) => d,
            Product::VanillaOption(d) => d,
        }
    }

    /// Returns the ProductId for this product.
    pub fn id(&self) -> ProductId {
        ProductId::new(&self.details().id)
    }

    /// Convert to protobuf representation.
    pub fn to_proto(&self) -> ProductProto {
        ProductProto {
            product_type: self.product_type_enum() as i32,
            details: Some(self.details().clone()),
        }
    }

    /// Convert from protobuf representation.
    pub fn from_proto(proto: ProductProto) -> Option<Self> {
        let product_type = ProductType::from_i32(proto.product_type)?;
        let details = proto.details?;

        Some(match product_type {
            ProductType::BarrierReverseConvertible => Product::BarrierReverseConvertible(details),
            ProductType::BarrierReverseConvertiblePro => {
                Product::BarrierReverseConvertiblePro(details)
            }
            ProductType::BonusCertificate => Product::BonusCertificate(details),
            ProductType::CappedBonusCertificate => Product::CappedBonusCertificate(details),
            ProductType::CappedBonusProCertificate => Product::CappedBonusProCertificate(details),
            ProductType::CappedWarrant => Product::CappedWarrant(details),
            ProductType::DiscountCertificate => Product::DiscountCertificate(details),
            ProductType::KnockOutWarrant => Product::KnockOutWarrant(details),
            ProductType::ReverseCappedBonusCertificate => {
                Product::ReverseCappedBonusCertificate(details)
            }
            ProductType::ReverseConvertible => Product::ReverseConvertible(details),
            ProductType::MiniCertificate => Product::MiniCertificate(details),
            ProductType::OpenEndTurbo => Product::OpenEndTurbo(details),
            ProductType::FactorCertificate => Product::FactorCertificate(details),
            ProductType::VanillaOption => Product::VanillaOption(details),
        })
    }

    /// Encode to protobuf bytes.
    pub fn encode_proto(&self) -> Vec<u8> {
        let proto = self.to_proto();
        let mut buf = Vec::new();
        proto.encode(&mut buf).expect("encoding should not fail");
        buf
    }

    /// Decode from protobuf bytes.
    pub fn decode_proto(buf: &[u8]) -> Option<Self> {
        let proto = ProductProto::decode(buf).ok()?;
        Self::from_proto(proto)
    }
}

/// Asset type discriminator for serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum AssetType {
    Equity = 1,
    Bond = 2,
    Future = 3,
    BarrierOption = 4,
    InterestRateFuture = 5,
    InterestRateOption = 6,
    Cash = 7,
    FxOption = 8,
    FxSwap = 9,
    StockBorrowLoan = 10,
}

impl AssetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetType::Equity => "Equity",
            AssetType::Bond => "Bond",
            AssetType::Future => "Future",
            AssetType::BarrierOption => "BarrierOption",
            AssetType::InterestRateFuture => "InterestRateFuture",
            AssetType::InterestRateOption => "InterestRateOption",
            AssetType::Cash => "Cash",
            AssetType::FxOption => "FxOption",
            AssetType::FxSwap => "FxSwap",
            AssetType::StockBorrowLoan => "StockBorrowLoan",
        }
    }

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(AssetType::Equity),
            2 => Some(AssetType::Bond),
            3 => Some(AssetType::Future),
            4 => Some(AssetType::BarrierOption),
            5 => Some(AssetType::InterestRateFuture),
            6 => Some(AssetType::InterestRateOption),
            7 => Some(AssetType::Cash),
            8 => Some(AssetType::FxOption),
            9 => Some(AssetType::FxSwap),
            10 => Some(AssetType::StockBorrowLoan),
            _ => None,
        }
    }
}

/// Common details for an asset (protobuf-serializable).
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Message)]
pub struct AssetDetails {
    /// Asset identifier
    #[prost(string, tag = "1")]
    pub id: String,

    /// Asset name/description
    #[prost(string, tag = "2")]
    pub name: String,

    /// Currency code (e.g., "USD", "EUR")
    #[prost(string, tag = "3")]
    pub currency: String,
}

impl AssetDetails {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        currency: impl Into<String>,
    ) -> Self {
        AssetDetails {
            id: id.into(),
            name: name.into(),
            currency: currency.into(),
        }
    }
}

/// Asset wrapper for protobuf serialization.
/// Encodes asset_type as an integer and details as a nested message.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Message)]
pub struct AssetProto {
    /// Asset type discriminator
    #[prost(int32, tag = "1")]
    pub asset_type: i32,

    /// Asset details
    #[prost(message, optional, tag = "2")]
    pub details: Option<AssetDetails>,
}

/// Asset types (10 variants).
///
/// These represent tradeable assets that can be used as hedges or held directly.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Asset {
    /// Equity - common stock
    Equity(AssetDetails),

    /// Bond - fixed income instrument
    Bond(AssetDetails),

    /// Future - exchange-traded futures contract
    Future(AssetDetails),

    /// Barrier Option - OTC option with barrier feature
    BarrierOption(AssetDetails),

    /// Interest Rate Future - futures on interest rates
    InterestRateFuture(AssetDetails),

    /// Interest Rate Option - options on interest rates
    InterestRateOption(AssetDetails),

    /// Cash - currency holdings
    Cash(AssetDetails),

    /// FX Option - foreign exchange option
    FxOption(AssetDetails),

    /// FX Swap - foreign exchange swap
    FxSwap(AssetDetails),

    /// Stock Borrow Loan - securities lending arrangement
    StockBorrowLoan(AssetDetails),
}

impl Default for Asset {
    fn default() -> Self {
        Asset::Cash(AssetDetails::default())
    }
}

impl Asset {
    /// Returns the asset type enum value.
    pub fn asset_type_enum(&self) -> AssetType {
        match self {
            Asset::Equity(_) => AssetType::Equity,
            Asset::Bond(_) => AssetType::Bond,
            Asset::Future(_) => AssetType::Future,
            Asset::BarrierOption(_) => AssetType::BarrierOption,
            Asset::InterestRateFuture(_) => AssetType::InterestRateFuture,
            Asset::InterestRateOption(_) => AssetType::InterestRateOption,
            Asset::Cash(_) => AssetType::Cash,
            Asset::FxOption(_) => AssetType::FxOption,
            Asset::FxSwap(_) => AssetType::FxSwap,
            Asset::StockBorrowLoan(_) => AssetType::StockBorrowLoan,
        }
    }

    /// Returns the asset type as a string for discrimination.
    pub fn asset_type(&self) -> &'static str {
        self.asset_type_enum().as_str()
    }

    /// Returns the asset details.
    pub fn details(&self) -> &AssetDetails {
        match self {
            Asset::Equity(d) => d,
            Asset::Bond(d) => d,
            Asset::Future(d) => d,
            Asset::BarrierOption(d) => d,
            Asset::InterestRateFuture(d) => d,
            Asset::InterestRateOption(d) => d,
            Asset::Cash(d) => d,
            Asset::FxOption(d) => d,
            Asset::FxSwap(d) => d,
            Asset::StockBorrowLoan(d) => d,
        }
    }

    /// Returns the AssetId for this asset.
    pub fn id(&self) -> AssetId {
        AssetId::new(&self.details().id)
    }

    /// Convert to protobuf representation.
    pub fn to_proto(&self) -> AssetProto {
        AssetProto {
            asset_type: self.asset_type_enum() as i32,
            details: Some(self.details().clone()),
        }
    }

    /// Convert from protobuf representation.
    pub fn from_proto(proto: AssetProto) -> Option<Self> {
        let asset_type = AssetType::from_i32(proto.asset_type)?;
        let details = proto.details?;

        Some(match asset_type {
            AssetType::Equity => Asset::Equity(details),
            AssetType::Bond => Asset::Bond(details),
            AssetType::Future => Asset::Future(details),
            AssetType::BarrierOption => Asset::BarrierOption(details),
            AssetType::InterestRateFuture => Asset::InterestRateFuture(details),
            AssetType::InterestRateOption => Asset::InterestRateOption(details),
            AssetType::Cash => Asset::Cash(details),
            AssetType::FxOption => Asset::FxOption(details),
            AssetType::FxSwap => Asset::FxSwap(details),
            AssetType::StockBorrowLoan => Asset::StockBorrowLoan(details),
        })
    }

    /// Encode to protobuf bytes.
    pub fn encode_proto(&self) -> Vec<u8> {
        let proto = self.to_proto();
        let mut buf = Vec::new();
        proto.encode(&mut buf).expect("encoding should not fail");
        buf
    }

    /// Decode from protobuf bytes.
    pub fn decode_proto(buf: &[u8]) -> Option<Self> {
        let proto = AssetProto::decode(buf).ok()?;
        Self::from_proto(proto)
    }
}

/// Instrument - union type for unified position handling.
///
/// Enables the Position Ledger to hold both Products and Assets,
/// supporting scenarios where products can be traded as assets.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Instrument {
    /// A structured product
    Product(Product),

    /// A tradeable asset (including products traded as assets)
    Asset(Asset),
}

impl Instrument {
    /// Returns the instrument type as a string.
    pub fn instrument_type(&self) -> &'static str {
        match self {
            Instrument::Product(_) => "Product",
            Instrument::Asset(_) => "Asset",
        }
    }

    /// Returns the specific type within the instrument category.
    pub fn specific_type(&self) -> &'static str {
        match self {
            Instrument::Product(p) => p.product_type(),
            Instrument::Asset(a) => a.asset_type(),
        }
    }

    /// Returns the InstrumentId for this instrument.
    pub fn id(&self) -> InstrumentId {
        match self {
            Instrument::Product(p) => InstrumentId::Product(p.id()),
            Instrument::Asset(a) => InstrumentId::Asset(a.id()),
        }
    }

    /// Returns the currency of the instrument.
    pub fn currency(&self) -> &str {
        match self {
            Instrument::Product(p) => &p.details().currency,
            Instrument::Asset(a) => &a.details().currency,
        }
    }
}

/// Convert a Product to an Instrument.
impl From<Product> for Instrument {
    fn from(product: Product) -> Self {
        Instrument::Product(product)
    }
}

/// Convert an Asset to an Instrument.
impl From<Asset> for Instrument {
    fn from(asset: Asset) -> Self {
        Instrument::Asset(asset)
    }
}

/// Enables treating a Product as an Asset (products-as-assets scenario).
///
/// This creates an Asset::BarrierOption representation of a product,
/// allowing products to be held and traded like assets in the Position Ledger.
impl Product {
    pub fn as_asset(&self) -> Asset {
        let details = self.details();
        Asset::BarrierOption(AssetDetails::new(
            format!("product_as_asset:{}", details.id),
            format!("{} (as asset)", details.name),
            &details.currency,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1: Product enum discrimination and serialization
    #[test]
    fn test_product_discrimination_and_serialization() {
        let details = ProductDetails::new("KO-001", "Test Knock-Out", "AAPL", "USD");
        let product = Product::KnockOutWarrant(details);

        assert_eq!(product.product_type(), "KnockOutWarrant");

        // Test JSON serialization roundtrip
        let json = serde_json::to_string(&product).unwrap();
        let deserialized: Product = serde_json::from_str(&json).unwrap();
        assert_eq!(product, deserialized);
        assert_eq!(deserialized.product_type(), "KnockOutWarrant");

        // Test all 14 product types are discriminable
        let products = vec![
            Product::BarrierReverseConvertible(ProductDetails::default()),
            Product::BarrierReverseConvertiblePro(ProductDetails::default()),
            Product::BonusCertificate(ProductDetails::default()),
            Product::CappedBonusCertificate(ProductDetails::default()),
            Product::CappedBonusProCertificate(ProductDetails::default()),
            Product::CappedWarrant(ProductDetails::default()),
            Product::DiscountCertificate(ProductDetails::default()),
            Product::KnockOutWarrant(ProductDetails::default()),
            Product::ReverseCappedBonusCertificate(ProductDetails::default()),
            Product::ReverseConvertible(ProductDetails::default()),
            Product::MiniCertificate(ProductDetails::default()),
            Product::OpenEndTurbo(ProductDetails::default()),
            Product::FactorCertificate(ProductDetails::default()),
            Product::VanillaOption(ProductDetails::default()),
        ];

        let types: Vec<&str> = products.iter().map(|p| p.product_type()).collect();
        assert_eq!(types.len(), 14);
        // Ensure all types are unique
        let mut unique_types = types.clone();
        unique_types.sort();
        unique_types.dedup();
        assert_eq!(unique_types.len(), 14, "All 14 product types must be unique");
    }

    // Test 2: Asset enum discrimination and serialization
    #[test]
    fn test_asset_discrimination_and_serialization() {
        let details = AssetDetails::new("EQ-AAPL", "Apple Inc.", "USD");
        let asset = Asset::Equity(details);

        assert_eq!(asset.asset_type(), "Equity");

        // Test JSON serialization roundtrip
        let json = serde_json::to_string(&asset).unwrap();
        let deserialized: Asset = serde_json::from_str(&json).unwrap();
        assert_eq!(asset, deserialized);
        assert_eq!(deserialized.asset_type(), "Equity");

        // Test all 10 asset types are discriminable
        let assets = vec![
            Asset::Equity(AssetDetails::default()),
            Asset::Bond(AssetDetails::default()),
            Asset::Future(AssetDetails::default()),
            Asset::BarrierOption(AssetDetails::default()),
            Asset::InterestRateFuture(AssetDetails::default()),
            Asset::InterestRateOption(AssetDetails::default()),
            Asset::Cash(AssetDetails::default()),
            Asset::FxOption(AssetDetails::default()),
            Asset::FxSwap(AssetDetails::default()),
            Asset::StockBorrowLoan(AssetDetails::default()),
        ];

        let types: Vec<&str> = assets.iter().map(|a| a.asset_type()).collect();
        assert_eq!(types.len(), 10);
        // Ensure all types are unique
        let mut unique_types = types.clone();
        unique_types.sort();
        unique_types.dedup();
        assert_eq!(unique_types.len(), 10, "All 10 asset types must be unique");
    }

    // Test 3: ProductId and AssetId key generation and equality
    #[test]
    fn test_id_key_generation_and_equality() {
        // ProductId tests
        let prod_id1 = ProductId::new("PROD-001");
        let prod_id2 = ProductId::new("PROD-001");
        let prod_id3 = ProductId::new("PROD-002");

        assert_eq!(prod_id1, prod_id2);
        assert_ne!(prod_id1, prod_id3);
        assert_eq!(prod_id1.as_str(), "PROD-001");
        assert_eq!(format!("{}", prod_id1), "PROD-001");

        // AssetId tests
        let asset_id1 = AssetId::new("ASSET-001");
        let asset_id2 = AssetId::new("ASSET-001");
        let asset_id3 = AssetId::new("ASSET-002");

        assert_eq!(asset_id1, asset_id2);
        assert_ne!(asset_id1, asset_id3);
        assert_eq!(asset_id1.as_str(), "ASSET-001");
        assert_eq!(format!("{}", asset_id1), "ASSET-001");

        // InstrumentId tests
        let inst_prod = InstrumentId::Product(prod_id1.clone());
        let inst_asset = InstrumentId::Asset(asset_id1.clone());

        assert_ne!(inst_prod, inst_asset);
        assert_eq!(format!("{}", inst_prod), "product:PROD-001");
        assert_eq!(format!("{}", inst_asset), "asset:ASSET-001");

        // Test ID extraction from Product and Asset
        let product = Product::KnockOutWarrant(ProductDetails::new("KO-123", "Test", "AAPL", "USD"));
        assert_eq!(product.id(), ProductId::new("KO-123"));

        let asset = Asset::Equity(AssetDetails::new("EQ-456", "Test Stock", "EUR"));
        assert_eq!(asset.id(), AssetId::new("EQ-456"));
    }

    // Test 4: Type conversions between Product and Asset
    #[test]
    fn test_product_asset_conversions() {
        let product_details = ProductDetails::new("BRC-001", "Barrier RC", "AAPL", "USD");
        let product = Product::BarrierReverseConvertible(product_details);

        // Test Product -> Instrument conversion
        let inst_from_product: Instrument = product.clone().into();
        assert_eq!(inst_from_product.instrument_type(), "Product");
        assert_eq!(inst_from_product.specific_type(), "BarrierReverseConvertible");

        // Test Asset -> Instrument conversion
        let asset = Asset::Equity(AssetDetails::new("EQ-001", "Apple Inc.", "USD"));
        let inst_from_asset: Instrument = asset.clone().into();
        assert_eq!(inst_from_asset.instrument_type(), "Asset");
        assert_eq!(inst_from_asset.specific_type(), "Equity");

        // Test products-as-assets scenario
        let product_as_asset = product.as_asset();
        assert_eq!(product_as_asset.asset_type(), "BarrierOption");
        assert!(product_as_asset.details().id.starts_with("product_as_asset:"));
        assert!(product_as_asset.details().name.contains("(as asset)"));

        // Verify InstrumentId preserves distinction
        let prod_inst_id = inst_from_product.id();
        let asset_inst_id = inst_from_asset.id();
        assert!(matches!(prod_inst_id, InstrumentId::Product(_)));
        assert!(matches!(asset_inst_id, InstrumentId::Asset(_)));
    }

    // Test 5: Protobuf serialization for Product
    #[test]
    fn test_product_protobuf_serialization() {
        let details = ProductDetails::new("MINI-001", "Mini Certificate", "DAX", "EUR");
        let product = Product::MiniCertificate(details);

        // Encode to protobuf bytes
        let buf = product.encode_proto();

        // Decode from protobuf bytes
        let decoded = Product::decode_proto(&buf).unwrap();
        assert_eq!(product, decoded);
        assert_eq!(decoded.product_type(), "MiniCertificate");
        assert_eq!(decoded.details().underlying, "DAX");
    }

    // Test 6: Protobuf serialization for Asset
    #[test]
    fn test_asset_protobuf_serialization() {
        let details = AssetDetails::new("FUT-ESZ24", "E-mini S&P Dec 2024", "USD");
        let asset = Asset::Future(details);

        // Encode to protobuf bytes
        let buf = asset.encode_proto();

        // Decode from protobuf bytes
        let decoded = Asset::decode_proto(&buf).unwrap();
        assert_eq!(asset, decoded);
        assert_eq!(decoded.asset_type(), "Future");
        assert_eq!(decoded.details().name, "E-mini S&P Dec 2024");
    }
}
