//! Route definitions for the API server

use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use super::handlers;
use super::state::AppState;

/// Creates the main application router with all routes and middleware
pub fn create_router(state: Arc<AppState>) -> Router {
    // Create CORS layer (allow all origins for POC)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router with routes
    Router::new()
        // Health check
        .route("/health", get(handlers::health_check))
        // Static information endpoints
        .route("/assets", get(handlers::list_assets))
        .route("/dag/nodes", get(handlers::list_analytics))
        // Asset data query
        .route("/assets/:asset/data", get(handlers::get_asset_data))
        // Pull-mode analytics
        .route("/analytics/:asset/:type", get(handlers::get_analytics))
        .route("/analytics/batch", post(handlers::batch_analytics))
        // Replay session management
        .route("/replay", post(handlers::create_replay_session))
        .route("/replay/:session_id", get(handlers::get_session_status))
        .route("/replay/:session_id", delete(handlers::stop_replay_session))
        // SSE streaming
        .route("/stream/:session_id", get(handlers::handle_stream))
        // Add middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        // Add shared state
        .with_state(state)
}

