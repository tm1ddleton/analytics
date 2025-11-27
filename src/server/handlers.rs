//! HTTP request handlers for API endpoints

use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use chrono::NaiveDate;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::convert::Infallible;
use std::sync::Arc;

use super::error::ApiError;
use super::state::{AnalyticConfig, AppState, ReplaySession, SessionStatus};
use crate::analytics::AnalyticRegistry;
use crate::asset_key::AssetKey;
use crate::dag::{AnalyticType, AnalyticsDag, NodeId, NodeKey, NodeOutput, WindowSpec};
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
    let data_points =
        crate::time_series::DataProvider::get_time_series(&*provider, &asset_key, &date_range)
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

fn build_node_key(
    asset: &AssetKey,
    analytic: AnalyticType,
    date_range: &DateRange,
    params: &HashMap<String, String>,
    override_tag: Option<String>,
) -> Result<NodeKey, ApiError> {
    let mut node_params = params.clone();

    let window_spec = match analytic {
        AnalyticType::Volatility => {
            let window_size = node_params
                .get("window")
                .or_else(|| node_params.get("window_size"))
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(10);

            if window_size == 0 {
                return Err(ApiError::InvalidParameter(
                    "Window size must be greater than 0".to_string(),
                ));
            }

            node_params.insert("window_size".to_string(), window_size.to_string());
            Some(WindowSpec::fixed(window_size))
        }
        _ => None,
    };

    if analytic == AnalyticType::Returns {
        let lag = node_params
            .get("lag")
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|&lag| lag > 0)
            .unwrap_or(1);
        node_params.insert("lag".to_string(), lag.to_string());
    }

    if let Some(tag) = &override_tag {
        node_params.insert("override".to_string(), tag.clone());
    }

    Ok(NodeKey {
        analytic,
        assets: vec![asset.clone()],
        range: Some(date_range.clone()),
        window: window_spec,
        override_tag,
        params: node_params,
    })
}

/// Helper function to build analytics DAG
fn build_analytics_dag(
    asset: &AssetKey,
    analytic_type: &str,
    date_range: &DateRange,
    params: &HashMap<String, String>,
    override_tag: Option<String>,
) -> Result<(AnalyticsDag, NodeId, NodeKey), ApiError> {
    let analytic = AnalyticType::from_str(analytic_type);
    let registry = AnalyticRegistry::default();

    if registry.definition(analytic).is_none() {
        return Err(ApiError::InvalidParameter(format!(
            "Unknown analytic type: {}",
            analytic_type
        )));
    }

    let node_key = build_node_key(asset, analytic, date_range, params, override_tag)?;

    let mut dag = AnalyticsDag::new();
    let target_node = dag
        .resolve_node(node_key.clone())
        .map_err(|e| ApiError::ComputationFailed(e.to_string()))?;

    Ok((dag, target_node, node_key))
}

/// Query parameters for analytics endpoint
#[derive(Debug, Deserialize)]
pub struct AnalyticsQueryParams {
    pub start: String,
    pub end: String,
    pub window: Option<usize>,
    #[serde(rename = "override")]
    pub override_tag: Option<String>,
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
    if let Some(tag) = &query_params.override_tag {
        params.insert("override".to_string(), tag.clone());
    }

    // Create date range
    let date_range = DateRange::new(start_date, end_date);

    // Build DAG
    let (dag, target_node, _) = build_analytics_dag(
        &asset_key,
        &analytic_type,
        &date_range,
        &params,
        query_params.override_tag.clone(),
    )?;

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
    #[serde(rename = "override")]
    #[serde(default)]
    pub override_tag: Option<String>,
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

    // Create date range
    let date_range = DateRange::new(start_date, end_date);

    // Prepare parameters (include override tag)
    let mut params = query.parameters.clone();
    if let Some(tag) = &query.override_tag {
        params.insert("override".to_string(), tag.clone());
    }

    // Build DAG
    let (dag, target_node, _) = build_analytics_dag(
        &asset_key,
        &query.analytic,
        &date_range,
        &params,
        query.override_tag.clone(),
    )?;

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
        parameters: params,
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
            AssetKey::new_equity(a)
                .map_err(|e| ApiError::InvalidParameter(format!("Invalid asset {}: {}", a, e)))
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

    let asset_strings: Vec<String> = session.assets.iter().map(|a| a.to_string()).collect();

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

// Task Group 7: Server-Sent Events Streaming

/// GET /stream/{session_id} - SSE stream for replay updates
pub async fn handle_stream(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let session_id = Uuid::parse_str(&session_id)
        .map_err(|_| ApiError::InvalidParameter("Invalid session ID".to_string()))?;

    // Get session info
    let sessions = state.sessions.read().await;
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| ApiError::SessionNotFound(session_id))?;

    let assets = session.assets.clone();
    let analytics = session.analytics.clone();
    let start_date = session.start_date;
    let end_date = session.end_date;
    drop(sessions);

    // Create channel for sending events
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // Send initial connected event
    let _ = tx.send(Ok(Event::default()
        .event("connected")
        .data(format!("{{\"session_id\":\"{}\"}}", session_id))));

    // Clone state for background task
    let state_clone = state.clone();
    let replay_range = DateRange::new(start_date, end_date);

    // Spawn task to run real push-mode replay
    tokio::spawn(async move {
        use crate::push_mode::PushModeEngine;

        // Process each asset separately (for simplicity in the demo)
        for asset_key in &assets {
            for analytic in &analytics {
                tracing::info!(
                    "Replay: Setting up push-mode for {} {}",
                    asset_key.to_string(),
                    analytic.analytic_type
                );

                // Build parameters
                let mut params = HashMap::new();
                for (key, value) in &analytic.parameters {
                    params.insert(key.clone(), value.clone());
                }
                if let Some(tag) = &analytic.override_tag {
                    params.insert("override".to_string(), tag.clone());
                }

                // Build DAG for this asset and analytic
                let dag_result = build_analytics_dag(
                    asset_key,
                    &analytic.analytic_type,
                    &replay_range,
                    &params,
                    analytic.override_tag.clone(),
                );
                if let Err(e) = dag_result {
                    tracing::error!("Replay: Failed to build DAG: {}", e);
                    continue;
                }

                let (dag, target_node, _) = dag_result.unwrap();

                // Create push-mode engine
                let mut push_engine = PushModeEngine::new(dag);

                // Initialize with historical data (for burn-in)
                let provider = state_clone.data_provider.lock().await;
                let init_end = start_date.and_hms_opt(0, 0, 0).unwrap().and_utc();

                if let Err(e) = push_engine.initialize(&*provider, init_end, 50) {
                    tracing::error!("Replay: Failed to initialize push engine: {}", e);
                    drop(provider);
                    continue;
                }

                // Load all data for this asset in the date range
                use crate::time_series::DataProvider;
                let date_range = replay_range.clone();
                let all_data = match (*provider).get_time_series(asset_key, &date_range) {
                    Ok(data) => data,
                    Err(e) => {
                        tracing::error!("Replay: Failed to load data: {}", e);
                        drop(provider);
                        continue;
                    }
                };
                drop(provider);

                tracing::info!(
                    "Replay: Loaded {} data points, will stream incrementally",
                    all_data.len()
                );

                // Register callback to capture results
                let asset_str = asset_key.to_string();
                let analytic_str = analytic.analytic_type.clone();
                let tx_clone = tx.clone();

                if let Err(e) = push_engine.register_callback(
                    target_node,
                    Box::new(move |_node_id, output, timestamp| {
                        match output {
                            NodeOutput::Single(ref data) => {
                                if let Some(last_point) = data.last() {
                                    if !last_point.close_price.is_nan() {
                                        tracing::debug!(
                                            "Push-mode callback: {} {} at {} = {}",
                                            asset_str,
                                            analytic_str,
                                            last_point.timestamp,
                                            last_point.close_price
                                        );
                                        let _ = tx_clone.send(Ok(Event::default()
                                            .event("update")
                                            .data(format!(
                                                "{{\"asset\":\"{}\",\"analytic\":\"{}\",\"timestamp\":\"{}\",\"value\":{}}}",
                                                asset_str,
                                                analytic_str,
                                                last_point.timestamp.to_rfc3339(),
                                                last_point.close_price
                                            ))));
                                    }
                                }
                            }
                            NodeOutput::Scalar(value) => {
                                if let Some(ts) = timestamp {
                                    if !value.is_nan() {
                                        tracing::debug!(
                                            "Push-mode callback: {} {} at {} = {}",
                                            asset_str,
                                            analytic_str,
                                            ts,
                                            value
                                        );
                                        let _ = tx_clone.send(Ok(Event::default()
                                            .event("update")
                                            .data(format!(
                                                "{{\"asset\":\"{}\",\"analytic\":\"{}\",\"timestamp\":\"{}\",\"value\":{}}}",
                                                asset_str,
                                                analytic_str,
                                                ts.to_rfc3339(),
                                                value
                                            ))));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }),
                ) {
                    tracing::error!("Replay: Failed to register callback: {}", e);
                    continue;
                }

                // Now feed data incrementally
                let num_points = all_data.len();
                for (i, point) in all_data.iter().enumerate() {
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

                    let progress = (i as f64) / (num_points as f64);

                    // Send progress update
                    let _ = tx.send(Ok(Event::default().event("progress").data(format!(
                        "{{\"current_date\":\"{}\",\"progress\":{}}}",
                        point.timestamp.format("%Y-%m-%d"),
                        progress
                    ))));

                    // Push data point - this triggers incremental computation
                    if let Err(e) =
                        push_engine.push_data(asset_key.clone(), point.timestamp, point.close_price)
                    {
                        tracing::error!("Replay: Failed to push data: {}", e);
                    }
                }

                tracing::info!(
                    "Replay: Completed push-mode replay for {} {}",
                    asset_key.to_string(),
                    analytic.analytic_type
                );
            }
        }

        // Send complete event
        let _ = tx.send(Ok(Event::default().event("complete").data("{}")));
    });

    // Create stream from receiver
    let stream = async_stream::stream! {
        while let Some(event) = rx.recv().await {
            yield event;
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_key::AssetKey;
    use crate::dag::AnalyticType;
    use crate::time_series::DateRange;
    use chrono::NaiveDate;
    use std::collections::HashMap;

    #[test]
    fn build_node_key_override_tag_is_distinct() {
        let asset = AssetKey::new_equity("AAPL").unwrap();
        let params = HashMap::new();
        let range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        let base_key =
            build_node_key(&asset, AnalyticType::Returns, &range, &params, None).unwrap();
        let override_key = build_node_key(
            &asset,
            AnalyticType::Returns,
            &range,
            &params,
            Some("arith".to_string()),
        )
        .unwrap();

        assert_ne!(base_key, override_key);
        assert!(override_key.override_tag.as_deref() == Some("arith"));
    }
}
