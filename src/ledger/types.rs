//! Ledger Abstraction Layer for the Quantitative Event Engine.
//!
//! This module defines the core ledger types and traits:
//! - Position: Represents a holding in an instrument with tags
//! - Move: Represents a ledger entry with audit trail
//! - LedgerReader/LedgerWriter/Ledger traits: Storage-agnostic interface

use chrono::NaiveDate;
use prost::Message;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

use super::domain::Instrument;

/// Unique identifier for a Position.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionId(String);

impl PositionId {
    pub fn new(id: impl Into<String>) -> Self {
        PositionId(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PositionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a Wallet (grouping of positions).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WalletId(String);

impl WalletId {
    pub fn new(id: impl Into<String>) -> Self {
        WalletId(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WalletId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a Move (ledger entry).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MoveId(String);

impl MoveId {
    pub fn new(id: impl Into<String>) -> Self {
        MoveId(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MoveId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tag key for multi-dimensional classification (e.g., "strategy", "desk", "trader").
pub type TagKey = String;

/// Tag value for classification (e.g., "yield_enhancement", "desk_a").
pub type TagValue = String;

/// Account identifier for ledger entries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Account(String);

impl Account {
    pub fn new(id: impl Into<String>) -> Self {
        Account(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Reference to the source ProductAction for audit trail.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductActionRef(String);

impl ProductActionRef {
    pub fn new(id: impl Into<String>) -> Self {
        ProductActionRef(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProductActionRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Reference to the MoveRule that generated the move for audit trail.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MoveRuleRef(String);

impl MoveRuleRef {
    pub fn new(id: impl Into<String>) -> Self {
        MoveRuleRef(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MoveRuleRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of ledger move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum MoveType {
    /// Standard transfer between accounts
    Transfer = 1,
    /// Opening position entry
    Open = 2,
    /// Closing position entry
    Close = 3,
    /// Adjustment (e.g., corporate action)
    Adjustment = 4,
    /// Settlement cash flow
    Settlement = 5,
    /// Coupon payment
    Coupon = 6,
    /// Fee or commission
    Fee = 7,
    /// Tax withholding
    Tax = 8,
}

impl MoveType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MoveType::Transfer => "Transfer",
            MoveType::Open => "Open",
            MoveType::Close => "Close",
            MoveType::Adjustment => "Adjustment",
            MoveType::Settlement => "Settlement",
            MoveType::Coupon => "Coupon",
            MoveType::Fee => "Fee",
            MoveType::Tax => "Tax",
        }
    }

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(MoveType::Transfer),
            2 => Some(MoveType::Open),
            3 => Some(MoveType::Close),
            4 => Some(MoveType::Adjustment),
            5 => Some(MoveType::Settlement),
            6 => Some(MoveType::Coupon),
            7 => Some(MoveType::Fee),
            8 => Some(MoveType::Tax),
            _ => None,
        }
    }
}

/// A position in the ledger - holding of an instrument with tags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// Unique position identifier
    pub id: PositionId,

    /// Wallet that contains this position
    pub wallet_id: WalletId,

    /// Instrument held (Product or Asset)
    pub instrument: Instrument,

    /// Current quantity held
    pub quantity: Decimal,

    /// Multi-dimensional tags for portfolio classification
    pub tags: HashMap<TagKey, TagValue>,
}

impl Position {
    /// Create a new position.
    pub fn new(
        id: impl Into<String>,
        wallet_id: impl Into<String>,
        instrument: Instrument,
        quantity: Decimal,
    ) -> Self {
        Position {
            id: PositionId::new(id),
            wallet_id: WalletId::new(wallet_id),
            instrument,
            quantity,
            tags: HashMap::new(),
        }
    }

    /// Add a tag to this position.
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Add multiple tags to this position.
    pub fn with_tags(mut self, tags: HashMap<TagKey, TagValue>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Check if position matches a tag criterion.
    pub fn has_tag(&self, key: &str, value: &str) -> bool {
        self.tags.get(key).map(|v| v == value).unwrap_or(false)
    }

    /// Check if position has a tag key (regardless of value).
    pub fn has_tag_key(&self, key: &str) -> bool {
        self.tags.contains_key(key)
    }
}

/// A ledger move - entry recording a transfer between accounts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Move {
    /// Unique move identifier
    pub id: MoveId,

    /// Position this move affects
    pub position_id: PositionId,

    /// Type of move
    pub move_type: MoveType,

    /// Account being debited
    pub debit_account: Account,

    /// Account being credited
    pub credit_account: Account,

    /// Amount of the move
    pub amount: Decimal,

    /// Date the move takes effect
    pub effective_date: NaiveDate,

    /// Reference to the ProductAction that caused this move (audit trail)
    pub source_action: ProductActionRef,

    /// Reference to the MoveRule that generated this move (audit trail)
    pub rule_applied: MoveRuleRef,

    /// Snapshot of context at time of move generation
    pub context_snapshot: HashMap<String, String>,

    /// Tags inherited from position for P&L attribution
    pub inherited_tags: HashMap<TagKey, TagValue>,
}

impl Move {
    /// Create a new move with required fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        position_id: PositionId,
        move_type: MoveType,
        debit_account: impl Into<String>,
        credit_account: impl Into<String>,
        amount: Decimal,
        effective_date: NaiveDate,
        source_action: impl Into<String>,
        rule_applied: impl Into<String>,
    ) -> Self {
        Move {
            id: MoveId::new(id),
            position_id,
            move_type,
            debit_account: Account::new(debit_account),
            credit_account: Account::new(credit_account),
            amount,
            effective_date,
            source_action: ProductActionRef::new(source_action),
            rule_applied: MoveRuleRef::new(rule_applied),
            context_snapshot: HashMap::new(),
            inherited_tags: HashMap::new(),
        }
    }

    /// Add context snapshot to this move.
    pub fn with_context(mut self, context: HashMap<String, String>) -> Self {
        self.context_snapshot = context;
        self
    }

    /// Add inherited tags to this move.
    pub fn with_inherited_tags(mut self, tags: HashMap<TagKey, TagValue>) -> Self {
        self.inherited_tags = tags;
        self
    }

    /// Inherit tags from a position.
    pub fn inherit_from_position(mut self, position: &Position) -> Self {
        self.inherited_tags = position.tags.clone();
        self
    }
}

/// Protobuf representation for Position.
#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct PositionProto {
    #[prost(string, tag = "1")]
    pub id: String,

    #[prost(string, tag = "2")]
    pub wallet_id: String,

    #[prost(string, tag = "3")]
    pub quantity: String,

    #[prost(map = "string, string", tag = "4")]
    pub tags: HashMap<String, String>,
}

/// Protobuf representation for Move.
#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct MoveProto {
    #[prost(string, tag = "1")]
    pub id: String,

    #[prost(string, tag = "2")]
    pub position_id: String,

    #[prost(int32, tag = "3")]
    pub move_type: i32,

    #[prost(string, tag = "4")]
    pub debit_account: String,

    #[prost(string, tag = "5")]
    pub credit_account: String,

    #[prost(string, tag = "6")]
    pub amount: String,

    #[prost(string, tag = "7")]
    pub effective_date: String,

    #[prost(string, tag = "8")]
    pub source_action: String,

    #[prost(string, tag = "9")]
    pub rule_applied: String,

    #[prost(map = "string, string", tag = "10")]
    pub context_snapshot: HashMap<String, String>,

    #[prost(map = "string, string", tag = "11")]
    pub inherited_tags: HashMap<String, String>,
}

impl Move {
    /// Convert to protobuf representation.
    pub fn to_proto(&self) -> MoveProto {
        MoveProto {
            id: self.id.as_str().to_string(),
            position_id: self.position_id.as_str().to_string(),
            move_type: self.move_type as i32,
            debit_account: self.debit_account.as_str().to_string(),
            credit_account: self.credit_account.as_str().to_string(),
            amount: self.amount.to_string(),
            effective_date: self.effective_date.format("%Y-%m-%d").to_string(),
            source_action: self.source_action.as_str().to_string(),
            rule_applied: self.rule_applied.as_str().to_string(),
            context_snapshot: self.context_snapshot.clone(),
            inherited_tags: self.inherited_tags.clone(),
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

/// Tag criterion for portfolio filtering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TagCriterion {
    /// Tag must equal specific value
    Equals(TagKey, TagValue),
    /// Tag must be one of the specified values
    In(TagKey, Vec<TagValue>),
    /// Tag key must exist (any value)
    Exists(TagKey),
}

impl TagCriterion {
    /// Check if a position matches this criterion.
    pub fn matches(&self, tags: &HashMap<TagKey, TagValue>) -> bool {
        match self {
            TagCriterion::Equals(key, value) => tags.get(key).map(|v| v == value).unwrap_or(false),
            TagCriterion::In(key, values) => {
                tags.get(key).map(|v| values.contains(v)).unwrap_or(false)
            }
            TagCriterion::Exists(key) => tags.contains_key(key),
        }
    }
}

/// Portfolio filter for querying positions and moves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortfolioFilter {
    /// Name of this portfolio view
    pub name: String,

    /// Criteria that must ALL match (AND logic)
    pub criteria: Vec<TagCriterion>,
}

impl PortfolioFilter {
    /// Create a new portfolio filter.
    pub fn new(name: impl Into<String>) -> Self {
        PortfolioFilter {
            name: name.into(),
            criteria: Vec::new(),
        }
    }

    /// Add a criterion to the filter.
    pub fn with_criterion(mut self, criterion: TagCriterion) -> Self {
        self.criteria.push(criterion);
        self
    }

    /// Check if a set of tags matches all criteria in this filter.
    pub fn matches(&self, tags: &HashMap<TagKey, TagValue>) -> bool {
        self.criteria.iter().all(|c| c.matches(tags))
    }
}

/// Errors that can occur during ledger operations.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum LedgerError {
    #[error("Position not found: {0}")]
    PositionNotFound(String),

    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    #[error("Duplicate position ID: {0}")]
    DuplicatePosition(String),

    #[error("Insufficient balance for position {position_id}: required {required}, available {available}")]
    InsufficientBalance {
        position_id: String,
        required: String,
        available: String,
    },

    #[error("Invalid move: {0}")]
    InvalidMove(String),

    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Result type for ledger operations.
pub type LedgerResult<T> = Result<T, LedgerError>;

/// Trait for reading from a ledger (storage-agnostic).
pub trait LedgerReader {
    /// Get a position by ID.
    fn get_position(&self, id: &PositionId) -> LedgerResult<Position>;

    /// Get all positions in a wallet.
    fn get_positions_by_wallet(&self, wallet_id: &WalletId) -> LedgerResult<Vec<Position>>;

    /// Get the current balance for a position (sum of all moves).
    fn get_balance(&self, position_id: &PositionId) -> LedgerResult<Decimal>;

    /// Get positions matching a portfolio filter.
    fn positions_by_portfolio(&self, filter: &PortfolioFilter) -> LedgerResult<Vec<Position>>;

    /// Get moves for positions matching a portfolio filter.
    fn moves_by_portfolio(&self, filter: &PortfolioFilter) -> LedgerResult<Vec<Move>>;
}

/// Trait for writing to a ledger (storage-agnostic).
pub trait LedgerWriter {
    /// Create a new position and return its ID.
    fn create_position(&mut self, position: Position) -> LedgerResult<PositionId>;

    /// Record a batch of moves to the ledger.
    fn record_moves(&mut self, moves: Vec<Move>) -> LedgerResult<()>;
}

/// Combined ledger trait for full read/write access.
///
/// Enables future persistent implementations (SQLite, PostgreSQL, etc.)
/// without changing consumers of the trait.
pub trait Ledger: LedgerReader + LedgerWriter {}

/// Blanket implementation: any type implementing both traits implements Ledger.
impl<T> Ledger for T where T: LedgerReader + LedgerWriter {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::{Asset, AssetDetails, Product, ProductDetails};

    // Test 1: Position struct construction with tags
    #[test]
    fn test_position_construction_with_tags() {
        let instrument = Instrument::Product(Product::KnockOutWarrant(ProductDetails::new(
            "KO-001",
            "Test Knock-Out",
            "AAPL",
            "USD",
        )));

        let position = Position::new("POS-001", "WALLET-001", instrument.clone(), Decimal::new(100, 0))
            .with_tag("strategy", "yield_enhancement")
            .with_tag("desk", "desk_a")
            .with_tag("trader", "john");

        assert_eq!(position.id.as_str(), "POS-001");
        assert_eq!(position.wallet_id.as_str(), "WALLET-001");
        assert_eq!(position.instrument, instrument);
        assert_eq!(position.quantity, Decimal::new(100, 0));
        assert_eq!(position.tags.len(), 3);
        assert!(position.has_tag("strategy", "yield_enhancement"));
        assert!(position.has_tag("desk", "desk_a"));
        assert!(position.has_tag_key("trader"));
        assert!(!position.has_tag("desk", "desk_b")); // Wrong value
        assert!(!position.has_tag_key("client")); // Missing key

        // Test with_tags method
        let mut more_tags = HashMap::new();
        more_tags.insert("client".to_string(), "client_x".to_string());
        more_tags.insert("book".to_string(), "trading_book".to_string());

        let position = position.with_tags(more_tags);
        assert_eq!(position.tags.len(), 5);
        assert!(position.has_tag("client", "client_x"));
    }

    // Test 2: Move struct construction with inherited tags
    #[test]
    fn test_move_construction_with_inherited_tags() {
        let effective_date = NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();

        let mov = Move::new(
            "MOV-001",
            PositionId::new("POS-001"),
            MoveType::Settlement,
            "CASH_ACCOUNT",
            "PRODUCT_ACCOUNT",
            Decimal::new(10000, 2), // 100.00
            effective_date,
            "ACTION-001",
            "RULE-DIRECT-BOOKING",
        );

        assert_eq!(mov.id.as_str(), "MOV-001");
        assert_eq!(mov.position_id.as_str(), "POS-001");
        assert_eq!(mov.move_type, MoveType::Settlement);
        assert_eq!(mov.debit_account.as_str(), "CASH_ACCOUNT");
        assert_eq!(mov.credit_account.as_str(), "PRODUCT_ACCOUNT");
        assert_eq!(mov.amount, Decimal::new(10000, 2));
        assert_eq!(mov.effective_date, effective_date);
        assert_eq!(mov.source_action.as_str(), "ACTION-001");
        assert_eq!(mov.rule_applied.as_str(), "RULE-DIRECT-BOOKING");
        assert!(mov.context_snapshot.is_empty());
        assert!(mov.inherited_tags.is_empty());

        // Test with context
        let mut context = HashMap::new();
        context.insert("legal_entity".to_string(), "ENTITY_A".to_string());
        context.insert("jurisdiction".to_string(), "US".to_string());

        let mov = mov.with_context(context);
        assert_eq!(mov.context_snapshot.len(), 2);
        assert_eq!(mov.context_snapshot.get("legal_entity"), Some(&"ENTITY_A".to_string()));

        // Test inherit_from_position
        let instrument = Instrument::Asset(Asset::Cash(AssetDetails::new("USD", "US Dollar", "USD")));
        let position = Position::new("POS-001", "WALLET-001", instrument, Decimal::new(1000, 0))
            .with_tag("desk", "desk_a")
            .with_tag("strategy", "carry");

        let mov = mov.inherit_from_position(&position);
        assert_eq!(mov.inherited_tags.len(), 2);
        assert_eq!(mov.inherited_tags.get("desk"), Some(&"desk_a".to_string()));
        assert_eq!(mov.inherited_tags.get("strategy"), Some(&"carry".to_string()));
    }

    // Test 3: LedgerError variants
    #[test]
    fn test_ledger_error_variants() {
        // PositionNotFound
        let err = LedgerError::PositionNotFound("POS-999".to_string());
        assert_eq!(err.to_string(), "Position not found: POS-999");

        // WalletNotFound
        let err = LedgerError::WalletNotFound("WALLET-999".to_string());
        assert_eq!(err.to_string(), "Wallet not found: WALLET-999");

        // DuplicatePosition
        let err = LedgerError::DuplicatePosition("POS-001".to_string());
        assert_eq!(err.to_string(), "Duplicate position ID: POS-001");

        // InsufficientBalance
        let err = LedgerError::InsufficientBalance {
            position_id: "POS-001".to_string(),
            required: "1000".to_string(),
            available: "500".to_string(),
        };
        assert!(err.to_string().contains("Insufficient balance"));
        assert!(err.to_string().contains("POS-001"));

        // InvalidMove
        let err = LedgerError::InvalidMove("Negative amount not allowed".to_string());
        assert_eq!(err.to_string(), "Invalid move: Negative amount not allowed");

        // StorageError
        let err = LedgerError::StorageError("Database connection failed".to_string());
        assert_eq!(err.to_string(), "Storage error: Database connection failed");
    }

    // Test 4: TagCriterion and PortfolioFilter matching
    #[test]
    fn test_portfolio_filter_matching() {
        let mut tags = HashMap::new();
        tags.insert("desk".to_string(), "desk_a".to_string());
        tags.insert("strategy".to_string(), "yield_enhancement".to_string());
        tags.insert("trader".to_string(), "john".to_string());

        // Test TagCriterion::Equals
        let criterion = TagCriterion::Equals("desk".to_string(), "desk_a".to_string());
        assert!(criterion.matches(&tags));
        let criterion = TagCriterion::Equals("desk".to_string(), "desk_b".to_string());
        assert!(!criterion.matches(&tags));

        // Test TagCriterion::In
        let criterion = TagCriterion::In(
            "desk".to_string(),
            vec!["desk_a".to_string(), "desk_b".to_string()],
        );
        assert!(criterion.matches(&tags));
        let criterion = TagCriterion::In(
            "desk".to_string(),
            vec!["desk_x".to_string(), "desk_y".to_string()],
        );
        assert!(!criterion.matches(&tags));

        // Test TagCriterion::Exists
        let criterion = TagCriterion::Exists("trader".to_string());
        assert!(criterion.matches(&tags));
        let criterion = TagCriterion::Exists("client".to_string());
        assert!(!criterion.matches(&tags));

        // Test PortfolioFilter with multiple criteria (AND logic)
        let filter = PortfolioFilter::new("Desk A Yield Portfolio")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()))
            .with_criterion(TagCriterion::Equals(
                "strategy".to_string(),
                "yield_enhancement".to_string(),
            ));
        assert!(filter.matches(&tags));

        // Filter should NOT match if one criterion fails
        let filter = PortfolioFilter::new("Desk B Portfolio")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_b".to_string()));
        assert!(!filter.matches(&tags));

        // Empty filter matches everything
        let empty_filter = PortfolioFilter::new("All Positions");
        assert!(empty_filter.matches(&tags));
    }

    // Test 5: MoveType enumeration
    #[test]
    fn test_move_type_enumeration() {
        let move_types = vec![
            (MoveType::Transfer, "Transfer", 1),
            (MoveType::Open, "Open", 2),
            (MoveType::Close, "Close", 3),
            (MoveType::Adjustment, "Adjustment", 4),
            (MoveType::Settlement, "Settlement", 5),
            (MoveType::Coupon, "Coupon", 6),
            (MoveType::Fee, "Fee", 7),
            (MoveType::Tax, "Tax", 8),
        ];

        for (move_type, name, value) in move_types {
            assert_eq!(move_type.as_str(), name);
            assert_eq!(move_type as i32, value);
            assert_eq!(MoveType::from_i32(value), Some(move_type));
        }

        // Invalid values
        assert_eq!(MoveType::from_i32(0), None);
        assert_eq!(MoveType::from_i32(9), None);

        // JSON serialization roundtrip
        let move_type = MoveType::Settlement;
        let json = serde_json::to_string(&move_type).unwrap();
        let deserialized: MoveType = serde_json::from_str(&json).unwrap();
        assert_eq!(move_type, deserialized);
    }

    // Test 6: Move protobuf serialization
    #[test]
    fn test_move_protobuf_serialization() {
        let effective_date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let mut context = HashMap::new();
        context.insert("legal_entity".to_string(), "ENTITY_A".to_string());

        let mut inherited_tags = HashMap::new();
        inherited_tags.insert("desk".to_string(), "desk_a".to_string());

        let mov = Move::new(
            "MOV-001",
            PositionId::new("POS-001"),
            MoveType::Settlement,
            "CASH",
            "PRODUCT",
            Decimal::new(15025, 2),
            effective_date,
            "ACTION-001",
            "RULE-001",
        )
        .with_context(context)
        .with_inherited_tags(inherited_tags);

        // Encode to protobuf
        let buf = mov.encode_proto();
        assert!(!buf.is_empty());

        // Decode the proto struct
        let proto = MoveProto::decode(&buf[..]).unwrap();
        assert_eq!(proto.id, "MOV-001");
        assert_eq!(proto.position_id, "POS-001");
        assert_eq!(proto.move_type, 5); // Settlement
        assert_eq!(proto.debit_account, "CASH");
        assert_eq!(proto.credit_account, "PRODUCT");
        assert_eq!(proto.amount, "150.25");
        assert_eq!(proto.effective_date, "2024-06-15");
        assert_eq!(proto.source_action, "ACTION-001");
        assert_eq!(proto.rule_applied, "RULE-001");
        assert_eq!(proto.context_snapshot.get("legal_entity"), Some(&"ENTITY_A".to_string()));
        assert_eq!(proto.inherited_tags.get("desk"), Some(&"desk_a".to_string()));
    }
}
