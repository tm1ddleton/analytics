//! Analytics API Server Binary
//!
//! Run with: `cargo run --bin analytics-server`

use analytics::{run_server, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Note: Tracing is initialized in run_server()
    // Set RUST_LOG environment variable to control log level:
    //   RUST_LOG=debug cargo run --bin analytics-server
    //   RUST_LOG=analytics::dag=debug cargo run --bin analytics-server  (DAG only)

    // Create configuration from environment variables or defaults
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    let database_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "analytics.db".to_string());
    
    let config = ServerConfig::new(host, port, database_path);

    println!("🚀 Starting Analytics API Server...");
    println!("   Host: {}", config.host);
    println!("   Port: {}", config.port);
    println!("   Database: {}", config.database_path);
    println!();
    println!(
        "Server will be available at: http://{}:{}",
        config.host, config.port
    );
    println!();
    println!("Available endpoints:");
    println!("  GET  /health                    - Health check");
    println!("  GET  /assets                    - List assets");
    println!("  GET  /assets/:asset/data        - Get price data");
    println!("  GET  /analytics/:asset/:type    - Pull-mode analytics");
    println!("  POST /analytics/batch           - Batch analytics");
    println!("  GET  /dag/nodes                 - List analytics types");
    println!("  POST /replay                    - Create replay session");
    println!("  GET  /replay/:id                - Session status");
    println!("  DELETE /replay/:id              - Stop session");
    println!("  GET  /stream/:id                - SSE stream");
    println!();

    // Run server
    run_server(config).await?;

    Ok(())
}
