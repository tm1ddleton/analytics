//! Analytics API Server Binary
//!
//! Run with: `cargo run --bin analytics-server`

use analytics::{run_server, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create default configuration
    // In production, this would load from config file or environment variables
    let config = ServerConfig::default();

    println!("🚀 Starting Analytics API Server...");
    println!("   Host: {}", config.host);
    println!("   Port: {}", config.port);
    println!("   Database: {}", config.database_path);
    println!();
    println!("Server will be available at: http://{}:{}", config.host, config.port);
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

