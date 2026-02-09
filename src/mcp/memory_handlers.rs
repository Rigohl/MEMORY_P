// Memory MCP Handlers - HTTP endpoints for predictive memory system
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Duration;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use super::memory_engine::{PredictiveMemory, PredictiveMemoryEngine};
use super::memory_models::*;

/// Shared state for memory system
pub struct MemoryState {
    pub engine: Arc<PredictiveMemoryEngine>,
}

impl MemoryState {
    pub fn new(config: MemoryEngineConfig) -> Self {
        Self {
            engine: Arc::new(PredictiveMemoryEngine::new(config)),
        }
    }
}

/// Store a new context in memory
/// POST /mcp/memory/store
pub async fn store_context_handler(
    State(state): State<Arc<MemoryState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let content = match payload.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Missing 'content' field" })),
            )
                .into_response();
        }
    };

    let mut ctx = MemoryContext::new(content);

    // Optional embedding with validation
    if let Some(embedding) = payload.get("embedding").and_then(|v| v.as_array()) {
        let emb: Vec<f64> = embedding
            .iter()
            .filter_map(|v| v.as_f64())
            .collect();
        
        // Validate embedding dimensions (expected: 1536 for OpenAI compatibility)
        if !emb.is_empty() {
            const EXPECTED_DIM: usize = 1536;
            if emb.len() != EXPECTED_DIM {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ 
                        "error": format!("Invalid embedding dimension: expected {}, got {}", EXPECTED_DIM, emb.len()) 
                    })),
                )
                    .into_response();
            }
            ctx = ctx.with_embedding(emb);
        }
    }

    match state.engine.store_context(ctx).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({
                "id": id,
                "status": "stored"
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Retrieve a context by ID
/// GET /mcp/memory/context/:id
pub async fn get_context_handler(
    State(state): State<Arc<MemoryState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.engine.get_context(id).await {
        Ok(Some(ctx)) => (StatusCode::OK, Json(json!(ctx))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Context not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Predict next contexts
/// POST /mcp/memory/predict
pub async fn predict_next_handler(
    State(state): State<Arc<MemoryState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let context_id = match payload.get("context_id").and_then(|v| v.as_str()) {
        Some(id_str) => match Uuid::parse_str(id_str) {
            Ok(id) => id,
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Invalid UUID format" })),
                )
                    .into_response();
            }
        },
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Missing 'context_id' field" })),
            )
                .into_response();
        }
    };

    let lookahead = payload
        .get("lookahead")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;

    // Get the context first
    let current_ctx = match state.engine.get_context(context_id).await {
        Ok(Some(ctx)) => ctx,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Context not found" })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    match state.engine.predict_next(&current_ctx, lookahead).await {
        Ok(result) => (StatusCode::OK, Json(json!(result))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Reorder contexts by strategy
/// POST /mcp/memory/reorder
pub async fn reorder_contexts_handler(
    State(state): State<Arc<MemoryState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let strategy_str = payload
        .get("strategy")
        .and_then(|v| v.as_str())
        .unwrap_or("combined");

    let strategy = match strategy_str {
        "most_accessed" => ReorderStrategy::MostAccessed,
        "most_recent" => ReorderStrategy::MostRecent,
        "highest_score" => ReorderStrategy::HighestPredictionScore,
        "combined" => ReorderStrategy::Combined,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid strategy. Use: most_accessed, most_recent, highest_score, or combined" })),
            )
                .into_response();
        }
    };

    match state.engine.auto_reorder(strategy).await {
        Ok(count) => (
            StatusCode::OK,
            Json(json!({
                "reordered": count,
                "strategy": strategy_str
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Cleanup stale contexts
/// POST /mcp/memory/cleanup
pub async fn cleanup_stale_handler(
    State(state): State<Arc<MemoryState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let threshold_hours = payload
        .get("threshold_hours")
        .and_then(|v| v.as_i64())
        .unwrap_or(24);

    let threshold = Duration::hours(threshold_hours);

    match state.engine.cleanup_stale(threshold).await {
        Ok(removed) => (
            StatusCode::OK,
            Json(json!({
                "removed": removed,
                "threshold_hours": threshold_hours
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Get memory statistics
/// GET /mcp/memory/stats
pub async fn get_stats_handler(
    State(state): State<Arc<MemoryState>>,
) -> impl IntoResponse {
    match state.engine.get_stats().await {
        Ok(stats) => (StatusCode::OK, Json(json!(stats))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
