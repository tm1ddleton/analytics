use analytics::{AssetKey, SqliteDataProvider, TimeSeriesPoint};
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use rusqlite::Connection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Creating demo data...");
    println!();

    // Open database
    let conn = Connection::open("analytics.db")?;

    // Create tables if they don't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS assets (
            asset_key TEXT PRIMARY KEY,
            asset_data TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS time_series_data (
            asset_key TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            close_price REAL NOT NULL,
            PRIMARY KEY (asset_key, timestamp)
        )",
        [],
    )?;

    println!("✓ Database tables ready");
    println!();

    // Generate synthetic data for three tickers
    let tickers = vec!["AAPL", "MSFT", "GOOG"];
    let base_prices = vec![150.0, 300.0, 120.0];

    for (ticker, base_price) in tickers.iter().zip(base_prices.iter()) {
        println!("Generating data for {}...", ticker);

        // Insert asset (with minimal asset_data as JSON)
        let asset_data = format!(r#"{{"ticker": "{}"}}"#, ticker);
        conn.execute(
            "INSERT OR REPLACE INTO assets (asset_key, asset_data) VALUES (?1, ?2)",
            rusqlite::params![ticker, asset_data],
        )?;

        // Generate daily data for 2024
        let mut price = *base_price;
        let mut date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let mut count = 0;

        while date <= end_date {
            // Skip weekends (very simple, doesn't handle holidays)
            if date.weekday().number_from_monday() <= 5 {
                // Simulate some price movement
                let change_pct = (rand::random::<f64>() - 0.5) * 0.04; // +/- 2% daily
                price = price * (1.0 + change_pct);

                let close = price;

                let datetime = Utc.from_utc_datetime(&date.and_hms_opt(16, 0, 0).unwrap());
                let timestamp_str = datetime.to_rfc3339();

                conn.execute(
                    "INSERT OR REPLACE INTO time_series_data 
                     (asset_key, timestamp, close_price)
                     VALUES (?1, ?2, ?3)",
                    rusqlite::params![ticker, timestamp_str, close],
                )?;
                count += 1;
            }

            date = date.succ_opt().unwrap();
        }

        println!("  ✓ {}: {} data points", ticker, count);
    }

    println!();
    println!("✨ Demo data created successfully!");
    println!();
    println!("You can now:");
    println!("  • Start the demo: ./run-demo.sh");
    println!("  • Query the API: curl 'http://localhost:3000/analytics/AAPL/returns?start=2024-01-01&end=2024-12-31'");
    println!("  • Use the dashboard: http://localhost:5173");

    Ok(())
}

// Simple pseudo-random number generator
mod rand {
    use std::cell::Cell;
    use std::time::{SystemTime, UNIX_EPOCH};

    thread_local! {
        static SEED: Cell<u64> = Cell::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64
        );
    }

    pub fn random<T: FromRandom>() -> T {
        T::from_random()
    }

    pub trait FromRandom {
        fn from_random() -> Self;
    }

    impl FromRandom for f64 {
        fn from_random() -> Self {
            SEED.with(|seed| {
                let mut s = seed.get();
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                seed.set(s);
                (s as f64) / (u64::MAX as f64)
            })
        }
    }
}
