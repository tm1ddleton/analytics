//! HTTP request handlers for API endpoints

use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::stream::Stream;
use std::convert::Infallible;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use super::error::ApiError;
use super::state::{AppState, AnalyticConfig, ReplaySession, SessionStatus};
use crate::asset_key::AssetKey;
use crate::dag::{AnalyticsDag, NodeId, NodeParams};
use crate::time_series::DateRange;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Health check endpoint
/// 
/// Returns a simple status response to verify the server is running
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok"
    }))
}

/// Response for asset listing
#[derive(Debug, Serialize)]
pub struct AssetsResponse {
    pub assets: Vec<AssetInfo>,
}

/// Information about a single asset
#[derive(Debug, Serialize)]
pub struct AssetInfo {
    pub key: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub name: String,
    pub data_available_from: Option<String>,
    pub data_available_to: Option<String>,
}

/// GET /assets - List all available assets
pub async fn list_assets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AssetsResponse>, ApiError> {
    // For POC, we'll return a simple list
    // In production, this would query the database for actual assets
    let assets = vec![
        AssetInfo {
            key: "AAPL".to_string(),
            asset_type: "equity".to_string(),
            name: "Apple Inc.".to_string(),
            data_available_from: Some("2020-01-01".to_string()),
            data_available_to: Some("2024-12-31".to_string()),
        },
        AssetInfo {
            key: "MSFT".to_string(),
            asset_type: "equity".to_string(),
            name: "Microsoft Corporation".to_string(),
            data_available_from: Some("2020-01-01".to_string()),
            data_available_to: Some("2024-12-31".to_string()),
        },
        AssetInfo {
            key: "GOOG".to_string(),
            asset_type: "equity".to_string(),
            name: "Alphabet Inc.".to_string(),
            data_available_from: Some("2020-01-01".to_string()),
            data_available_to: Some("2024-12-31".to_string()),
        },
    ];

    Ok(Json(AssetsResponse { assets }))
}

/// Response for analytics listing
#[derive(Debug, Serialize)]
pub struct AnalyticsListResponse {
    pub analytics: Vec<AnalyticInfo>,
}

/// Information about an analytic type
#[derive(Debug, Serialize)]
pub struct AnalyticInfo {
    #[serde(rename = "type")]
    pub analytic_type: String,
    pub description: String,
    pub parameters: Vec<ParameterInfo>,
    pub burnin_days: String,
}

/// Information about an analytic parameter
#[derive(Debug, Serialize)]
pub struct ParameterInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
    pub default: Option<String>,
    pub description: String,
}

/// GET /dag/nodes - List available analytic types
pub async fn list_analytics() -> Json<AnalyticsListResponse> {
    let analytics = vec![
        AnalyticInfo {
            analytic_type: "returns".to_string(),
            description: "Log returns calculation".to_string(),
            parameters: vec![],
            burnin_days: "1".to_string(),
        },
        AnalyticInfo {
            analytic_type: "volatility".to_string(),
            description: "Rolling volatility (population std dev)".to_string(),
            parameters: vec![ParameterInfo {
                name: "window".to_string(),
                param_type: "integer".to_string(),
                required: false,
                default: Some("10".to_string()),
                description: "Rolling window size in days".to_string(),
            }],
            burnin_days: "window + 1".to_string(),
        },
    ];

    Json(AnalyticsListResponse { analytics })
}

// Task Group 4: Asset Data Query

/// Query parameters for asset data endpoint
#[derive(Debug, Deserialize)]
pub struct DataQueryParams {
    pub start: String,
    pub end: String,
}

/// Single data point in response
#[derive(Debug, Serialize)]
pub struct DataPoint {
    pub timestamp: String,
    pub close: f64,
}

/// Response for asset data query
#[derive(Debug, Serialize)]
pub struct AssetDataResponse {
    pub asset: String,
    pub start_date: String,
    pub end_date: String,
    pub data: Vec<DataPoint>,
}

/// GET /assets/{asset}/data - Get raw price data for an asset
pub async fn get_asset_data(
    State(state): State<Arc<AppState>>,
    Path(asset): Path<String>,
    Query(params): Query<DataQueryParams>,
) -> Result<Json<AssetDataResponse>, ApiError> {
    // Parse dates
    let start_date = NaiveDate::parse_from_str(&params.start, "%Y-%m-%d")
        .map_err(|e| ApiError::InvalidDateRange(format!("Invalid start date: {}", e)))?;
    let end_date = NaiveDate::parse_from_str(&params.end, "%Y-%m-%d")
        .map_err(|e| ApiError::InvalidDateRange(format!("Invalid end date: {}", e)))?;

    // Validate date range
    if start_date > end_date {
        return Err(ApiError::InvalidDateRange(
            "Start date must be before or equal to end date".to_string(),
        ));
    }

    // Create asset key
    let asset_key = AssetKey::new_equity(&asset)
        .map_err(|e| ApiError::InvalidParameter(format!("Invalid asset: {}", e)))?;

    // Create date range
    let date_range = DateRange::new(start_date, end_date);

    // Query data provider
    let provider = state.data_provider.lock().await;
    let data_points = crate::time_series::DataProvider::get_time_series(&*provider, &asset_key, &date_range)
        .map_err(|e| match e {
            crate::time_series::DataProviderError::AssetNotFound => {
                ApiError::AssetNotFound(asset.clone())
            }
            _ => ApiError::InternalError(e.to_string()),
        })?;

    // Convert to response format
    let data: Vec<DataPoint> = data_points
        .iter()
        .map(|point| DataPoint {
            timestamp: point.timestamp.to_rfc3339(),
            close: point.close_price,
        })
        .collect();

    Ok(Json(AssetDataResponse {
        asset,
        start_date: params.start,
        end_date: params.end,
        data,
    }))
}

// Task Group 5: Pull-Mode Analytics Endpoints

/// Helper function to build analytics DAG
fn build_analytics_dag(
    asset: &AssetKey,
    analytic_type: &str,
    params: &HashMap<String, String>,
) -> Result<(AnalyticsDag, NodeId), ApiError> {
    let mut dag = AnalyticsDag::new();

    let data_node = dag.add_node(
        "DataProvider".to_string(),
        NodeParams::None,
        vec![asset.clone()],
    );

    match analytic_type {
        "returns" => {
            let returns_node = dag.add_node(
                "Returns".to_string(),
                NodeParams::None,
                vec![asset.clone()],
            );
            dag.add_edge(data_node, returns_node)
                .map_err(|e| ApiError::InternalError(e.to_string()))?;
            Ok((dag, returns_node))
        }
        "volatility" => {
            let window = params
                .get("window")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(10);

            if window == 0 {
                return Err(ApiError::InvalidParameter(
                    "Window size must be greater than 0".to_string(),
                ));
            }

            // Build: DataProvider -> Returns -> Volatility
            let returns_node = dag.add_node(
                "Returns".to_string(),
                NodeParams::None,
                vec![asset.clone()],
            );
            
            let mut vol_params_map = HashMap::new();
            vol_params_map.insert("window_size".to_string(), window.to_string());
            
            let vol_node = dag.add_node(
                "Volatility".to_string(),
                NodeParams::Map(vol_params_map),
                vec![asset.clone()],
            );

            dag.add_edge(data_node, returns_node)
                .map_err(|e| ApiError::InternalError(e.to_string()))?;
            dag.add_edge(returns_node, vol_node)
                .map_err(|e| ApiError::InternalError(e.to_string()))?;
            
            Ok((dag, vol_node))
        }
        _ => Err(ApiError::InvalidParameter(format!(
            "Unknown analytic type: {}",
            analytic_type
        ))),
    }
}

/// Query parameters for analytics endpoint
#[derive(Debug, Deserialize)]
pub struct AnalyticsQueryParams {
    pub start: String,
    pub end: String,
    pub window: Option<usize>,
}

/// Single data point in analytics response
#[derive(Debug, Serialize)]
pub struct AnalyticDataPoint {
    pub timestamp: String,
    pub value: Option<f64>,
}

/// Response for analytics query
#[derive(Debug, Serialize)]
pub struct AnalyticsResponse {
    pub asset: String,
    pub analytic: String,
    pub parameters: HashMap<String, String>,
    pub start_date: String,
    pub end_date: String,
    pub data: Vec<AnalyticDataPoint>,
}

/// GET /analytics/{asset}/{type} - Execute pull-mode analytics query
pub async fn get_analytics(
    State(state): State<Arc<AppState>>,
    Path((asset, analytic_type)): Path<(String, String)>,
    Query(query_params): Query<AnalyticsQueryParams>,
) -> Result<Json<AnalyticsResponse>, ApiError> {
    // Parse dates
    let start_date = NaiveDate::parse_from_str(&query_params.start, "%Y-%m-%d")
        .map_err(|e| ApiError::InvalidDateRange(format!("Invalid start date: {}", e)))?;
    let end_date = NaiveDate::parse_from_str(&query_params.end, "%Y-%m-%d")
        .map_err(|e| ApiError::InvalidDateRange(format!("Invalid end date: {}", e)))?;

    // Validate date range
    if start_date > end_date {
        return Err(ApiError::InvalidDateRange(
            "Start date must be before or equal to end date".to_string(),
        ));
    }

    // Create asset key
    let asset_key = AssetKey::new_equity(&asset)
        .map_err(|e| ApiError::InvalidParameter(format!("Invalid asset: {}", e)))?;

    // Build parameters map
    let mut params = HashMap::new();
    if let Some(window) = query_params.window {
        params.insert("window".to_string(), window.to_string());
    }

    // Build DAG
    let (dag, target_node) = build_analytics_dag(&asset_key, &analytic_type, &params)?;

    // Create date range
    let date_range = DateRange::new(start_date, end_date);

    // Execute pull-mode query
    let provider = state.data_provider.lock().await;
    let result = dag
        .execute_pull_mode(target_node, date_range, &*provider)
        .map_err(|e| ApiError::ComputationFailed(e.to_string()))?;

    // Convert to response format
    let data: Vec<AnalyticDataPoint> = result
        .iter()
        .map(|point| AnalyticDataPoint {
            timestamp: point.timestamp.to_rfc3339(),
            value: if point.close_price.is_nan() {
                None
            } else {
                Some(point.close_price)
            },
        })
        .collect();

    Ok(Json(AnalyticsResponse {
        asset,
        analytic: analytic_type,
        parameters: params,
        start_date: query_params.start,
        end_date: query_params.end,
        data,
    }))
}

/// Request for batch analytics query
#[derive(Debug, Deserialize)]
pub struct BatchQueryRequest {
    pub queries: Vec<BatchQuery>,
}

/// Single query in a batch request
#[derive(Debug, Clone, Deserialize)]
pub struct BatchQuery {
    pub asset: String,
    pub analytic: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub parameters: HashMap<String, String>,
}

/// Response for batch query
#[derive(Debug, Serialize)]
pub struct BatchQueryResponse {
    pub results: Vec<AnalyticsResponse>,
    pub errors: Vec<BatchError>,
}

/// Error for a single query in a batch
#[derive(Debug, Serialize)]
pub struct BatchError {
    pub asset: String,
    pub analytic: String,
    pub error: String,
}

/// POST /analytics/batch - Execute multiple analytics queries
pub async fn batch_analytics(
    State(state): State<Arc<AppState>>,
    Json(request): Json<BatchQueryRequest>,
) -> Result<Json<BatchQueryResponse>, ApiError> {
    let mut results = Vec::new();
    let mut errors = Vec::new();

    for query in request.queries {
        // Execute each query
        match execute_single_batch_query(&state, query.clone()).await {
            Ok(response) => results.push(response),
            Err(e) => errors.push(BatchError {
                asset: query.asset.clone(),
                analytic: query.analytic.clone(),
                error: e.to_string(),
            }),
        }
    }

    Ok(Json(BatchQueryResponse { results, errors }))
}

/// Helper to execute a single query in a batch
async fn execute_single_batch_query(
    state: &AppState,
    query: BatchQuery,
) -> Result<AnalyticsResponse, ApiError> {
    // Parse dates
    let start_date = NaiveDate::parse_from_str(&query.start_date, "%Y-%m-%d")
        .map_err(|e| ApiError::InvalidDateRange(format!("Invalid start date: {}", e)))?;
    let end_date = NaiveDate::parse_from_str(&query.end_date, "%Y-%m-%d")
        .map_err(|e| ApiError::InvalidDateRange(format!("Invalid end date: {}", e)))?;

    // Create asset key
    let asset_key = AssetKey::new_equity(&query.asset)
        .map_err(|e| ApiError::InvalidParameter(format!("Invalid asset: {}", e)))?;

    // Build DAG
    let (dag, target_node) = build_analytics_dag(&asset_key, &query.analytic, &query.parameters)?;

    // Create date range
    let date_range = DateRange::new(start_date, end_date);

    // Execute pull-mode query
    let provider = state.data_provider.lock().await;
    let result = dag
        .execute_pull_mode(target_node, date_range, &*provider)
        .map_err(|e| ApiError::ComputationFailed(e.to_string()))?;

    // Convert to response format
    let data: Vec<AnalyticDataPoint> = result
        .iter()
        .map(|point| AnalyticDataPoint {
            timestamp: point.timestamp.to_rfc3339(),
            value: if point.close_price.is_nan() {
                None
            } else {
                Some(point.close_price)
            },
        })
        .collect();

    Ok(AnalyticsResponse {
        asset: query.asset,
        analytic: query.analytic,
        parameters: query.parameters,
        start_date: query.start_date,
        end_date: query.end_date,
        data,
    })
}

// Task Group 6: Replay Session Management

/// Request to create a replay session
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub assets: Vec<String>,
    pub analytics: Vec<AnalyticConfig>,
    pub start_date: String,
    pub end_date: String,
}

/// Response for session creation
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub status: SessionStatus,
    pub assets: Vec<String>,
    pub analytics: Vec<String>,
    pub start_date: String,
    pub end_date: String,
    pub stream_url: String,
}

/// Response for session status query
#[derive(Debug, Serialize)]
pub struct SessionStatusResponse {
    pub session_id: String,
    pub status: SessionStatus,
    pub assets: Vec<String>,
    pub analytics: Vec<String>,
    pub start_date: String,
    pub end_date: String,
    pub current_date: Option<String>,
    pub progress: f64,
    pub created_at: String,
    pub started_at: Option<String>,
    pub stream_url: String,
}

/// POST /replay - Create a new replay session
pub async fn create_replay_session(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<Json<SessionResponse>, ApiError> {
    // Check session limit
    let sessions = state.sessions.read().await;
    if sessions.len() >= 10 {
        return Err(ApiError::SessionLimitReached);
    }
    drop(sessions);

    // Parse dates
    let start_date = NaiveDate::parse_from_str(&request.start_date, "%Y-%m-%d")
        .map_err(|e| ApiError::InvalidDateRange(format!("Invalid start date: {}", e)))?;
    let end_date = NaiveDate::parse_from_str(&request.end_date, "%Y-%m-%d")
        .map_err(|e| ApiError::InvalidDateRange(format!("Invalid end date: {}", e)))?;

    // Validate assets exist
    let asset_keys: Result<Vec<AssetKey>, _> = request
        .assets
        .iter()
        .map(|a| {
            AssetKey::new_equity(a).map_err(|e| ApiError::InvalidParameter(format!("Invalid asset {}: {}", a, e)))
        })
        .collect();
    let asset_keys = asset_keys?;

    // Generate session ID
    let session_id = Uuid::new_v4();

    // Create session
    let session = ReplaySession {
        id: session_id,
        assets: asset_keys,
        analytics: request.analytics.clone(),
        start_date,
        end_date,
        status: SessionStatus::Created,
        created_at: Utc::now(),
        started_at: None,
        current_date: None,
        progress: 0.0,
    };

    // Store session
    let mut sessions = state.sessions.write().await;
    sessions.insert(session_id, session);
    drop(sessions);

    // Extract analytic types for response
    let analytic_types: Vec<String> = request
        .analytics
        .iter()
        .map(|a| a.analytic_type.clone())
        .collect();

    Ok(Json(SessionResponse {
        session_id: session_id.to_string(),
        status: SessionStatus::Created,
        assets: request.assets,
        analytics: analytic_types,
        start_date: request.start_date,
        end_date: request.end_date,
        stream_url: format!("/stream/{}", session_id),
    }))
}

/// GET /replay/{session_id} - Get session status
pub async fn get_session_status(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionStatusResponse>, ApiError> {
    let session_id = Uuid::parse_str(&session_id)
        .map_err(|_| ApiError::InvalidParameter("Invalid session ID".to_string()))?;

    let sessions = state.sessions.read().await;
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| ApiError::SessionNotFound(session_id))?;

    let asset_strings: Vec<String> = session
        .assets
        .iter()
        .map(|a| a.to_string())
        .collect();

    let analytic_types: Vec<String> = session
        .analytics
        .iter()
        .map(|a| a.analytic_type.clone())
        .collect();

    Ok(Json(SessionStatusResponse {
        session_id: session_id.to_string(),
        status: session.status,
        assets: asset_strings,
        analytics: analytic_types,
        start_date: session.start_date.to_string(),
        end_date: session.end_date.to_string(),
        current_date: session.current_date.map(|d| d.to_string()),
        progress: session.progress,
        created_at: session.created_at.to_rfc3339(),
        started_at: session.started_at.map(|dt| dt.to_rfc3339()),
        stream_url: format!("/stream/{}", session_id),
    }))
}

/// Response for session deletion
#[derive(Debug, Serialize)]
pub struct DeleteSessionResponse {
    pub session_id: String,
    pub status: String,
    pub message: String,
}

/// DELETE /replay/{session_id} - Stop a replay session
pub async fn stop_replay_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Result<Json<DeleteSessionResponse>, ApiError> {
    let session_id = Uuid::parse_str(&session_id)
        .map_err(|_| ApiError::InvalidParameter("Invalid session ID".to_string()))?;

    let mut sessions = state.sessions.write().await;
    let session = sessions
        .get_mut(&session_id)
        .ok_or_else(|| ApiError::SessionNotFound(session_id))?;

    // Check if already completed or stopped
    if session.status == SessionStatus::Completed || session.status == SessionStatus::Stopped {
        return Err(ApiError::InvalidParameter(format!(
            "Session already {:?}",
            session.status
        )));
    }

    // Update status to stopped
    session.status = SessionStatus::Stopped;

    Ok(Json(DeleteSessionResponse {
        session_id: session_id.to_string(),
        status: "stopped".to_string(),
        message: "Replay session stopped".to_string(),
    }))
}

// Task Group 7: Server-Sent Events Streaming (Simplified)

/// GET /stream/{session_id} - SSE stream for replay updates
/// 
/// This is a simplified implementation that establishes the SSE connection.
/// Full replay integration would happen in Task Group 8.
pub async fn handle_stream(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let session_id = Uuid::parse_str(&session_id)
        .map_err(|_| ApiError::InvalidParameter("Invalid session ID".to_string()))?;

    // Verify session exists
    let sessions = state.sessions.read().await;
    if !sessions.contains_key(&session_id) {
        return Err(ApiError::SessionNotFound(session_id));
    }
    drop(sessions);

    // Create a simple stream that sends a test message
    // In full implementation, this would receive events from the replay engine
    let stream = futures::stream::once(async move {
        Ok(Event::default()
            .event("connected")
            .data(format!("{{\"session_id\":\"{}\",\"message\":\"Connected to replay stream\"}}", session_id)))
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

