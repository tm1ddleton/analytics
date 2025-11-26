//! REST API server with Server-Sent Events support

mod error;
mod handlers;
mod routes;
mod state;

pub use error::ApiError;
pub use state::{AnalyticConfig, AppState, SessionStatus};

use crate::sqlite_provider::SqliteDataProvider;
use std::sync::Arc;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server host address (default: "127.0.0.1")
    pub host: String,
    /// Server port (default: 3000)
    pub port: u16,
    /// Path to SQLite database
    pub database_path: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            database_path: "analytics.db".to_string(),
        }
    }
}

impl ServerConfig {
    /// Creates a new server configuration
    pub fn new(host: impl Into<String>, port: u16, database_path: impl Into<String>) -> Self {
        ServerConfig {
            host: host.into(),
            port,
            database_path: database_path.into(),
        }
    }
}

/// Runs the API server
///
/// # Arguments
/// * `config` - Server configuration
///
/// # Returns
/// Returns an error if the server fails to start or encounters a fatal error
///
/// # Example
/// ```rust,no_run
/// use analytics::server::{run_server, ServerConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = ServerConfig::default();
///     run_server(config).await?;
///     Ok(())
/// }
/// ```
pub async fn run_server(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Create data provider
    let data_provider = SqliteDataProvider::new(&config.database_path)?;

    // Create application state
    let state = Arc::new(AppState::new(data_provider));

    // Create router
    let app = routes::create_router(state);

    // Build server address
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Server listening on http://{}", addr);

    // Run server
    axum::serve(listener, app).await?;

    Ok(())
}
