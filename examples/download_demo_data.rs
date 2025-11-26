use analytics::{
    AssetKey, DataProvider, DateRange, DownloaderConfig, SqliteDataProvider, YahooFinanceDownloader,
};
use chrono::NaiveDate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("📥 Downloading demo data...");
    println!();

    // Configure downloader
    let config = DownloaderConfig {
        max_retries: 3,
        timeout_seconds: 30,
        requests_per_second: 2.0,
    };

    let downloader = YahooFinanceDownloader::with_config(config)?;
    let mut provider = SqliteDataProvider::new("analytics.db")?;

    // Assets to download
    let tickers = vec!["AAPL", "MSFT", "GOOG"];

    println!("Assets: {}", tickers.join(", "));
    println!("Date range: 2024-01-01 to 2024-12-31");
    println!();

    // Build asset list with date ranges
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
    let date_range = DateRange::new(start_date, end_date);

    let assets: Vec<(AssetKey, DateRange)> = tickers
        .iter()
        .map(|ticker| (AssetKey::new_equity(*ticker).unwrap(), date_range.clone()))
        .collect();

    // Download data
    let result = downloader
        .download_multiple_to_sqlite(&mut provider, &assets)
        .await;

    println!("✅ Download complete!");
    println!();
    println!("Results:");
    println!("  ✓ Successful: {}", result.successful.len());
    println!("  ✗ Failed: {}", result.failed.len());
    println!();

    // Show successful downloads
    if !result.successful.is_empty() {
        println!("Successfully downloaded:");
        for (ticker, count) in &result.successful {
            println!("  • {}: {} data points", ticker, count);
        }
        println!();
    }

    // Show failures
    if !result.failed.is_empty() {
        println!("Failed downloads:");
        for (ticker, error) in &result.failed {
            println!("  ✗ {}: {}", ticker, error);
        }
        println!();
    }

    // Verify data by checking the database
    println!("🔍 Verifying database...");

    for ticker in result.successful.keys() {
        let asset_key = AssetKey::new_equity(ticker.as_str())?;

        // Get some sample data
        let sample_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        );

        let data = provider.get_time_series(&asset_key, &sample_range)?;

        println!(
            "  • {}: {} data points (sample from Jan 2024)",
            ticker,
            data.len()
        );
    }
    println!();

    println!("✨ Demo data is ready!");
    println!();
    println!("You can now:");
    println!("  • Start the demo: ./run-demo.sh");
    println!("  • Query the API: curl 'http://localhost:3000/analytics/AAPL/returns?start=2024-01-01&end=2024-12-31'");
    println!("  • Use the dashboard: http://localhost:5173");

    Ok(())
}
