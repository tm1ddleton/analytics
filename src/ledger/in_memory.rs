//! In-memory ledger implementation for research and simulation use.
//!
//! This module provides a storage-agnostic in-memory implementation of the Ledger trait,
//! suitable for research, simulation, and testing scenarios.

use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use super::types::{
    LedgerError, LedgerReader, LedgerResult, LedgerWriter, Move, PortfolioFilter, Position,
    PositionId, TagKey, TagValue, WalletId,
};

/// Wallet - a grouping of positions with default tags.
#[derive(Debug, Clone, PartialEq)]
pub struct Wallet {
    /// Unique wallet identifier
    pub id: WalletId,

    /// Display name for the wallet
    pub name: String,

    /// Default tags applied to new positions in this wallet
    pub default_tags: HashMap<TagKey, TagValue>,
}

impl Wallet {
    /// Create a new wallet.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Wallet {
            id: WalletId::new(id),
            name: name.into(),
            default_tags: HashMap::new(),
        }
    }

    /// Add a default tag to this wallet.
    pub fn with_default_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_tags.insert(key.into(), value.into());
        self
    }

    /// Add multiple default tags to this wallet.
    pub fn with_default_tags(mut self, tags: HashMap<TagKey, TagValue>) -> Self {
        self.default_tags.extend(tags);
        self
    }
}

/// Counter for generating unique IDs.
static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn generate_position_id() -> PositionId {
    let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    PositionId::new(format!("POS-{:08}", id))
}

/// In-memory ledger implementation.
///
/// Provides O(1) position lookup and supports portfolio filtering via tag criteria.
/// Suitable for research, simulation, and testing scenarios.
///
/// Implements both `LedgerReader` and `LedgerWriter`, which through the blanket
/// implementation in types.rs automatically provides the `Ledger` trait.
///
/// ## Tag Inheritance
///
/// The ledger supports automatic tag inheritance:
/// - Wallet default tags are copied to new positions when using `create_position_with_inheritance`
/// - Position tags override wallet defaults (position-level tags take precedence)
/// - Moves can inherit position tags via `record_moves_with_inheritance`
#[derive(Debug, Default)]
pub struct InMemoryLedger {
    /// Positions indexed by ID for O(1) lookup
    positions: HashMap<PositionId, Position>,

    /// All recorded moves (append-only)
    moves: Vec<Move>,

    /// Wallets indexed by ID
    wallets: HashMap<WalletId, Wallet>,
}

impl InMemoryLedger {
    /// Create a new empty in-memory ledger.
    pub fn new() -> Self {
        InMemoryLedger {
            positions: HashMap::new(),
            moves: Vec::new(),
            wallets: HashMap::new(),
        }
    }

    /// Create a wallet in the ledger.
    pub fn create_wallet(&mut self, wallet: Wallet) -> LedgerResult<WalletId> {
        let id = wallet.id.clone();
        if self.wallets.contains_key(&id) {
            return Err(LedgerError::StorageError(format!(
                "Wallet already exists: {}",
                id
            )));
        }
        self.wallets.insert(id.clone(), wallet);
        Ok(id)
    }

    /// Get a wallet by ID.
    pub fn get_wallet(&self, id: &WalletId) -> LedgerResult<&Wallet> {
        self.wallets
            .get(id)
            .ok_or_else(|| LedgerError::WalletNotFound(id.to_string()))
    }

    /// Get all positions (for iteration/debugging).
    pub fn all_positions(&self) -> impl Iterator<Item = &Position> {
        self.positions.values()
    }

    /// Get all moves (for iteration/debugging).
    pub fn all_moves(&self) -> &[Move] {
        &self.moves
    }

    /// Get the number of positions.
    pub fn position_count(&self) -> usize {
        self.positions.len()
    }

    /// Get the number of moves.
    pub fn move_count(&self) -> usize {
        self.moves.len()
    }

    /// Create a position with tag inheritance from wallet.
    ///
    /// Wallet default tags are applied first, then position tags override them.
    /// This implements the tag inheritance hierarchy: Wallet -> Position.
    pub fn create_position_with_inheritance(
        &mut self,
        mut position: Position,
    ) -> LedgerResult<PositionId> {
        // Generate ID if the position has an empty ID
        if position.id.as_str().is_empty() {
            position.id = generate_position_id();
        }

        // Check for duplicate
        if self.positions.contains_key(&position.id) {
            return Err(LedgerError::DuplicatePosition(position.id.to_string()));
        }

        // Apply wallet default tags if wallet exists
        if let Some(wallet) = self.wallets.get(&position.wallet_id) {
            // Start with wallet defaults
            let mut merged_tags = wallet.default_tags.clone();
            // Position tags override wallet defaults
            merged_tags.extend(position.tags);
            position.tags = merged_tags;
        }

        let id = position.id.clone();
        self.positions.insert(id.clone(), position);
        Ok(id)
    }

    /// Record moves with automatic tag inheritance from their positions.
    ///
    /// Each move's `inherited_tags` will be populated from the associated position's tags.
    /// This implements the tag inheritance hierarchy: Position -> Move.
    pub fn record_moves_with_inheritance(&mut self, mut moves: Vec<Move>) -> LedgerResult<()> {
        // Validate and apply tag inheritance for all moves
        for mov in &mut moves {
            let position = self.positions.get(&mov.position_id).ok_or_else(|| {
                LedgerError::PositionNotFound(mov.position_id.to_string())
            })?;

            // Inherit tags from position
            mov.inherited_tags = position.tags.clone();
        }

        // Append all moves
        self.moves.extend(moves);
        Ok(())
    }
}

impl LedgerReader for InMemoryLedger {
    /// Get a position by ID - O(1) lookup.
    fn get_position(&self, id: &PositionId) -> LedgerResult<Position> {
        self.positions
            .get(id)
            .cloned()
            .ok_or_else(|| LedgerError::PositionNotFound(id.to_string()))
    }

    /// Get all positions in a wallet - filters by wallet_id.
    fn get_positions_by_wallet(&self, wallet_id: &WalletId) -> LedgerResult<Vec<Position>> {
        let positions: Vec<Position> = self
            .positions
            .values()
            .filter(|p| &p.wallet_id == wallet_id)
            .cloned()
            .collect();
        Ok(positions)
    }

    /// Get the current balance for a position - sums all moves affecting this position.
    ///
    /// Balance is calculated as: sum of credits - sum of debits for the position.
    /// This aggregates all moves where `position_id` matches.
    fn get_balance(&self, position_id: &PositionId) -> LedgerResult<Decimal> {
        // Verify position exists
        if !self.positions.contains_key(position_id) {
            return Err(LedgerError::PositionNotFound(position_id.to_string()));
        }

        // Sum all moves for this position
        let balance = self
            .moves
            .iter()
            .filter(|m| &m.position_id == position_id)
            .map(|m| m.amount)
            .fold(Decimal::ZERO, |acc, amount| acc + amount);

        Ok(balance)
    }

    /// Get positions matching a portfolio filter - filters by tag criteria (AND logic).
    fn positions_by_portfolio(&self, filter: &PortfolioFilter) -> LedgerResult<Vec<Position>> {
        let positions: Vec<Position> = self
            .positions
            .values()
            .filter(|p| filter.matches(&p.tags))
            .cloned()
            .collect();
        Ok(positions)
    }

    /// Get moves for positions matching a portfolio filter.
    ///
    /// Filters moves by their inherited_tags using the portfolio criteria.
    fn moves_by_portfolio(&self, filter: &PortfolioFilter) -> LedgerResult<Vec<Move>> {
        let moves: Vec<Move> = self
            .moves
            .iter()
            .filter(|m| filter.matches(&m.inherited_tags))
            .cloned()
            .collect();
        Ok(moves)
    }
}

impl LedgerWriter for InMemoryLedger {
    /// Create a new position - generates ID if not set, stores position.
    ///
    /// Note: This method does NOT apply wallet tag inheritance.
    /// Use `create_position_with_inheritance` for automatic tag inheritance.
    fn create_position(&mut self, mut position: Position) -> LedgerResult<PositionId> {
        // Generate ID if the position has an empty ID
        if position.id.as_str().is_empty() {
            position.id = generate_position_id();
        }

        // Check for duplicate
        if self.positions.contains_key(&position.id) {
            return Err(LedgerError::DuplicatePosition(position.id.to_string()));
        }

        let id = position.id.clone();
        self.positions.insert(id.clone(), position);
        Ok(id)
    }

    /// Record a batch of moves to the ledger - appends moves.
    ///
    /// Note: This method does NOT apply position tag inheritance.
    /// Use `record_moves_with_inheritance` for automatic tag inheritance.
    fn record_moves(&mut self, moves: Vec<Move>) -> LedgerResult<()> {
        // Validate all moves reference existing positions
        for mov in &moves {
            if !self.positions.contains_key(&mov.position_id) {
                return Err(LedgerError::PositionNotFound(mov.position_id.to_string()));
            }
        }

        // Append all moves
        self.moves.extend(moves);
        Ok(())
    }
}

// Note: InMemoryLedger automatically implements the Ledger trait via the blanket
// implementation in types.rs: impl<T> Ledger for T where T: LedgerReader + LedgerWriter {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::domain::{Asset, AssetDetails, Instrument, Product, ProductDetails};
    use crate::ledger::types::{Move, MoveType, TagCriterion};
    use chrono::NaiveDate;

    // Helper to create a test position with tags
    fn create_test_position(
        id: &str,
        wallet_id: &str,
        tags: Vec<(&str, &str)>,
    ) -> Position {
        let instrument = Instrument::Product(Product::KnockOutWarrant(ProductDetails::new(
            format!("{}-product", id),
            "Test Knock-Out",
            "AAPL",
            "USD",
        )));

        let mut position = Position::new(id, wallet_id, instrument, Decimal::new(100, 0));
        for (key, value) in tags {
            position = position.with_tag(key, value);
        }
        position
    }

    // Helper to create a test move
    fn create_test_move(
        id: &str,
        position_id: &PositionId,
        amount: Decimal,
        tags: Vec<(&str, &str)>,
    ) -> Move {
        let effective_date = NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
        let mut mov = Move::new(
            id,
            position_id.clone(),
            MoveType::Settlement,
            "CASH",
            "PRODUCT",
            amount,
            effective_date,
            "ACTION-001",
            "RULE-001",
        );
        let mut inherited: HashMap<TagKey, TagValue> = HashMap::new();
        for (key, value) in tags {
            inherited.insert(key.to_string(), value.to_string());
        }
        mov.inherited_tags = inherited;
        mov
    }

    // Test 1: Position creation and retrieval
    #[test]
    fn test_position_creation_and_retrieval() {
        let mut ledger = InMemoryLedger::new();

        // Create a position
        let position = create_test_position("POS-001", "WALLET-001", vec![("desk", "desk_a")]);
        let id = ledger.create_position(position.clone()).unwrap();
        assert_eq!(id.as_str(), "POS-001");

        // Retrieve the position
        let retrieved = ledger.get_position(&id).unwrap();
        assert_eq!(retrieved.id, position.id);
        assert_eq!(retrieved.wallet_id.as_str(), "WALLET-001");
        assert!(retrieved.has_tag("desk", "desk_a"));

        // Verify O(1) lookup works (implicit - using HashMap)
        assert_eq!(ledger.position_count(), 1);

        // Attempt to create duplicate should fail
        let dup_position = create_test_position("POS-001", "WALLET-002", vec![]);
        let result = ledger.create_position(dup_position);
        assert!(matches!(result, Err(LedgerError::DuplicatePosition(_))));

        // Position not found should return error
        let result = ledger.get_position(&PositionId::new("POS-999"));
        assert!(matches!(result, Err(LedgerError::PositionNotFound(_))));
    }

    // Test 2: Move recording and balance calculation
    #[test]
    fn test_move_recording_and_balance_calculation() {
        let mut ledger = InMemoryLedger::new();

        // Create a position
        let position = create_test_position("POS-001", "WALLET-001", vec![]);
        let pos_id = ledger.create_position(position).unwrap();

        // Initial balance should be zero
        let balance = ledger.get_balance(&pos_id).unwrap();
        assert_eq!(balance, Decimal::ZERO);

        // Record some moves
        let moves = vec![
            create_test_move("MOV-001", &pos_id, Decimal::new(10000, 2), vec![]), // +100.00
            create_test_move("MOV-002", &pos_id, Decimal::new(5000, 2), vec![]),  // +50.00
            create_test_move("MOV-003", &pos_id, Decimal::new(-2500, 2), vec![]), // -25.00
        ];
        ledger.record_moves(moves).unwrap();

        // Balance should be sum of moves: 100 + 50 - 25 = 125.00
        let balance = ledger.get_balance(&pos_id).unwrap();
        assert_eq!(balance, Decimal::new(12500, 2));

        // Verify move count
        assert_eq!(ledger.move_count(), 3);

        // Recording moves for non-existent position should fail
        let invalid_moves = vec![create_test_move(
            "MOV-004",
            &PositionId::new("POS-999"),
            Decimal::new(100, 0),
            vec![],
        )];
        let result = ledger.record_moves(invalid_moves);
        assert!(matches!(result, Err(LedgerError::PositionNotFound(_))));

        // Balance for non-existent position should fail
        let result = ledger.get_balance(&PositionId::new("POS-999"));
        assert!(matches!(result, Err(LedgerError::PositionNotFound(_))));
    }

    // Test 3: Portfolio filtering with tag criteria
    #[test]
    fn test_portfolio_filtering_with_tag_criteria() {
        let mut ledger = InMemoryLedger::new();

        // Create positions with various tags
        let pos1 = create_test_position(
            "POS-001",
            "WALLET-001",
            vec![("desk", "desk_a"), ("strategy", "yield")],
        );
        let pos2 = create_test_position(
            "POS-002",
            "WALLET-001",
            vec![("desk", "desk_a"), ("strategy", "momentum")],
        );
        let pos3 = create_test_position(
            "POS-003",
            "WALLET-001",
            vec![("desk", "desk_b"), ("strategy", "yield")],
        );
        let pos4 = create_test_position(
            "POS-004",
            "WALLET-002",
            vec![("desk", "desk_b"), ("strategy", "carry")],
        );

        let id1 = ledger.create_position(pos1).unwrap();
        let id2 = ledger.create_position(pos2).unwrap();
        let id3 = ledger.create_position(pos3).unwrap();
        let _id4 = ledger.create_position(pos4).unwrap();

        // Record moves with inherited tags
        ledger
            .record_moves(vec![
                create_test_move(
                    "MOV-001",
                    &id1,
                    Decimal::new(100, 0),
                    vec![("desk", "desk_a"), ("strategy", "yield")],
                ),
                create_test_move(
                    "MOV-002",
                    &id2,
                    Decimal::new(200, 0),
                    vec![("desk", "desk_a"), ("strategy", "momentum")],
                ),
                create_test_move(
                    "MOV-003",
                    &id3,
                    Decimal::new(300, 0),
                    vec![("desk", "desk_b"), ("strategy", "yield")],
                ),
            ])
            .unwrap();

        // Filter by single tag (Equals)
        let filter = PortfolioFilter::new("Desk A")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()));
        let positions = ledger.positions_by_portfolio(&filter).unwrap();
        assert_eq!(positions.len(), 2);

        // Filter by multiple tags (AND logic)
        let filter = PortfolioFilter::new("Desk A Yield")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()))
            .with_criterion(TagCriterion::Equals(
                "strategy".to_string(),
                "yield".to_string(),
            ));
        let positions = ledger.positions_by_portfolio(&filter).unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].id.as_str(), "POS-001");

        // Filter using In criterion
        let filter = PortfolioFilter::new("Yield or Momentum").with_criterion(TagCriterion::In(
            "strategy".to_string(),
            vec!["yield".to_string(), "momentum".to_string()],
        ));
        let positions = ledger.positions_by_portfolio(&filter).unwrap();
        assert_eq!(positions.len(), 3);

        // Filter using Exists criterion
        let filter = PortfolioFilter::new("Has Desk Tag")
            .with_criterion(TagCriterion::Exists("desk".to_string()));
        let positions = ledger.positions_by_portfolio(&filter).unwrap();
        assert_eq!(positions.len(), 4);

        // Filter moves by portfolio
        let filter = PortfolioFilter::new("Desk A Moves")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()));
        let moves = ledger.moves_by_portfolio(&filter).unwrap();
        assert_eq!(moves.len(), 2);

        // Empty filter matches all
        let filter = PortfolioFilter::new("All");
        let positions = ledger.positions_by_portfolio(&filter).unwrap();
        assert_eq!(positions.len(), 4);
    }

    // Test 4: Wallet-scoped position queries
    #[test]
    fn test_wallet_scoped_position_queries() {
        let mut ledger = InMemoryLedger::new();

        // Create wallets
        let wallet1 = Wallet::new("WALLET-001", "Trading Book A")
            .with_default_tag("book", "trading");
        let wallet2 = Wallet::new("WALLET-002", "Trading Book B")
            .with_default_tag("book", "banking");

        ledger.create_wallet(wallet1).unwrap();
        ledger.create_wallet(wallet2).unwrap();

        // Create positions in different wallets
        let pos1 = create_test_position("POS-001", "WALLET-001", vec![("trader", "alice")]);
        let pos2 = create_test_position("POS-002", "WALLET-001", vec![("trader", "bob")]);
        let pos3 = create_test_position("POS-003", "WALLET-002", vec![("trader", "charlie")]);

        ledger.create_position(pos1).unwrap();
        ledger.create_position(pos2).unwrap();
        ledger.create_position(pos3).unwrap();

        // Query positions by wallet
        let wallet1_positions = ledger
            .get_positions_by_wallet(&WalletId::new("WALLET-001"))
            .unwrap();
        assert_eq!(wallet1_positions.len(), 2);

        let wallet2_positions = ledger
            .get_positions_by_wallet(&WalletId::new("WALLET-002"))
            .unwrap();
        assert_eq!(wallet2_positions.len(), 1);
        assert_eq!(wallet2_positions[0].id.as_str(), "POS-003");

        // Query non-existent wallet returns empty (not error)
        let empty_positions = ledger
            .get_positions_by_wallet(&WalletId::new("WALLET-999"))
            .unwrap();
        assert!(empty_positions.is_empty());

        // Verify wallet retrieval
        let wallet = ledger.get_wallet(&WalletId::new("WALLET-001")).unwrap();
        assert_eq!(wallet.name, "Trading Book A");
        assert_eq!(
            wallet.default_tags.get("book"),
            Some(&"trading".to_string())
        );

        // Wallet not found
        let result = ledger.get_wallet(&WalletId::new("WALLET-999"));
        assert!(matches!(result, Err(LedgerError::WalletNotFound(_))));

        // Duplicate wallet creation should fail
        let dup_wallet = Wallet::new("WALLET-001", "Duplicate");
        let result = ledger.create_wallet(dup_wallet);
        assert!(matches!(result, Err(LedgerError::StorageError(_))));
    }

    // Test 5: Position appearing in multiple portfolio queries (non-exclusive)
    #[test]
    fn test_non_exclusive_portfolio_membership() {
        let mut ledger = InMemoryLedger::new();

        // Create a position with multiple tags
        let position = create_test_position(
            "POS-001",
            "WALLET-001",
            vec![
                ("desk", "desk_a"),
                ("strategy", "yield"),
                ("region", "EMEA"),
                ("client", "client_x"),
            ],
        );
        ledger.create_position(position).unwrap();

        // This single position should appear in multiple portfolio views
        let desk_a_filter = PortfolioFilter::new("Desk A")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()));
        let yield_filter = PortfolioFilter::new("Yield Strategy")
            .with_criterion(TagCriterion::Equals(
                "strategy".to_string(),
                "yield".to_string(),
            ));
        let emea_filter = PortfolioFilter::new("EMEA Region")
            .with_criterion(TagCriterion::Equals("region".to_string(), "EMEA".to_string()));
        let client_x_filter = PortfolioFilter::new("Client X")
            .with_criterion(TagCriterion::Equals(
                "client".to_string(),
                "client_x".to_string(),
            ));

        // Position appears in all four portfolio views
        assert_eq!(ledger.positions_by_portfolio(&desk_a_filter).unwrap().len(), 1);
        assert_eq!(ledger.positions_by_portfolio(&yield_filter).unwrap().len(), 1);
        assert_eq!(ledger.positions_by_portfolio(&emea_filter).unwrap().len(), 1);
        assert_eq!(
            ledger.positions_by_portfolio(&client_x_filter).unwrap().len(),
            1
        );

        // Combined filter (AND logic) also matches
        let combined_filter = PortfolioFilter::new("Desk A + EMEA")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()))
            .with_criterion(TagCriterion::Equals("region".to_string(), "EMEA".to_string()));
        assert_eq!(
            ledger.positions_by_portfolio(&combined_filter).unwrap().len(),
            1
        );
    }

    // Test 6: Auto-generated position IDs
    #[test]
    fn test_auto_generated_position_ids() {
        let mut ledger = InMemoryLedger::new();

        // Create a position with empty ID - should auto-generate
        let instrument = Instrument::Asset(Asset::Cash(AssetDetails::new("USD", "US Dollar", "USD")));
        let position = Position::new("", "WALLET-001", instrument.clone(), Decimal::new(1000, 0));

        let id1 = ledger.create_position(position).unwrap();
        assert!(id1.as_str().starts_with("POS-"));

        // Create another - should get different ID
        let position2 = Position::new("", "WALLET-001", instrument, Decimal::new(2000, 0));
        let id2 = ledger.create_position(position2).unwrap();
        assert!(id2.as_str().starts_with("POS-"));
        assert_ne!(id1, id2);

        // Both positions should be retrievable
        assert!(ledger.get_position(&id1).is_ok());
        assert!(ledger.get_position(&id2).is_ok());
    }
}

/// Tests specifically for the Portfolio Tagging System (Task Group 10).
///
/// These tests validate:
/// - TagCriterion matching (Equals, In, Exists)
/// - PortfolioFilter with multiple criteria (AND logic)
/// - Position appearing in multiple portfolio queries
/// - Move tag inheritance from position
/// - Wallet -> Position tag inheritance
#[cfg(test)]
mod portfolio_tagging_tests {
    use super::*;
    use crate::ledger::domain::{Instrument, Product, ProductDetails};
    use crate::ledger::types::{Move, MoveType, TagCriterion};
    use chrono::NaiveDate;

    // Helper to create a minimal test position
    fn make_position(id: &str, wallet_id: &str, tags: Vec<(&str, &str)>) -> Position {
        let instrument = Instrument::Product(Product::KnockOutWarrant(ProductDetails::new(
            format!("{}-product", id),
            "Test Product",
            "AAPL",
            "USD",
        )));

        let mut position = Position::new(id, wallet_id, instrument, Decimal::new(100, 0));
        for (key, value) in tags {
            position = position.with_tag(key, value);
        }
        position
    }

    // Helper to create a minimal test move (without tags - to test inheritance)
    fn make_move(id: &str, position_id: &PositionId, amount: Decimal) -> Move {
        Move::new(
            id,
            position_id.clone(),
            MoveType::Settlement,
            "CASH",
            "PRODUCT",
            amount,
            NaiveDate::from_ymd_opt(2024, 12, 15).unwrap(),
            "ACTION-001",
            "RULE-001",
        )
    }

    /// Test 1: TagCriterion matching - Equals, In, Exists variants
    #[test]
    fn test_tag_criterion_matching() {
        let mut tags = HashMap::new();
        tags.insert("strategy".to_string(), "yield_enhancement".to_string());
        tags.insert("desk".to_string(), "desk_a".to_string());
        tags.insert("trader".to_string(), "john".to_string());

        // Equals - exact match
        let equals_match = TagCriterion::Equals("strategy".to_string(), "yield_enhancement".to_string());
        assert!(equals_match.matches(&tags), "Equals should match exact value");

        let equals_no_match = TagCriterion::Equals("strategy".to_string(), "momentum".to_string());
        assert!(!equals_no_match.matches(&tags), "Equals should not match different value");

        let equals_missing_key = TagCriterion::Equals("client".to_string(), "client_x".to_string());
        assert!(!equals_missing_key.matches(&tags), "Equals should not match missing key");

        // In - value in list
        let in_match = TagCriterion::In(
            "desk".to_string(),
            vec!["desk_a".to_string(), "desk_b".to_string(), "desk_c".to_string()],
        );
        assert!(in_match.matches(&tags), "In should match when value is in list");

        let in_no_match = TagCriterion::In(
            "desk".to_string(),
            vec!["desk_x".to_string(), "desk_y".to_string()],
        );
        assert!(!in_no_match.matches(&tags), "In should not match when value not in list");

        let in_missing_key = TagCriterion::In(
            "region".to_string(),
            vec!["EMEA".to_string(), "APAC".to_string()],
        );
        assert!(!in_missing_key.matches(&tags), "In should not match missing key");

        // Exists - key presence
        let exists_match = TagCriterion::Exists("trader".to_string());
        assert!(exists_match.matches(&tags), "Exists should match when key present");

        let exists_no_match = TagCriterion::Exists("book".to_string());
        assert!(!exists_no_match.matches(&tags), "Exists should not match missing key");
    }

    /// Test 2: PortfolioFilter with multiple criteria (AND logic)
    #[test]
    fn test_portfolio_filter_and_logic() {
        let mut ledger = InMemoryLedger::new();

        // Create positions with different tag combinations
        let pos1 = make_position("POS-001", "WALLET-001", vec![
            ("desk", "desk_a"),
            ("strategy", "yield"),
            ("region", "EMEA"),
        ]);
        let pos2 = make_position("POS-002", "WALLET-001", vec![
            ("desk", "desk_a"),
            ("strategy", "momentum"),
            ("region", "EMEA"),
        ]);
        let pos3 = make_position("POS-003", "WALLET-001", vec![
            ("desk", "desk_b"),
            ("strategy", "yield"),
            ("region", "APAC"),
        ]);

        ledger.create_position(pos1).unwrap();
        ledger.create_position(pos2).unwrap();
        ledger.create_position(pos3).unwrap();

        // Single criterion filter
        let desk_a_filter = PortfolioFilter::new("Desk A Only")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()));
        let results = ledger.positions_by_portfolio(&desk_a_filter).unwrap();
        assert_eq!(results.len(), 2, "Should match 2 positions with desk=desk_a");

        // Two criteria (AND): desk_a AND yield
        let desk_a_yield_filter = PortfolioFilter::new("Desk A + Yield")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()))
            .with_criterion(TagCriterion::Equals("strategy".to_string(), "yield".to_string()));
        let results = ledger.positions_by_portfolio(&desk_a_yield_filter).unwrap();
        assert_eq!(results.len(), 1, "Should match only POS-001");
        assert_eq!(results[0].id.as_str(), "POS-001");

        // Three criteria (AND): desk_a AND EMEA AND strategy exists
        let complex_filter = PortfolioFilter::new("Complex Filter")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()))
            .with_criterion(TagCriterion::Equals("region".to_string(), "EMEA".to_string()))
            .with_criterion(TagCriterion::Exists("strategy".to_string()));
        let results = ledger.positions_by_portfolio(&complex_filter).unwrap();
        assert_eq!(results.len(), 2, "Should match POS-001 and POS-002");

        // Filter with no matches
        let no_match_filter = PortfolioFilter::new("No Match")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_c".to_string()));
        let results = ledger.positions_by_portfolio(&no_match_filter).unwrap();
        assert_eq!(results.len(), 0, "Should match no positions");

        // Empty filter matches all
        let all_filter = PortfolioFilter::new("All Positions");
        let results = ledger.positions_by_portfolio(&all_filter).unwrap();
        assert_eq!(results.len(), 3, "Empty filter should match all positions");
    }

    /// Test 3: Position appearing in multiple portfolio queries (non-exclusive membership)
    #[test]
    fn test_non_exclusive_portfolio_membership() {
        let mut ledger = InMemoryLedger::new();

        // Create a single position with multiple classification dimensions
        let position = make_position("POS-MULTI", "WALLET-001", vec![
            ("desk", "desk_a"),
            ("strategy", "yield_enhancement"),
            ("region", "EMEA"),
            ("client", "institutional"),
            ("product_type", "structured"),
            ("underlying", "AAPL"),
        ]);
        ledger.create_position(position).unwrap();

        // Define multiple portfolio views
        let portfolios = vec![
            ("By Desk", TagCriterion::Equals("desk".to_string(), "desk_a".to_string())),
            ("By Strategy", TagCriterion::Equals("strategy".to_string(), "yield_enhancement".to_string())),
            ("By Region", TagCriterion::Equals("region".to_string(), "EMEA".to_string())),
            ("By Client Type", TagCriterion::Equals("client".to_string(), "institutional".to_string())),
            ("By Product", TagCriterion::Equals("product_type".to_string(), "structured".to_string())),
            ("By Underlying", TagCriterion::Equals("underlying".to_string(), "AAPL".to_string())),
        ];

        // Same position should appear in ALL portfolio views
        for (name, criterion) in portfolios {
            let filter = PortfolioFilter::new(name).with_criterion(criterion);
            let results = ledger.positions_by_portfolio(&filter).unwrap();
            assert_eq!(
                results.len(), 1,
                "Position should appear in '{}' portfolio",
                name
            );
            assert_eq!(results[0].id.as_str(), "POS-MULTI");
        }

        // Position should also match combined filters
        let combined = PortfolioFilter::new("Desk A + EMEA + Institutional")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()))
            .with_criterion(TagCriterion::Equals("region".to_string(), "EMEA".to_string()))
            .with_criterion(TagCriterion::Equals("client".to_string(), "institutional".to_string()));
        let results = ledger.positions_by_portfolio(&combined).unwrap();
        assert_eq!(results.len(), 1, "Position should match combined filter");
    }

    /// Test 4: Move tag inheritance from position
    #[test]
    fn test_move_tag_inheritance_from_position() {
        let mut ledger = InMemoryLedger::new();

        // Create a position with tags
        let position = make_position("POS-001", "WALLET-001", vec![
            ("desk", "desk_a"),
            ("strategy", "yield"),
            ("trader", "alice"),
        ]);
        let pos_id = ledger.create_position(position).unwrap();

        // Create moves without explicit tags - use inheritance method
        let moves = vec![
            make_move("MOV-001", &pos_id, Decimal::new(100, 0)),
            make_move("MOV-002", &pos_id, Decimal::new(200, 0)),
        ];

        // Record with inheritance
        ledger.record_moves_with_inheritance(moves).unwrap();

        // Verify moves have inherited tags
        let all_moves = ledger.all_moves();
        assert_eq!(all_moves.len(), 2);

        for mov in all_moves {
            assert_eq!(
                mov.inherited_tags.get("desk"),
                Some(&"desk_a".to_string()),
                "Move should inherit desk tag"
            );
            assert_eq!(
                mov.inherited_tags.get("strategy"),
                Some(&"yield".to_string()),
                "Move should inherit strategy tag"
            );
            assert_eq!(
                mov.inherited_tags.get("trader"),
                Some(&"alice".to_string()),
                "Move should inherit trader tag"
            );
        }

        // Verify moves can be filtered by portfolio
        let desk_filter = PortfolioFilter::new("Desk A Moves")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()));
        let filtered_moves = ledger.moves_by_portfolio(&desk_filter).unwrap();
        assert_eq!(filtered_moves.len(), 2, "Both moves should match desk filter");

        let trader_filter = PortfolioFilter::new("Alice's Moves")
            .with_criterion(TagCriterion::Equals("trader".to_string(), "alice".to_string()));
        let filtered_moves = ledger.moves_by_portfolio(&trader_filter).unwrap();
        assert_eq!(filtered_moves.len(), 2, "Both moves should match trader filter");
    }

    /// Test 5: Wallet default tags inherited by position
    #[test]
    fn test_wallet_to_position_tag_inheritance() {
        let mut ledger = InMemoryLedger::new();

        // Create a wallet with default tags
        let wallet = Wallet::new("WALLET-001", "Trading Book A")
            .with_default_tag("book", "trading")
            .with_default_tag("legal_entity", "ENTITY_A")
            .with_default_tag("desk", "default_desk");

        ledger.create_wallet(wallet).unwrap();

        // Create a position with some of its own tags (desk should override wallet default)
        let position = make_position("POS-001", "WALLET-001", vec![
            ("strategy", "yield"),
            ("desk", "desk_a"),  // This should override wallet's "default_desk"
        ]);

        // Use inheritance method
        let pos_id = ledger.create_position_with_inheritance(position).unwrap();

        // Retrieve and verify tags
        let retrieved = ledger.get_position(&pos_id).unwrap();

        // Inherited from wallet
        assert_eq!(
            retrieved.tags.get("book"),
            Some(&"trading".to_string()),
            "Should inherit 'book' tag from wallet"
        );
        assert_eq!(
            retrieved.tags.get("legal_entity"),
            Some(&"ENTITY_A".to_string()),
            "Should inherit 'legal_entity' tag from wallet"
        );

        // Position's own tag
        assert_eq!(
            retrieved.tags.get("strategy"),
            Some(&"yield".to_string()),
            "Should have position's own 'strategy' tag"
        );

        // Overridden tag - position value takes precedence
        assert_eq!(
            retrieved.tags.get("desk"),
            Some(&"desk_a".to_string()),
            "Position's 'desk' tag should override wallet default"
        );

        // Verify the position appears in correct portfolio views
        let trading_book_filter = PortfolioFilter::new("Trading Book")
            .with_criterion(TagCriterion::Equals("book".to_string(), "trading".to_string()));
        let results = ledger.positions_by_portfolio(&trading_book_filter).unwrap();
        assert_eq!(results.len(), 1, "Position should appear in Trading Book portfolio");

        let desk_a_filter = PortfolioFilter::new("Desk A")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "desk_a".to_string()));
        let results = ledger.positions_by_portfolio(&desk_a_filter).unwrap();
        assert_eq!(results.len(), 1, "Position should appear in Desk A portfolio (overridden)");

        // Should NOT appear in default_desk portfolio (was overridden)
        let default_desk_filter = PortfolioFilter::new("Default Desk")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "default_desk".to_string()));
        let results = ledger.positions_by_portfolio(&default_desk_filter).unwrap();
        assert_eq!(results.len(), 0, "Position should NOT appear in default_desk portfolio");
    }

    /// Test 6: Full tag inheritance chain - Wallet -> Position -> Move
    #[test]
    fn test_full_tag_inheritance_chain() {
        let mut ledger = InMemoryLedger::new();

        // Create wallet with default tags
        let wallet = Wallet::new("WALLET-001", "Yield Desk Book")
            .with_default_tag("book", "trading")
            .with_default_tag("desk", "yield_desk")
            .with_default_tag("regulatory_book", "FRTB");

        ledger.create_wallet(wallet).unwrap();

        // Create position that inherits from wallet and adds its own tags
        let position = make_position("POS-001", "WALLET-001", vec![
            ("strategy", "yield_enhancement"),
            ("trader", "bob"),
            // Note: 'desk' not set, so wallet default will be used
        ]);

        let pos_id = ledger.create_position_with_inheritance(position).unwrap();

        // Create moves that inherit from position
        let moves = vec![
            make_move("MOV-001", &pos_id, Decimal::new(1000, 0)),
            make_move("MOV-002", &pos_id, Decimal::new(-500, 0)),
        ];

        ledger.record_moves_with_inheritance(moves).unwrap();

        // Verify the full inheritance chain
        let all_moves = ledger.all_moves();
        assert_eq!(all_moves.len(), 2);

        for mov in all_moves {
            // From wallet (via position)
            assert_eq!(
                mov.inherited_tags.get("book"),
                Some(&"trading".to_string()),
                "Move should inherit 'book' from wallet->position chain"
            );
            assert_eq!(
                mov.inherited_tags.get("desk"),
                Some(&"yield_desk".to_string()),
                "Move should inherit 'desk' from wallet->position chain"
            );
            assert_eq!(
                mov.inherited_tags.get("regulatory_book"),
                Some(&"FRTB".to_string()),
                "Move should inherit 'regulatory_book' from wallet->position chain"
            );

            // From position
            assert_eq!(
                mov.inherited_tags.get("strategy"),
                Some(&"yield_enhancement".to_string()),
                "Move should inherit 'strategy' from position"
            );
            assert_eq!(
                mov.inherited_tags.get("trader"),
                Some(&"bob".to_string()),
                "Move should inherit 'trader' from position"
            );
        }

        // Verify P&L attribution via portfolio queries
        let yield_desk_filter = PortfolioFilter::new("Yield Desk P&L")
            .with_criterion(TagCriterion::Equals("desk".to_string(), "yield_desk".to_string()));
        let desk_moves = ledger.moves_by_portfolio(&yield_desk_filter).unwrap();
        assert_eq!(desk_moves.len(), 2, "All moves attributed to yield desk");

        let frtb_filter = PortfolioFilter::new("FRTB Book P&L")
            .with_criterion(TagCriterion::Equals("regulatory_book".to_string(), "FRTB".to_string()));
        let frtb_moves = ledger.moves_by_portfolio(&frtb_filter).unwrap();
        assert_eq!(frtb_moves.len(), 2, "All moves attributed to FRTB book");

        let bob_filter = PortfolioFilter::new("Bob's P&L")
            .with_criterion(TagCriterion::Equals("trader".to_string(), "bob".to_string()));
        let bob_moves = ledger.moves_by_portfolio(&bob_filter).unwrap();
        assert_eq!(bob_moves.len(), 2, "All moves attributed to Bob");
    }
}
