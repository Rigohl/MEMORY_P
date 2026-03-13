// src/routes.rs - HTTP Axum Routes for MEMORY_P MCP Server

use crate::auto_manager::AutoManager;
use crate::decision_logic::DecisionEngine;
use crate::kpi_tracker::KpiTracker;
use crate::mcp::shared_memory_tools::{
    register_shared_memory_tools, CleanupParams, GetContextParams, SharedMemoryToolHandler,
    SyncContextsParams, UpdateContextParams,
};
use crate::motores::factory::engine_factory::EngineFactory;
use crate::prediction_engine::PredictionEngine;
use crate::shared_memory::SharedMemorySystem;
use crate::telemetry::TelemetrySystem;
use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub text: Option<String>,
    pub vector: Option<Vec<f32>>,
    pub query_type: String,
    pub limit: Option<usize>,
    pub engines: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineSearchRequest {
    pub engine: String,
    pub text: Option<String>,
    pub vector: Option<Vec<f32>>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Health check endpoint
pub async fn health_check(
    Extension(_telemetry): Extension<Arc<TelemetrySystem>>,
) -> impl IntoResponse {
    let response = HealthResponse {
        status: "operational".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };
    (StatusCode::OK, Json(response))
}

/// MCP JSON-RPC 2.0 handler
pub async fn mcp_handler(
    Extension(shared_memory): Extension<Arc<SharedMemorySystem>>,
    Extension(kpi_tracker): Extension<Arc<KpiTracker>>,
    Extension(_auto_manager): Extension<Arc<AutoManager>>,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    tracing::info!("MCP request: method={}", req.method);

    match req.method.as_str() {
        "tools/list" => {
            let mut tools = vec![
                serde_json::json!({
                    "name": "search_text",
                    "description": "Full-text search using Tantivy",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "text": { "type": "string" },
                            "limit": { "type": "integer", "minimum": 1 }
                        },
                        "required": ["text"]
                    }
                }),
                serde_json::json!({
                    "name": "search_vector",
                    "description": "Vector similarity search using Qdrant",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "vector": { "type": "array", "items": { "type": "number" } },
                            "limit": { "type": "integer", "minimum": 1 }
                        },
                        "required": ["vector"]
                    }
                }),
                serde_json::json!({
                    "name": "search_hybrid",
                    "description": "Hybrid text + vector search",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "text": { "type": "string" },
                            "vector": { "type": "array", "items": { "type": "number" } },
                            "limit": { "type": "integer", "minimum": 1 }
                        }
                    }
                }),
                serde_json::json!({
                    "name": "list_engines",
                    "description": "List all declared motores and their availability in MCP",
                    "inputSchema": { "type": "object", "properties": {} }
                }),
                serde_json::json!({
                    "name": "search_engine",
                    "description": "Execute a search against any declared motor by name",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "engine": { "type": "string" },
                            "text": { "type": "string" },
                            "vector": { "type": "array", "items": { "type": "number" } },
                            "limit": { "type": "integer", "minimum": 1 }
                        },
                        "required": ["engine"]
                    }
                }),
                serde_json::json!({
                    "name": "kpi_report",
                    "description": "Get real KPI dashboard from the always-on tracker",
                    "inputSchema": { "type": "object", "properties": {} }
                }),
            ];
            tools.extend(register_shared_memory_tools());

            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: Some(serde_json::json!({
                    "tools": tools
                })),
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
        "tools/call" => {
            let params = req.params.unwrap_or_else(|| serde_json::json!({}));
            let tool_name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));

            let shared_memory_tools = SharedMemoryToolHandler::new(shared_memory.clone());

            let tool_result: Result<serde_json::Value, String> = match tool_name {
                "search_text" => {
                    let text = arguments
                        .get("text")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "search_text requires 'text'".to_string());

                    match text {
                        Ok(text) => {
                            execute_text_search(
                                text,
                                arguments
                                    .get("limit")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as usize),
                            )
                            .await
                        }
                        Err(e) => Err(e),
                    }
                }
                "search_vector" => {
                    let vector = arguments
                        .get("vector")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|n| n.as_f64().map(|f| f as f32))
                                .collect::<Vec<f32>>()
                        })
                        .ok_or_else(|| "search_vector requires 'vector'".to_string());

                    match vector {
                        Ok(vector) => {
                            execute_vector_search(
                                vector,
                                arguments
                                    .get("limit")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as usize),
                            )
                            .await
                        }
                        Err(e) => Err(e),
                    }
                }
                "search_hybrid" => {
                    let text = arguments
                        .get("text")
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    let vector = arguments
                        .get("vector")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|n| n.as_f64().map(|f| f as f32))
                                .collect::<Vec<f32>>()
                        });
                    execute_hybrid_search(
                        text,
                        vector,
                        arguments
                            .get("limit")
                            .and_then(|v| v.as_u64())
                            .map(|v| v as usize),
                    )
                    .await
                }
                "list_engines" => Ok(serde_json::json!({
                    "engines": EngineFactory::available_engines(),
                })),
                "search_engine" => {
                    let engine = arguments
                        .get("engine")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| "search_engine requires 'engine'".to_string());
                    let text = arguments
                        .get("text")
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    let vector = arguments
                        .get("vector")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|n| n.as_f64().map(|f| f as f32))
                                .collect::<Vec<f32>>()
                        });
                    match engine {
                        Ok(engine) => {
                            execute_named_engine_search(
                                engine,
                                text,
                                vector,
                                arguments
                                    .get("limit")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as usize),
                            )
                            .await
                        }
                        Err(e) => Err(e),
                    }
                }
                "kpi_report" => serde_json::to_value(kpi_tracker.get_dashboard())
                    .map_err(|e| format!("Failed to serialize KPI dashboard: {e}")),
                "get_agent_context" => {
                    let typed: GetContextParams = match serde_json::from_value(arguments) {
                        Ok(v) => v,
                        Err(e) => {
                            return {
                                let response = JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    id: req.id,
                                    result: None,
                                    error: Some(JsonRpcError {
                                        code: -32602,
                                        message: format!("Invalid params: {e}"),
                                        data: None,
                                    }),
                                };
                                (StatusCode::OK, Json(response))
                            }
                        }
                    };
                    shared_memory_tools
                        .get_agent_context(typed)
                        .await
                        .map_err(|e| e.to_string())
                }
                "update_agent_context" => {
                    match serde_json::from_value::<UpdateContextParams>(arguments) {
                        Ok(typed) => shared_memory_tools
                            .update_agent_context(typed)
                            .await
                            .map_err(|e| e.to_string()),
                        Err(e) => Err(format!("Invalid update_agent_context params: {e}")),
                    }
                }
                "sync_agent_contexts" => {
                    match serde_json::from_value::<SyncContextsParams>(arguments) {
                        Ok(typed) => shared_memory_tools
                            .sync_contexts(typed)
                            .await
                            .map_err(|e| e.to_string()),
                        Err(e) => Err(format!("Invalid sync_agent_contexts params: {e}")),
                    }
                }
                "get_memory_stats" => shared_memory_tools
                    .get_memory_stats()
                    .await
                    .map_err(|e| e.to_string()),
                "cleanup_inactive_contexts" => {
                    match serde_json::from_value::<CleanupParams>(arguments) {
                        Ok(typed) => shared_memory_tools
                            .cleanup_inactive_contexts(typed)
                            .await
                            .map_err(|e| e.to_string()),
                        Err(e) => Err(format!("Invalid cleanup_inactive_contexts params: {e}")),
                    }
                }
                "register_prediction" => shared_memory_tools
                    .register_prediction(arguments)
                    .await
                    .map_err(|e| e.to_string()),
                "get_next_moves" => shared_memory_tools
                    .get_next_moves(arguments)
                    .await
                    .map_err(|e| e.to_string()),
                _ => Err(format!("Unknown tool: {tool_name}")),
            };

            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: tool_result.as_ref().ok().cloned(),
                error: tool_result.err().map(|message| JsonRpcError {
                    code: -32000,
                    message,
                    data: None,
                }),
            };
            (StatusCode::OK, Json(response))
        }
        _ => {
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            };
            (StatusCode::NOT_FOUND, Json(response))
        }
    }
}

/// Text search endpoint - REAL Tantivy engine
pub async fn search_text(Json(req): Json<SearchRequest>) -> impl IntoResponse {
    let text = req.text.unwrap_or_default();
    tracing::info!("Text search executing: {}", text);

    match execute_text_search(&text, req.limit).await {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(e) => {
            tracing::error!("Tantivy search error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e,
                    "results": []
                })),
            )
        }
    }
}

pub async fn execute_text_search(
    text: &str,
    limit: Option<usize>,
) -> Result<serde_json::Value, String> {
    use crate::motores::core::traits::SearchEngine;
    use crate::motores::core::types::{EngineConfig, QueryType, SearchQuery};
    use crate::motores::text_search::tantivy::engine::TantivyEngine;

    let mut engine = TantivyEngine::new(EngineConfig::default());
    match engine.initialize().await {
        Ok(_) => {
            let query = SearchQuery {
                text: text.to_string(),
                vector: None,
                query_type: QueryType::Term,
                limit: limit.unwrap_or(10),
                offset: 0,
                filters: HashMap::new(),
                min_score: 0.0,
            };

            match engine.search(&query).await {
                Ok(results) => Ok(serde_json::json!({
                    "results": results.iter().map(|r| serde_json::json!({
                        "id": r.id,
                        "score": r.score,
                        "content": r.content,
                        "engine": r.engine
                    })).collect::<Vec<_>>(),
                    "total": results.len(),
                    "query": text
                })),
                Err(e) => Err(format!("Search error: {e}")),
            }
        }
        Err(e) => Err(format!("Engine init failed: {e}")),
    }
}

/// Vector search endpoint - REAL JAX + Qdrant
pub async fn search_vector(Json(req): Json<SearchRequest>) -> impl IntoResponse {
    let vector_dim = req.vector.as_ref().map(|v| v.len()).unwrap_or(0);
    tracing::info!("Vector search executing: dimension={}", vector_dim);

    if vector_dim == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Vector required for vector search",
                "results": []
            })),
        );
    }

    match execute_vector_search(req.vector.unwrap_or_default(), req.limit).await {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(e) => {
            tracing::error!("Qdrant search error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e,
                    "results": []
                })),
            )
        }
    }
}

pub async fn execute_vector_search(
    vector: Vec<f32>,
    limit: Option<usize>,
) -> Result<serde_json::Value, String> {
    use crate::motores::core::traits::SearchEngine;
    use crate::motores::core::types::{EngineConfig, QueryType, SearchQuery};
    use crate::motores::vector_search::qdrant::engine::QdrantEngine;

    let vector_dim = vector.len();
    let mut engine = QdrantEngine::new(EngineConfig::default());
    match engine.initialize().await {
        Ok(_) => {
            let query = SearchQuery {
                text: String::new(),
                vector: Some(vector),
                query_type: QueryType::Vector,
                limit: limit.unwrap_or(10),
                offset: 0,
                filters: HashMap::new(),
                min_score: 0.0,
            };

            match engine.search(&query).await {
                Ok(results) => Ok(serde_json::json!({
                    "results": results.iter().map(|r: &crate::motores::core::types::SearchResult| serde_json::json!({
                        "id": r.id.clone(),
                        "score": r.score,
                        "content": r.content.clone(),
                        "engine": r.engine.clone()
                    })).collect::<Vec<_>>(),
                    "total": results.len(),
                    "vector_dim": vector_dim
                })),
                Err(e) => Err(format!("Search error: {e}")),
            }
        }
        Err(e) => Err(format!("Engine init failed: {e}")),
    }
}

/// Hybrid search endpoint - REAL orchestration of Tantivy + Qdrant
pub async fn search_hybrid(Json(req): Json<SearchRequest>) -> impl IntoResponse {
    tracing::info!("Hybrid search executing");

    match execute_hybrid_search(req.text.clone(), req.vector.clone(), req.limit).await {
        Ok(mut response) => {
            if let Some(obj) = response.as_object_mut() {
                obj.insert(
                    "engines".to_string(),
                    serde_json::json!(req.engines.unwrap_or_default()),
                );
            }
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("Hybrid search error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e,
                    "results": []
                })),
            )
        }
    }
}

pub async fn search_engine(Json(req): Json<EngineSearchRequest>) -> impl IntoResponse {
    tracing::info!("Engine search executing: {}", req.engine);

    match execute_named_engine_search(&req.engine, req.text, req.vector, req.limit).await {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(e) => {
            tracing::error!("Named engine search error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e,
                    "results": []
                })),
            )
        }
    }
}

pub async fn execute_hybrid_search(
    text: Option<String>,
    vector: Option<Vec<f32>>,
    limit: Option<usize>,
) -> Result<serde_json::Value, String> {
    let mut combined_results = Vec::new();

    if let Some(text) = text.as_deref() {
        if !text.is_empty() {
            let response = execute_text_search(text, Some(limit.unwrap_or(5))).await?;
            if let Some(results) = response.get("results").and_then(|v| v.as_array()) {
                combined_results.extend(results.iter().cloned());
            }
        }
    }

    if let Some(vector) = vector {
        if !vector.is_empty() {
            let response = execute_vector_search(vector, Some(limit.unwrap_or(5))).await?;
            if let Some(results) = response.get("results").and_then(|v| v.as_array()) {
                combined_results.extend(results.iter().cloned());
            }
        }
    }

    combined_results.sort_by(|a, b| {
        let score_a = a.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let score_b = b.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    combined_results.dedup_by(|a, b| a.get("id") == b.get("id"));
    combined_results.truncate(limit.unwrap_or(10));

    Ok(serde_json::json!({
        "results": combined_results,
        "total": combined_results.len()
    }))
}

async fn execute_named_engine_search(
    engine_name: &str,
    text: Option<String>,
    vector: Option<Vec<f32>>,
    limit: Option<usize>,
) -> Result<serde_json::Value, String> {
    use crate::motores::core::types::{EngineConfig, QueryType, SearchQuery};
    use crate::motores::specialized::{JuliaNlpEngine, MemoryBankEngine};
    use crate::motores::text_search::{LnxEngine, MeiliSearchEngine, TantivyEngine, ToshiEngine};
    use crate::motores::vector_search::{FaissEngine, QdrantEngine, ScannEngine};

    let query = SearchQuery {
        text: text.unwrap_or_default(),
        vector,
        query_type: QueryType::Hybrid,
        limit: limit.unwrap_or(10),
        offset: 0,
        filters: HashMap::new(),
        min_score: 0.0,
    };

    let results = match engine_name {
        "qdrant" => run_engine(QdrantEngine::new(EngineConfig::default()), &query).await,
        "faiss" => run_engine(FaissEngine::new(EngineConfig::default()), &query).await,
        "scann" => run_engine(ScannEngine::new(EngineConfig::default()), &query).await,
        "tantivy" => run_engine(TantivyEngine::new(EngineConfig::default()), &query).await,
        "lnx" => run_engine(LnxEngine::new(EngineConfig::default()), &query).await,
        "toshi" => run_engine(ToshiEngine::new(EngineConfig::default()), &query).await,
        "meilisearch" => run_engine(MeiliSearchEngine::new(EngineConfig::default()), &query).await,
        "julia_nlp" => run_engine(JuliaNlpEngine::new(EngineConfig::default()), &query).await,
        "memory_bank" => run_engine(MemoryBankEngine::new(EngineConfig::default()), &query).await,
        _ => return Err(format!("Unknown engine: {}", engine_name)),
    }?;

    Ok(serde_json::json!({
        "engine": engine_name,
        "results": results.iter().map(|r| serde_json::json!({
            "id": r.id,
            "score": r.score,
            "content": r.content,
            "engine": r.engine,
            "metadata": r.metadata,
            "highlights": r.highlights,
        })).collect::<Vec<_>>(),
        "total": results.len(),
    }))
}

async fn run_engine<E>(
    mut engine: E,
    query: &crate::motores::core::types::SearchQuery,
) -> Result<Vec<crate::motores::core::types::SearchResult>, String>
where
    E: crate::motores::core::traits::SearchEngine,
{
    engine
        .initialize()
        .await
        .map_err(|e| format!("Engine init failed: {e}"))?;
    engine
        .search(query)
        .await
        .map_err(|e| format!("Engine search failed: {e}"))
}

/// Create complete router with all routes
pub fn create_router(
    auto_manager: Arc<AutoManager>,
    kpi_tracker: Arc<KpiTracker>,
    shared_memory: Arc<SharedMemorySystem>,
    prediction_engine: Arc<PredictionEngine>,
    decision_engine: Arc<DecisionEngine>,
    telemetry: Arc<TelemetrySystem>,
) -> Router {
    Router::new()
        // Health
        .route("/health", get(health_check))

        // MCP JSON-RPC (use closures to ensure Handler trait compatibility)
        .route(
            "/mcp",
            post(|Extension(shared_memory): Extension<Arc<SharedMemorySystem>>, Extension(kpi_tracker): Extension<Arc<KpiTracker>>, Extension(_auto_manager): Extension<Arc<AutoManager>>, Json(req): Json<JsonRpcRequest>| async move {
                mcp_handler(Extension(shared_memory), Extension(kpi_tracker), Extension(_auto_manager), Json(req)).await
            }),
        )

        // Search endpoints (wrap in closures to avoid ambiguous Handler bounds)
        .route(
            "/search/text",
            post(|Json(req): Json<SearchRequest>| async move { search_text(Json(req)).await }),
        )
        .route(
            "/search/vector",
            post(|Json(req): Json<SearchRequest>| async move { search_vector(Json(req)).await }),
        )
        .route(
            "/search/hybrid",
            post(|Json(req): Json<SearchRequest>| async move { search_hybrid(Json(req)).await }),
        )
        .route(
            "/search/engine",
            post(|Json(req): Json<EngineSearchRequest>| async move { search_engine(Json(req)).await }),
        )

        // Fallback
        .fallback(|| async { (StatusCode::NOT_FOUND, "Not Found") })

        // Extensions
        .layer(Extension(auto_manager))
        .layer(Extension(kpi_tracker))
        .layer(Extension(shared_memory))
        .layer(Extension(prediction_engine))
        .layer(Extension(decision_engine))
        .layer(Extension(telemetry))
}
