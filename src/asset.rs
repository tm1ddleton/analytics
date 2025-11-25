use crate::asset_key::AssetKey;

/// Trait for asset type discrimination and common asset behavior.
/// 
/// All asset types (Equity, Future) must implement this trait.
/// This provides a common interface for working with different asset types
/// while maintaining immutability.
pub trait Asset {
    /// Returns the asset key that uniquely identifies this asset.
    fn key(&self) -> &AssetKey;

    /// Returns the asset type as a string for identification.
    fn asset_type(&self) -> AssetType;
}

/// Enum representing the type of asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    /// Equity asset (stock)
    Equity,
    /// Futures contract
    Future,
}

impl AssetType {
    /// Returns a string representation of the asset type.
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetType::Equity => "Equity",
            AssetType::Future => "Future",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_type_equity() {
        let asset_type = AssetType::Equity;
        assert_eq!(asset_type.as_str(), "Equity");
    }

    #[test]
    fn test_asset_type_future() {
        let asset_type = AssetType::Future;
        assert_eq!(asset_type.as_str(), "Future");
    }

    #[test]
    fn test_asset_type_discrimination() {
        let equity_type = AssetType::Equity;
        let future_type = AssetType::Future;
        assert_ne!(equity_type, future_type);
    }
}

