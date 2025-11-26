//! Shared application state for the API server

use crate::asset_key::AssetKey;
use crate::dag::AnalyticsDag;
use crate::push_mode::PushModeEngine;
use crate::replay::ReplayEngine;
use crate::sqlite_provider::SqliteDataProvider;
use axum::response::sse::Event;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc::Sender, Mutex, RwLock};
use uuid::Uuid;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// SQLite data provider for querying historical data
    /// Wrapped in Mutex because SQLite connections are not thread-safe
    pub data_provider: Arc<Mutex<SqliteDataProvider>>,
    /// Active replay sessions
    pub sessions: Arc<RwLock<HashMap<Uuid, ReplaySession>>>,
    /// SSE broadcasters for each session
    pub broadcasters: Arc<RwLock<HashMap<Uuid, Sender<Event>>>>,
}

impl AppState {
    /// Creates a new application state
    pub fn new(data_provider: SqliteDataProvider) -> Self {
        AppState {
            data_provider: Arc::new(Mutex::new(data_provider)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            broadcasters: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Replay session state
///
/// Note: The actual DAG, push-mode engine, and replay engine are managed separately
/// due to thread-safety requirements. This structure contains only the session metadata.
pub struct ReplaySession {
    /// Unique session identifier
    pub id: Uuid,
    /// Assets being replayed
    pub assets: Vec<AssetKey>,
    /// Analytics configurations
    pub analytics: Vec<AnalyticConfig>,
    /// Start date of replay
    pub start_date: NaiveDate,
    /// End date of replay
    pub end_date: NaiveDate,
    /// Current session status
    pub status: SessionStatus,
    /// When session was created
    pub created_at: DateTime<Utc>,
    /// When replay started
    pub started_at: Option<DateTime<Utc>>,
    /// Current date being replayed
    pub current_date: Option<NaiveDate>,
    /// Progress (0.0 to 1.0)
    pub progress: f64,
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    /// Session created but not started
    Created,
    /// Replay in progress
    Running,
    /// Replay completed successfully
    Completed,
    /// Replay stopped by user
    Stopped,
    /// Replay failed with error
    Error,
}

/// Analytics configuration for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticConfig {
    /// Type of analytic (e.g., "returns", "volatility")
    #[serde(rename = "type")]
    pub analytic_type: String,
    /// Parameters for the analytic
    #[serde(default)]
    pub parameters: HashMap<String, String>,
}
