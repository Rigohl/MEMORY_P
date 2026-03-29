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
    pub results: Vec<CapResult>,
    pub latency_ms: f64,
    pub status: SearchStatus,
    pub metadata: SearchMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapResult {
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

pub async fn chapel_search(
    Extension(_telemetry): Extension<Arc<TelemetrySystem>>,
    Json(req): Json<MultiLanguageSearchRequest>,
) -> impl IntoResponse {
    let start = Instant::now();
    let sla_target_ms = req.timeout_ms.unwrap_or(250);
    let (results, parallelism) = execute_chapel_search(&req).await;
    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    let sla_met = latency_ms < sla_target_ms as f64;
    let response = MultiLanguageSearchResponse {
        engine: "chapel-2.0".to_string(),
        results,
        latency_ms,
        status: if sla_met { SearchStatus::Success } else { SearchStatus::Timeout },
        metadata: SearchMetadata {
            timestamp: chrono::Utc::now().timestamp(),
            parallelism_level: parallelism,
            engine_version: "2.0.0".to_string(),
            sla_target_ms,
            sla_met,
        },
    };
    let status = if sla_met { StatusCode::OK } else { StatusCode::GATEWAY_TIMEOUT };
    (status, Json(response))
}

pub async fn pony_search(
    Extension(_telemetry): Extension<Arc<TelemetrySystem>>,
    Json(req): Json<MultiLanguageSearchRequest>,
) -> impl IntoResponse {
    let start = Instant::now();
    let sla_target_ms = req.timeout_ms.unwrap_or(300);
    let (results, actor_count) = execute_pony_search(&req).await;
    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    let sla_met = latency_ms < sla_target_ms as f64;
    let response = MultiLanguageSearchResponse {
        engine: "pony-actors".to_string(),
        results,
        latency_ms,
        status: if sla_met { SearchStatus::Success } else { SearchStatus::Partial },
        metadata: SearchMetadata {
            timestamp: chrono::Utc::now().timestamp(),
            parallelism_level: actor_count,
            engine_version: "0.1.0".to_string(),
            sla_target_ms,
            sla_met,
        },
    };
    let status = if sla_met { StatusCode::OK } else { StatusCode::REQUEST_TIMEOUT };
    (status, Json(response))
}

pub async fn hybrid_multi_language_search(
    Extension(_telemetry): Extension<Arc<TelemetrySystem>>,
    Json(req): Json<MultiLanguageSearchRequest>,
) -> impl IntoResponse {
    let start = Instant::now();
    let (chapel_result, pony_result) = tokio::join!(
        execute_chapel_search(&req),
        execute_pony_search(&req)
    );
    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
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
            parallelism_level: 16,
            engine_version: "2.0-hybrid".to_string(),
            sla_target_ms: 250,
            sla_met,
        },
    };
    (StatusCode::OK, Json(response))
}

async fn execute_chapel_search(req: &MultiLanguageSearchRequest) -> (Vec<CapResult>, usize) {
    (vec![
        CapResult { id: "chapel_1".to_string(), score: 0.95, content: format!("Chapel result for: {}", req.query), rank: 1 },
        CapResult { id: "chapel_2".to_string(), score: 0.87, content: "Parallel multi-locale result".to_string(), rank: 2 },
    ], 8)
}

async fn execute_pony_search(req: &MultiLanguageSearchRequest) -> (Vec<CapResult>, usize) {
    (vec![
        CapResult { id: "pony_1".to_string(), score: 0.92, content: format!("Pony actor result for: {}", req.query), rank: 1 },
        CapResult { id: "pony_2".to_string(), score: 0.84, content: "Actor-isolated result".to_string(), rank: 2 },
    ], 4)
}
