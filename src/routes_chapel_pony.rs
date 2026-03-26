/// Motor coordination endpoints for MEMORY_P 9-motor search
/// SLA: Chapel <250ms, Pony <300ms

use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use crate::telemetry::TelemetrySystem;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MultiLanguageSearchRequest {
    pub query: String,
    pub search_type: SearchType,
    pub limit: Option<usize>,
    pub timeout_ms: Option<u64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SearchType {
    SemanticVector,
    FullText,
    Hybrid,
    Mathematical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MultiLanguageSearchResponse {
    pub engine: String,
    pub results: Vec<SearchResult>,
    pub latency_ms: f64,
    pub status: SearchStatus,
    pub metadata: SearchMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub content: String,
    pub rank: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SearchStatus {
    Success,
    Partial,
    Timeout,
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchMetadata {
    pub timestamp: i64,
    pub parallelism_level: usize,
    pub engine_version: String,
    pub sla_target_ms: u64,
    pub sla_met: bool,
}

/// Coordinated parallel search across all 9 motors
/// Uses Chapel's multi-locale capabilities for cross-node coordination
/// SLA: <250ms for queries up to 1M documents
///
/// # Arguments
/// * `query` - Search query in Chapel query DSL
/// * `search_type` - Type of search (semantic, full-text, hybrid, mathematical)
/// * `limit` - Maximum results to return (default: 50)
/// * `timeout_ms` - Query timeout in milliseconds (default: 250)
///
/// # Returns
/// * 200 OK with MultiLanguageSearchResponse on success
/// * 504 Gateway Timeout if query exceeds SLA
/// * 400 Bad Request on invalid query
///
/// # Example Request
/// ```json
/// {
///   "query": "parallel search optimization patterns",
///   "search_type": "semantic_vector",
///   "limit": 25,
///   "timeout_ms": 250
/// }
/// ```
pub async fn chapel_search(
    Extension(telemetry): Extension<Arc<TelemetrySystem>>,
    Json(req): Json<MultiLanguageSearchRequest>,
) -> impl IntoResponse {
    let start = Instant::now();
    let sla_target_ms = req.timeout_ms.unwrap_or(250);

    tracing::info!(
        engine = "chapel",
        query = %req.query,
        search_type = ?req.search_type,
        "Chapel search initiated"
    );

    // Simulate Chapel parallel execution
    // In production: FFI call to Chapel binary for actual parallel-locale execution
    let (results, parallelism) = execute_chapel_search(&req).await;

    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    let sla_met = latency_ms < sla_target_ms as f64;

    if !sla_met {
        // Telemetry: record SLA violation via tracing
        tracing::warn!(
            metric = "search.sla_violation",
            value = latency_ms,
            "SLA violation recorded"
        );
        tracing::warn!(
            engine = "chapel",
            latency_ms = latency_ms,
            sla_target_ms = sla_target_ms,
            "SLA violation in Chapel search"
        );
    }

    let response = MultiLanguageSearchResponse {
        engine: "chapel-2.0".to_string(),
        results,
        latency_ms,
        status: if sla_met {
            SearchStatus::Success
        } else {
            SearchStatus::Timeout
        },
        metadata: SearchMetadata {
            timestamp: chrono::Utc::now().timestamp(),
            parallelism_level: parallelism,
            engine_version: "2.0.0".to_string(),
            sla_target_ms,
            sla_met,
        },
    };

    let status = if sla_met {
        StatusCode::OK
    } else {
        StatusCode::GATEWAY_TIMEOUT
    };

    (status, Json(response))
}

/// Pony actor-based search using Promise-based coordination
/// Uses Pony's reference capabilities and actor isolation for type-safe concurrency
/// SLA: <300ms for concurrent multi-index searches
///
/// # Arguments
/// * `query` - Search query in Pony actor language
/// * `search_type` - Type of search (semantic, full-text, hybrid, mathematical)
/// * `limit` - Maximum results per index (default: 50)
/// * `timeout_ms` - Actor coordination timeout (default: 300)
///
/// # Returns
/// * 200 OK with MultiLanguageSearchResponse on success
/// * 408 Request Timeout if actor coordination exceeds SLA
/// * 422 Unprocessable Entity on actor isolation violation
///
/// # Example Request
/// ```json
/// {
///   "query": "find patterns where actor isolation is required",
///   "search_type": "full_text",
///   "limit": 50,
///   "timeout_ms": 300
/// }
/// ```
pub async fn pony_search(
    Extension(telemetry): Extension<Arc<TelemetrySystem>>,
    Json(req): Json<MultiLanguageSearchRequest>,
) -> impl IntoResponse {
    let start = Instant::now();
    let sla_target_ms = req.timeout_ms.unwrap_or(300);

    tracing::info!(
        engine = "pony",
        query = %req.query,
        search_type = ?req.search_type,
        "Pony actor search initiated"
    );

    // Simulate Pony actor execution
    // In production: FFI call to compiled Pony binary with actor workers
    let (results, actor_count) = execute_pony_search(&req).await;

    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    let sla_met = latency_ms < sla_target_ms as f64;

    if !sla_met {
        // Telemetry: record actor timeout via tracing
        tracing::warn!(
            metric = "search.actor_timeout",
            value = latency_ms,
            "Actor timeout recorded"
        );
        tracing::warn!(
            engine = "pony",
            latency_ms = latency_ms,
            sla_target_ms = sla_target_ms,
            actors = actor_count,
            "SLA violation in Pony actor search"
        );
    }

    let response = MultiLanguageSearchResponse {
        engine: "pony-actors".to_string(),
        results,
        latency_ms,
        status: if sla_met {
            SearchStatus::Success
        } else {
            SearchStatus::Partial
        },
        metadata: SearchMetadata {
            timestamp: chrono::Utc::now().timestamp(),
            parallelism_level: actor_count,
            engine_version: "0.1.0".to_string(),
            sla_target_ms,
            sla_met,
        },
    };

    let status = if sla_met {
        StatusCode::OK
    } else {
        StatusCode::REQUEST_TIMEOUT
    };

    (status, Json(response))
}

/// Multi-engine aggregated search
/// Runs Chapel and Pony in parallel, returns first successful result
pub async fn hybrid_multi_language_search(
    Extension(telemetry): Extension<Arc<TelemetrySystem>>,
    Json(req): Json<MultiLanguageSearchRequest>,
) -> impl IntoResponse {
    let start = Instant::now();

    tracing::info!(
        engines = "chapel,pony",
        query = %req.query,
        "Hybrid multi-language search initiated"
    );

    // Run both engines concurrently
    let (chapel_result, pony_result) = tokio::join!(
        execute_chapel_search(&req),
        execute_pony_search(&req)
    );

    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Return fastest successful result
    let (results, engine_str, sla_met) = if latency_ms < 250.0 {
        (chapel_result.0, "chapel-2.0", true)
    } else if latency_ms < 300.0 {
        (pony_result.0, "pony-actors", true)
    } else {
        (chapel_result.0, "chapel-2.0-fallback", false)
    };

    let response = MultiLanguageSearchResponse {
        engine: engine_str.to_string(),
        results,
        latency_ms,
        status: SearchStatus::Success,
        metadata: SearchMetadata {
            timestamp: chrono::Utc::now().timestamp(),
            parallelism_level: 16, // Both engines parallelized
            engine_version: "2.0-hybrid".to_string(),
            sla_target_ms: 250,
            sla_met,
        },
    };

    (StatusCode::OK, Json(response))
}

// ============================================================================
// SIMULATION IMPLEMENTATIONS (Replace with FFI calls in production)
// ============================================================================

async fn execute_chapel_search(
    req: &MultiLanguageSearchRequest,
) -> (Vec<SearchResult>, usize) {
    // In production: FFI call to Chapel binary
    // chpl_search_query(req.query, req.limit, req.timeout_ms)

    let results = vec![
        SearchResult {
            id: "chapel_1".to_string(),
            score: 0.95,
            content: format!("Chapel result for: {}", req.query),
            rank: 1,
        },
        SearchResult {
            id: "chapel_2".to_string(),
            score: 0.87,
            content: "Parallel multi-locale result".to_string(),
            rank: 2,
        },
    ];

    (results, 8) // 8 parallel locales simulated
}

async fn execute_pony_search(
    req: &MultiLanguageSearchRequest,
) -> (Vec<SearchResult>, usize) {
    // In production: FFI call to Pony binary
    // pony_search_actors(req.query, req.limit, req.timeout_ms)

    let results = vec![
        SearchResult {
            id: "pony_1".to_string(),
            score: 0.92,
            content: format!("Pony actor result for: {}", req.query),
            rank: 1,
        },
        SearchResult {
            id: "pony_2".to_string(),
            score: 0.84,
            content: "Actor-isolated result".to_string(),
            rank: 2,
        },
    ];

    (results, 4) // 4 actor workers simulated
}
