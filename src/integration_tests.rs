// Integration tests for end-to-end workflows and critical user scenarios

#[cfg(test)]
mod integration_tests {
    use crate::asset::{Asset, AssetType};
    use crate::asset_key::AssetKey;
    use crate::equity::{CorporateAction, Equity};
    use crate::future::Future;
    use crate::time_series::{DataProvider, DateRange, InMemoryDataProvider, TimeSeriesPoint};
    use chrono::{NaiveDate, TimeZone, Utc};

    /// Test end-to-end workflow: Create equity -> Query data -> Apply corporate actions
    #[test]
    fn test_equity_end_to_end_workflow() {
        // Create equity with corporate actions
        let split = CorporateAction::Split {
            ratio: 2.0,
            effective_date: NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
        };
        let equity = Equity::with_corporate_actions(
            "AAPL",
            "Apple Inc.",
            "NASDAQ",
            "USD",
            "Technology",
            vec![split],
        )
        .unwrap();

        // Set up data provider with time-series data
        let mut provider = InMemoryDataProvider::new();
        let points = vec![
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap(),
                200.0, // Before split
            ),
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 1, 17, 16, 0, 0).unwrap(),
                200.0, // After split
            ),
        ];
        provider.add_data(equity.key().clone(), points);

        // Query time-series data
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(),
        );
        let raw_data = equity.get_time_series(&provider, &date_range).unwrap();

        // Apply corporate actions
        let adjusted_data = equity.apply_corporate_actions_to_series(raw_data);

        // Verify results
        assert_eq!(adjusted_data.len(), 2);
        assert_eq!(adjusted_data[0].close_price, 200.0); // Before split
        assert_eq!(adjusted_data[1].close_price, 100.0); // After split (adjusted)
    }

    /// Test end-to-end workflow: Create multiple futures -> Generate rolling series
    #[test]
    fn test_futures_rolling_series_end_to_end() {
        // Create two consecutive contracts
        let expiry1 = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let contract1 = Future::new(
            "ES",
            expiry1,
            "2024-12",
            "E-mini S&P 500 Dec 2024",
            "CME",
            "USD",
            "CME",
            5,
        )
        .unwrap();

        let expiry2 = NaiveDate::from_ymd_opt(2025, 3, 20).unwrap();
        let contract2 = Future::new(
            "ES",
            expiry2,
            "2025-03",
            "E-mini S&P 500 Mar 2025",
            "CME",
            "USD",
            "CME",
            5,
        )
        .unwrap();

        // Set up data provider
        let mut provider = InMemoryDataProvider::new();

        // Add data for contract1 (before rollover)
        let points1 = vec![
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 12, 10, 16, 0, 0).unwrap(),
                4500.0,
            ),
            TimeSeriesPoint::new(
                Utc.with_ymd_and_hms(2024, 12, 15, 16, 0, 0).unwrap(), // Rollover date
                4510.0,
            ),
        ];
        provider.add_data(contract1.key().clone(), points1);

        // Add data for contract2 (after rollover)
        let points2 = vec![TimeSeriesPoint::new(
            Utc.with_ymd_and_hms(2024, 12, 16, 16, 0, 0).unwrap(),
            4520.0,
        )];
        provider.add_data(contract2.key().clone(), points2);

        // Generate rolling series
        let contracts = vec![&contract1, &contract2];
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 12, 10).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 16).unwrap(),
        );

        let rolling_series =
            Future::generate_rolling_price_series(&provider, &contracts, &date_range, 5).unwrap();

        // Verify rolling series contains data from both contracts
        assert!(rolling_series.len() >= 2);
    }

    /// Test Asset trait polymorphism - using different asset types through common interface
    #[test]
    fn test_asset_trait_polymorphism() {
        let equity = Equity::new("AAPL", "Apple Inc.", "NASDAQ", "USD", "Technology").unwrap();

        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let future = Future::new(
            "ES",
            expiry,
            "2024-12",
            "E-mini S&P 500",
            "CME",
            "USD",
            "CME",
            5,
        )
        .unwrap();

        // Use Asset trait methods on both types
        let assets: Vec<&dyn Asset> = vec![&equity, &future];

        assert_eq!(assets[0].asset_type(), AssetType::Equity);
        assert_eq!(assets[1].asset_type(), AssetType::Future);

        // Verify keys are accessible through trait
        assert!(matches!(assets[0].key(), AssetKey::Equity(_)));
        assert!(matches!(assets[1].key(), AssetKey::Future { .. }));
    }

    /// Test asset key equality and hashing in collections
    #[test]
    fn test_asset_key_collection_usage() {
        use std::collections::HashMap;

        let key1 = AssetKey::new_equity("AAPL").unwrap();
        let key2 = AssetKey::new_equity("AAPL").unwrap();
        let key3 = AssetKey::new_equity("MSFT").unwrap();

        // Test equality
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);

        // Test hashing in HashMap
        let mut map = HashMap::new();
        map.insert(key1.clone(), "Apple Inc.");
        assert_eq!(map.get(&key2), Some(&"Apple Inc."));
        assert_eq!(map.get(&key3), None);
    }

    /// Test DateRange edge cases
    #[test]
    fn test_date_range_edge_cases() {
        let single_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let single_day_range = DateRange::new(single_date, single_date);

        let mut provider = InMemoryDataProvider::new();
        let asset_key = AssetKey::new_equity("AAPL").unwrap();
        let points = vec![TimeSeriesPoint::new(
            Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap(),
            150.0,
        )];
        provider.add_data(asset_key.clone(), points);

        let result = provider
            .get_time_series(&asset_key, &single_day_range)
            .unwrap();
        assert_eq!(result.len(), 1);
    }

    /// Test corporate actions with multiple splits over time
    #[test]
    fn test_multiple_corporate_actions() {
        let split1 = CorporateAction::Split {
            ratio: 2.0,
            effective_date: NaiveDate::from_ymd_opt(2020, 8, 31).unwrap(),
        };
        let split2 = CorporateAction::Split {
            ratio: 3.0,
            effective_date: NaiveDate::from_ymd_opt(2024, 6, 9).unwrap(),
        };

        let equity = Equity::with_corporate_actions(
            "AAPL",
            "Apple Inc.",
            "NASDAQ",
            "USD",
            "Technology",
            vec![split1, split2],
        )
        .unwrap();

        // Price before first split
        let price1 =
            equity.apply_corporate_actions(600.0, NaiveDate::from_ymd_opt(2020, 8, 30).unwrap());
        assert_eq!(price1, 600.0);

        // Price after first split, before second
        let price2 =
            equity.apply_corporate_actions(300.0, NaiveDate::from_ymd_opt(2024, 6, 8).unwrap());
        assert_eq!(price2, 150.0); // 300 / 2

        // Price after both splits
        let price3 =
            equity.apply_corporate_actions(300.0, NaiveDate::from_ymd_opt(2024, 6, 10).unwrap());
        assert_eq!(price3, 50.0); // 300 / 2 / 3
    }

    /// Test rolling futures with missing contract data
    #[test]
    fn test_rolling_futures_missing_data() {
        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let contract = Future::new(
            "ES",
            expiry,
            "2024-12",
            "E-mini S&P 500",
            "CME",
            "USD",
            "CME",
            5,
        )
        .unwrap();

        let provider = InMemoryDataProvider::new(); // Empty provider
        let contracts = vec![&contract];
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 12, 10).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 15).unwrap(),
        );

        // Should handle missing data gracefully
        let result = Future::generate_rolling_price_series(&provider, &contracts, &date_range, 5);
        // Result may be empty but should not panic
        assert!(result.is_ok());
    }

    /// Test asset metadata access patterns
    #[test]
    fn test_asset_metadata_access_patterns() {
        let equity = Equity::new("AAPL", "Apple Inc.", "NASDAQ", "USD", "Technology").unwrap();

        // Verify all metadata fields are accessible
        assert_eq!(equity.name(), "Apple Inc.");
        assert_eq!(equity.exchange(), "NASDAQ");
        assert_eq!(equity.currency(), "USD");
        assert_eq!(equity.sector(), "Technology");

        let expiry = NaiveDate::from_ymd_opt(2024, 12, 20).unwrap();
        let future = Future::new(
            "ES",
            expiry,
            "2024-12",
            "E-mini S&P 500",
            "CME",
            "USD",
            "CME",
            5,
        )
        .unwrap();

        assert_eq!(future.name(), "E-mini S&P 500");
        assert_eq!(future.exchange(), "CME");
        assert_eq!(future.currency(), "USD");
    }

    /// Test integration: Asset + DataProvider + TimeSeries complete workflow
    #[test]
    fn test_complete_asset_data_workflow() {
        // Create asset
        let equity = Equity::new(
            "MSFT",
            "Microsoft Corporation",
            "NASDAQ",
            "USD",
            "Technology",
        )
        .unwrap();

        // Set up data provider
        let mut provider = InMemoryDataProvider::new();
        let points = vec![
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 15, 16, 0, 0).unwrap(), 400.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 16, 16, 0, 0).unwrap(), 401.0),
            TimeSeriesPoint::new(Utc.with_ymd_and_hms(2024, 1, 17, 16, 0, 0).unwrap(), 402.0),
        ];
        provider.add_data(equity.key().clone(), points);

        // Query data
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 17).unwrap(),
        );
        let data = equity.get_time_series(&provider, &date_range).unwrap();

        // Verify complete workflow
        assert_eq!(data.len(), 3);
        assert_eq!(data[0].close_price, 400.0);
        assert_eq!(data[1].close_price, 401.0);
        assert_eq!(data[2].close_price, 402.0);
    }

    /// Test asset key format with special characters (valid ones)
    #[test]
    fn test_asset_key_special_characters() {
        // Test valid special characters (dots, hyphens, underscores)
        let key1 = AssetKey::new_equity("BRK.B").unwrap(); // Dot
        let key2 = AssetKey::new_equity("BRK-B").unwrap(); // Hyphen
        let key3 = AssetKey::new_equity("BRK_B").unwrap(); // Underscore

        assert!(matches!(key1, AssetKey::Equity(_)));
        assert!(matches!(key2, AssetKey::Equity(_)));
        assert!(matches!(key3, AssetKey::Equity(_)));
    }
}
