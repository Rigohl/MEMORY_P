// src/mcp/http_server.rs - MCP Streamable HTTP Server (Spec 2025-11-25)
//
// Implements the Model Context Protocol over Streamable HTTP transport.
// Single /mcp endpoint supporting POST (JSON-RPC), GET (SSE), DELETE (session close).
// Auto-gestión: self-activating chaos-math decision engine + workspace scanner.

use crate::motores::persistence::PersistenceLayer;
use axum::{
    extract::{Json, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response, Sse},
    routing::{delete, get, post},
    Router,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use uuid::Uuid;

// ── Protocol Constants ──────────────────────────────────────────────────
const MCP_PROTOCOL_VERSION: &str = "2025-11-25";
const SERVER_NAME: &str = "MEMORY_P";
const SERVER_VERSION: &str = "3.0.0";

// ── Data Types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub engines: Option<Vec<String>>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
    pub text: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPInfo {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub engines: Vec<String>,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFIStatus {
    pub zig: bool,
    pub julia: bool,
    pub mojo: bool,
    pub pony: bool,
    pub jax: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Active MCP session
#[derive(Debug, Clone)]
pub struct McpSession {
    pub id: String,
    pub protocol_version: String,
    pub created_at: Instant,
    pub initialized: bool,
}

/// Application state shared across handlers
pub struct AppState {
    pub info: MCPInfo,
    pub ffi_status: FFIStatus,
    pub sessions: DashMap<String, McpSession>,
    pub notification_tx: broadcast::Sender<String>,
    pub workspace_root: String,
}

impl AppState {
    /// Detect real FFI availability at startup
    async fn detect_ffi_status() -> FFIStatus {
        let _ = crate::ffi::init().await;
        let status = crate::ffi::detect_status();

        FFIStatus {
            zig: status.zig,
            julia: status.julia,
            mojo: status.mojo,
            pony: status.pony,
            jax: status.jax,
        }
    }

    /// Chaos-math workspace analysis using Julia FFI entropy + Lyapunov
    pub fn analyze_workspace_chaos(&self) -> serde_json::Value {
        let workspace = std::path::Path::new(&self.workspace_root);
        let mut file_sizes: Vec<f64> = Vec::new();
        let mut total_files = 0u64;
        let mut total_dirs = 0u64;
        let mut extensions: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        if let Ok(entries) = jwalk::WalkDir::new(workspace)
            .skip_hidden(true)
            .into_iter()
            .take(5000)
            .collect::<Result<Vec<_>, _>>()
        {
            for entry in &entries {
                if entry.file_type().is_file() {
                    total_files += 1;
                    if let Ok(meta) = entry.metadata() {
                        file_sizes.push(meta.len() as f64);
                    }
                    if let Some(ext) = entry.path().extension() {
                        *extensions
                            .entry(ext.to_string_lossy().to_string())
                            .or_insert(0) += 1;
                    }
                } else if entry.file_type().is_dir() {
                    total_dirs += 1;
                }
            }
        }

        let entropy = crate::ffi::julia::shannon_entropy(&file_sizes);
        let chaos = crate::ffi::julia::chaos_analysis(&file_sizes).unwrap_or(0.0);
        let decision = crate::ffi::julia::get_search_decision(entropy, chaos, 0.5)
            .unwrap_or_else(|_| "sequential".to_string());

        serde_json::json!({
            "workspace": self.workspace_root,
            "total_files": total_files,
            "total_dirs": total_dirs,
            "extensions": extensions,
            "chaos_metrics": {
                "entropy": entropy,
                "lyapunov_exponent": chaos,
                "recommended_strategy": decision,
            }
        })
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn json_rpc_ok(id: Option<serde_json::Value>, result: serde_json::Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id,
        result: Some(result),
        error: None,
    }
}

fn json_rpc_err(
    id: Option<serde_json::Value>,
    code: i32,
    msg: impl Into<String>,
) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message: msg.into(),
            data: None,
        }),
    }
}

#[allow(clippy::result_large_err)]
fn validate_origin(headers: &HeaderMap) -> Result<(), Response> {
    if let Some(origin) = headers.get(header::ORIGIN) {
        let origin_str = origin.to_str().unwrap_or("");
        // Allow localhost origins and no origin (same-origin)
        if !origin_str.is_empty()
            && !origin_str.contains("localhost")
            && !origin_str.contains("127.0.0.1")
        {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json_rpc_err(None, -32001, "Invalid origin")),
            )
                .into_response());
        }
    }
    Ok(())
}

// ── MCP POST Handler (JSON-RPC 2.0) ────────────────────────────────────

async fn mcp_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<JsonRpcRequest>,
) -> Response {
    // Origin validation (DNS rebinding protection)
    if let Err(resp) = validate_origin(&headers) {
        return resp;
    }

    // Check protocol version on non-initialize requests
    if req.method != "initialize" {
        if let Some(session_id) = headers.get("mcp-session-id").and_then(|v| v.to_str().ok()) {
            if !state.sessions.contains_key(session_id) {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json_rpc_err(req.id, -32002, "Session expired or unknown")),
                )
                    .into_response();
            }
        }
    }

    let response = match req.method.as_str() {
        // ── Lifecycle ───────────────────────────────────────────────
        "initialize" => {
            let session_id = Uuid::new_v4().to_string();
            state.sessions.insert(
                session_id.clone(),
                McpSession {
                    id: session_id.clone(),
                    protocol_version: MCP_PROTOCOL_VERSION.to_string(),
                    created_at: Instant::now(),
                    initialized: false,
                },
            );

            let result = serde_json::json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {
                    "tools": { "listChanged": true },
                    "resources": { "subscribe": false, "listChanged": false },
                    "logging": {},
                },
                "serverInfo": {
                    "name": SERVER_NAME,
                    "version": SERVER_VERSION,
                },
            });

            // Return with Mcp-Session-Id header
            let body = json_rpc_ok(req.id, result);
            let mut resp = (StatusCode::OK, Json(body)).into_response();
            resp.headers_mut()
                .insert("mcp-session-id", session_id.parse().unwrap());
            return resp;
        }

        "notifications/initialized" | "initialized" => {
            if let Some(sid) = headers.get("mcp-session-id").and_then(|v| v.to_str().ok()) {
                if let Some(mut session) = state.sessions.get_mut(sid) {
                    session.initialized = true;
                }
            }
            return StatusCode::ACCEPTED.into_response();
        }

        // ── Tools ───────────────────────────────────────────────────
        "tools/list" => {
            let tools = build_tools_list();
            json_rpc_ok(req.id, serde_json::json!({ "tools": tools }))
        }

        "tools/call" => {
            let params = req.params.unwrap_or_default();
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or_default();

            match execute_tool(&state, tool_name, args).await {
                Ok(content) => json_rpc_ok(
                    req.id,
                    serde_json::json!({
                        "content": [{ "type": "text", "text": content.to_string() }]
                    }),
                ),
                Err(e) => json_rpc_err(req.id, -32000, e),
            }
        }

        // ── Resources ───────────────────────────────────────────────
        "resources/list" => json_rpc_ok(
            req.id,
            serde_json::json!({
                "resources": [
                    {
                        "uri": "memory://workspace/analysis",
                        "name": "Workspace Chaos Analysis",
                        "description": "Real-time chaos-math analysis of current workspace",
                        "mimeType": "application/json"
                    },
                    {
                        "uri": "memory://ffi/status",
                        "name": "FFI Bridge Status",
                        "description": "Status of all FFI language bridges (Zig, Julia, Mojo, Pony, JAX)",
                        "mimeType": "application/json"
                    },
                    {
                        "uri": "memory://engines/status",
                        "name": "Search Engines Status",
                        "description": "Status and availability of all search engines",
                        "mimeType": "application/json"
                    }
                ]
            }),
        ),

        "resources/read" => {
            let uri = req
                .params
                .as_ref()
                .and_then(|p| p.get("uri"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let content = match uri {
                "memory://workspace/analysis" => state.analyze_workspace_chaos(),
                "memory://ffi/status" => {
                    serde_json::to_value(&state.ffi_status).unwrap_or_default()
                }
                "memory://engines/status" => serde_json::json!({
                    "engines": state.info.engines,
                    "count": state.info.engines.len(),
                }),
                _ => {
                    return (
                        StatusCode::OK,
                        Json(json_rpc_err(
                            req.id,
                            -32002,
                            format!("Unknown resource: {uri}"),
                        )),
                    )
                        .into_response()
                }
            };

            json_rpc_ok(
                req.id,
                serde_json::json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": content.to_string()
                    }]
                }),
            )
        }

        // ── Ping ────────────────────────────────────────────────────
        "ping" => json_rpc_ok(req.id, serde_json::json!({})),

        _ => json_rpc_err(req.id, -32601, format!("Method not found: {}", req.method)),
    };

    (StatusCode::OK, Json(response)).into_response()
}

// ── MCP GET Handler (SSE Stream) ────────────────────────────────────────

async fn mcp_get(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if let Err(resp) = validate_origin(&headers) {
        return resp;
    }

    // Check session
    if let Some(sid) = headers.get("mcp-session-id").and_then(|v| v.to_str().ok()) {
        if !state.sessions.contains_key(sid) {
            return StatusCode::NOT_FOUND.into_response();
        }
    }

    let mut rx = state.notification_tx.subscribe();

    let stream = async_stream::stream! {
        // Prime event with empty data (per spec)
        yield Ok::<_, Infallible>(axum::response::sse::Event::default()
            .id(Uuid::new_v4().to_string())
            .data(""));

        loop {
            match rx.recv().await {
                Ok(msg) => {
                    yield Ok(axum::response::sse::Event::default()
                        .id(Uuid::new_v4().to_string())
                        .data(msg));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)))
        .into_response()
}

// ── MCP DELETE Handler (Session Termination) ────────────────────────────

async fn mcp_delete(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    if let Some(sid) = headers.get("mcp-session-id").and_then(|v| v.to_str().ok()) {
        if state.sessions.remove(sid).is_some() {
            return StatusCode::OK.into_response();
        }
        return StatusCode::NOT_FOUND.into_response();
    }
    StatusCode::BAD_REQUEST.into_response()
}

// ── Tool Definitions ────────────────────────────────────────────────────

fn build_tools_list() -> Vec<serde_json::Value> {
    vec![
        tool_def(
            "workspace_scan",
            "Scan workspace using chaos-math analysis (entropy, Lyapunov exponent)",
            serde_json::json!({
                "type": "object",
                "properties": { "path": { "type": "string", "description": "Workspace path to analyze" } }
            }),
        ),
        tool_def(
            "search_text",
            "Full-text search using Tantivy engine",
            serde_json::json!({
                "type": "object",
                "properties": { "text": { "type": "string" }, "limit": { "type": "integer" } },
                "required": ["text"]
            }),
        ),
        tool_def(
            "search_vector",
            "Vector similarity search using Qdrant engine",
            serde_json::json!({
                "type": "object",
                "properties": { "vector": { "type": "array", "items": { "type": "number" } }, "limit": { "type": "integer" } },
                "required": ["vector"]
            }),
        ),
        tool_def(
            "search_hybrid",
            "Combined text + vector search across engines",
            serde_json::json!({
                "type": "object",
                "properties": { "text": { "type": "string" }, "vector": { "type": "array" }, "limit": { "type": "integer" } }
            }),
        ),
        tool_def(
            "chaos_analysis",
            "Mathematical chaos analysis (Lyapunov, entropy) on numeric data",
            serde_json::json!({
                "type": "object",
                "properties": { "data": { "type": "array", "items": { "type": "number" } } },
                "required": ["data"]
            }),
        ),
        tool_def(
            "predict_next_moves",
            "Predict next agent moves using JAX inference + chaos math",
            serde_json::json!({
                "type": "object",
                "properties": { "context": { "type": "array", "items": { "type": "number" } }, "n_moves": { "type": "integer" } },
                "required": ["context"]
            }),
        ),
        tool_def(
            "decision_analyze",
            "Analyze a situation and recommend optimal strategy using entropy/chaos/stability",
            serde_json::json!({
                "type": "object",
                "properties": { "situation": { "type": "string" }, "data": { "type": "array", "items": { "type": "number" } } },
                "required": ["situation"]
            }),
        ),
        tool_def(
            "ffi_status",
            "Get status of all FFI language bridges (Zig, Julia, Mojo, Pony, JAX)",
            serde_json::json!({
                "type": "object", "properties": {}
            }),
        ),
        tool_def(
            "list_engines",
            "List all available search engines and their status",
            serde_json::json!({
                "type": "object", "properties": {}
            }),
        ),
        tool_def(
            "mojo_dot_product",
            "Compute dot product using real Mojo SIMD kernels",
            serde_json::json!({
                "type": "object",
                "properties": { "a": { "type": "array", "items": { "type": "number" } }, "b": { "type": "array", "items": { "type": "number" } } },
                "required": ["a", "b"]
            }),
        ),
        tool_def(
            "mojo_cosine_similarity",
            "Compute cosine similarity using real Mojo SIMD kernels",
            serde_json::json!({
                "type": "object",
                "properties": { "a": { "type": "array", "items": { "type": "number" } }, "b": { "type": "array", "items": { "type": "number" } } },
                "required": ["a", "b"]
            }),
        ),
        tool_def(
            "zig_shared_buffer",
            "Create/manage Zig shared memory buffers for zero-copy data transfer",
            serde_json::json!({
                "type": "object",
                "properties": { "operation": { "type": "string", "enum": ["create", "write", "read", "info"] }, "capacity": { "type": "integer" } },
                "required": ["operation"]
            }),
        ),
        tool_def(
            "julia_entropy",
            "Calculate Shannon entropy of data distribution",
            serde_json::json!({
                "type": "object",
                "properties": { "data": { "type": "array", "items": { "type": "number" } } },
                "required": ["data"]
            }),
        ),
        tool_def(
            "memory_stats",
            "Get shared memory system statistics for multi-agent coordination",
            serde_json::json!({
                "type": "object", "properties": {}
            }),
        ),
    ]
}

fn tool_def(name: &str, description: &str, input_schema: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
    })
}

// ── Tool Execution ──────────────────────────────────────────────────────

pub async fn execute_tool(
    state: &AppState,
    name: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    match name {
        "workspace_scan" => Ok(state.analyze_workspace_chaos()),

        "chaos_analysis" => {
            let data: Vec<f64> = args
                .get("data")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|n| n.as_f64()).collect())
                .ok_or("chaos_analysis requires 'data' array")?;
            let chaos = crate::ffi::julia::chaos_analysis(&data).map_err(|e| format!("{e}"))?;
            let entropy = crate::ffi::julia::shannon_entropy(&data);
            let (mean, variance, std_dev) =
                crate::ffi::julia::analyze_vector(&data).map_err(|e| format!("{e}"))?;
            let decision = crate::ffi::julia::get_search_decision(entropy, chaos, 0.5)
                .unwrap_or_else(|_| "sequential".into());
            Ok(serde_json::json!({
                "lyapunov_exponent": chaos,
                "entropy": entropy,
                "mean": mean, "variance": variance, "std_dev": std_dev,
                "recommended_strategy": decision,
            }))
        }

        "predict_next_moves" => {
            let ctx: Vec<f32> = args
                .get("context")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|n| n.as_f64().map(|f| f as f32))
                        .collect()
                })
                .ok_or("predict_next_moves requires 'context' array")?;
            let n = args.get("n_moves").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
            if !state.ffi_status.jax {
                return Err("JAX FFI is not available on this runtime".to_string());
            }
            match crate::ffi::jax::predict_next_moves(&ctx, n) {
                Ok(moves) => Ok(serde_json::json!({
                    "predictions": moves,
                    "backend": "jax_ffi",
                })),
                Err(e) => Err(format!("JAX prediction failed: {e}")),
            }
        }

        "decision_analyze" => {
            let situation = args
                .get("situation")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let context_data: std::collections::HashMap<String, String> = args
                .get("data")
                .and_then(|v| v.as_object())
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| (k.clone(), v.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let engine = crate::decision_logic::DecisionEngine::new();
            match engine.analyze_decision(situation, &context_data).await {
                Ok(rationale) => Ok(serde_json::to_value(&rationale).unwrap_or_default()),
                Err(e) => Err(format!("Decision analysis failed: {e}")),
            }
        }

        "ffi_status" => Ok(serde_json::to_value(&state.ffi_status).unwrap_or_default()),

        "list_engines" => Ok(serde_json::json!({
            "engines": state.info.engines,
            "count": state.info.engines.len(),
        })),

        "mojo_dot_product" => {
            if !state.ffi_status.mojo {
                return Err("Mojo FFI is not available on this runtime".to_string());
            }
            let a = extract_f64_array(&args, "a")?;
            let b = extract_f64_array(&args, "b")?;
            let result = crate::ffi::mojo::dot_product(&a, &b).map_err(|e| format!("{e}"))?;
            Ok(serde_json::json!({
                "result": result,
                "backend": "mojo_ffi",
            }))
        }

        "mojo_cosine_similarity" => {
            if !state.ffi_status.mojo {
                return Err("Mojo FFI is not available on this runtime".to_string());
            }
            let a = extract_f64_array(&args, "a")?;
            let b = extract_f64_array(&args, "b")?;
            let result = crate::ffi::mojo::cosine_similarity(&a, &b).map_err(|e| format!("{e}"))?;
            Ok(serde_json::json!({
                "result": result,
                "backend": "mojo_ffi",
            }))
        }

        "zig_shared_buffer" => {
            let op = args
                .get("operation")
                .and_then(|v| v.as_str())
                .unwrap_or("info");
            let cap = args
                .get("capacity")
                .and_then(|v| v.as_u64())
                .unwrap_or(4096) as usize;
            match op {
                "create" | "info" => {
                    let buf = crate::ffi::zig::ZigBridge::new(cap).map_err(|e| format!("{e}"))?;
                    let info = buf.get_info();
                    Ok(serde_json::json!({
                        "capacity": info.capacity,
                        "used": info.used,
                        "available": info.available,
                        "ref_count": info.ref_count,
                        "initialized": info.initialized,
                    }))
                }
                _ => Err(format!("Unknown buffer operation: {op}")),
            }
        }

        "julia_entropy" => {
            let data = extract_f64_array(&args, "data")?;
            let entropy = crate::ffi::julia::shannon_entropy(&data);
            let chaos = crate::ffi::julia::chaos_analysis(&data).unwrap_or(0.0);
            Ok(serde_json::json!({ "entropy": entropy, "lyapunov": chaos }))
        }

        "search_text" => {
            let text = args
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or("search_text requires 'text' parameter")?;
            let limit = args
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize);
            match crate::routes::execute_text_search(text, limit).await {
                Ok(result) => Ok(result),
                Err(e) => Err(format!("Text search failed: {e}")),
            }
        }

        "search_vector" => {
            let vector = extract_f32_array(&args, "vector")?;
            let limit = args
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize);
            match crate::routes::execute_vector_search(vector, limit).await {
                Ok(result) => Ok(result),
                Err(e) => Err(format!("Vector search failed: {e}")),
            }
        }

        "search_hybrid" => {
            let text = args
                .get("text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let vector = args.get("vector").and_then(|v| v.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|n| n.as_f64().map(|f| f as f32))
                    .collect()
            });
            let limit = args
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize);
            match crate::routes::execute_hybrid_search(text, vector, limit).await {
                Ok(result) => Ok(result),
                Err(e) => Err(format!("Hybrid search failed: {e}")),
            }
        }

        "memory_stats" => Ok(serde_json::json!({
            "sessions_active": state.sessions.len(),
            "server_uptime_secs": state.sessions.iter().next().map(|s| s.created_at.elapsed().as_secs()).unwrap_or(0),
        })),

        "predict_trajectory" => {
            let history: Vec<String> = args
                .get("history")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(|x| x.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            let steps = args.get("steps").and_then(|v| v.as_u64()).unwrap_or(3) as usize;

            let mut engine = crate::prediction_engine::PredictionEngine::new();
            let mut working_history = history.clone();
            let mut trajectory = Vec::with_capacity(steps);

            for _ in 0..steps {
                let (next_move, lyapunov, entropy) = engine
                    .predict_next_move(&working_history)
                    .await
                    .map_err(|e| format!("{e}"))?;
                engine.update_metrics(lyapunov, engine.get_correlation_dimension(), entropy);
                working_history.push(next_move.clone());
                trajectory.push(serde_json::json!({
                    "move": next_move,
                    "lyapunov": lyapunov,
                    "entropy": entropy,
                    "strategy": if lyapunov > 0.5 {
                        "DiscreteExploration"
                    } else if lyapunov < -0.2 {
                        "LocalExploitation"
                    } else {
                        "AdaptiveSearch"
                    },
                }));
            }

            Ok(serde_json::json!({
                "input_history": history,
                "trajectory": trajectory,
                "count": steps,
                "final_metrics": engine.get_chaos_metrics(),
            }))
        }

        "motor_route_query" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or("motor_route_query requires 'query' parameter")?;
            let query_type = args
                .get("query_type")
                .and_then(|v| v.as_str())
                .unwrap_or("hybrid");

            let router = crate::motores::core::routing_ai::RoutingAI::new();
            let route_query = crate::motores::core::types::SearchQuery {
                text: query.to_string(),
                vector: None,
                query_type: match query_type {
                    "semantic" => crate::motores::core::types::QueryType::Vector,
                    "exact" => crate::motores::core::types::QueryType::Term,
                    "fuzzy" => crate::motores::core::types::QueryType::Fuzzy,
                    _ => crate::motores::core::types::QueryType::Hybrid,
                },
                limit: args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize,
                offset: 0,
                filters: HashMap::new(),
                min_score: 0.0,
            };
            let selections = router.route_query(&route_query);
            let recommended_motors: Vec<String> = selections
                .iter()
                .map(|selection| match selection {
                    crate::motores::core::types::EngineSelection::Primary(name)
                    | crate::motores::core::types::EngineSelection::Fallback(name)
                    | crate::motores::core::types::EngineSelection::Secondary(name)
                    | crate::motores::core::types::EngineSelection::Distributed(name)
                    | crate::motores::core::types::EngineSelection::Comparison(name)
                    | crate::motores::core::types::EngineSelection::Mathematical(name)
                    | crate::motores::core::types::EngineSelection::Semantic(name) => {
                        (*name).to_string()
                    }
                })
                .collect();

            Ok(serde_json::json!({
                "query": query,
                "query_type": query_type,
                "recommended_motors": recommended_motors,
                "fallback": recommended_motors.get(1).cloned().unwrap_or_else(|| "qdrant".to_string()),
            }))
        }

        "memory_persist" => {
            let session_id = args
                .get("session_id")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string())
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            let data = args.get("data").cloned().unwrap_or(serde_json::json!({}));

            let persistence = crate::motores::persistence::SledPersistence::new("mcp_sessions")
                .map_err(|e| format!("Persistence init failed: {e}"))?;
            persistence
                .save_data(&session_id, &data)
                .await
                .map_err(|e| format!("Persistence write failed: {e}"))?;

            Ok(serde_json::json!({
                "session_id": session_id,
                "data": data,
                "stored": true,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "backend": "sled:mcp_sessions",
            }))
        }

        "analysis_workspace_detailed" => {
            let analysis = state.analyze_workspace_chaos();
            let detailed = serde_json::json!({
                "basic": analysis,
                "extensions_detail": analysis
                    .get("extensions")
                    .cloned()
                    .unwrap_or_default(),
                "chaos_summary": {
                    "is_chaotic": analysis
                        .get("chaos_metrics")
                        .and_then(|m| m.get("lyapunov_exponent"))
                        .and_then(|l| l.as_f64())
                        .map(|l| l > 0.5)
                        .unwrap_or(false),
                },
            });
            Ok(detailed)
        }

        "ffi_test_all" => {
            // Test all FFI subsystems with real operations
            let test_vector = vec![1.0, 2.0, 3.0, 4.0, 5.0];

            let mojo_result = if state.ffi_status.mojo {
                crate::ffi::mojo::dot_product(&test_vector, &test_vector)
                    .map_err(|e| format!("Mojo: {e}"))
                    .ok()
            } else {
                None
            };

            let julia_entropy = crate::ffi::julia::shannon_entropy(&test_vector);
            let julia_chaos = if state.ffi_status.julia {
                crate::ffi::julia::chaos_analysis(&test_vector)
                    .map_err(|e| format!("Julia: {e}"))
                    .ok()
            } else {
                None
            };

            Ok(serde_json::json!({
                "mojo_dot_product": mojo_result,
                "julia_entropy": julia_entropy,
                "julia_chaos": julia_chaos,
                "ffi_status": state.ffi_status,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        }

        _ => Err(format!("Unknown tool: {name}")),
    }
}

fn extract_f64_array(args: &serde_json::Value, key: &str) -> Result<Vec<f64>, String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|n| n.as_f64()).collect())
        .ok_or_else(|| format!("Required array field: '{key}'"))
}

fn extract_f32_array(args: &serde_json::Value, key: &str) -> Result<Vec<f32>, String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|n| n.as_f64().map(|f| f as f32))
                .collect()
        })
        .ok_or_else(|| format!("Required array field: '{key}'"))
}

// ── Health Check ────────────────────────────────────────────────────────

async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "operational",
        "version": SERVER_VERSION,
        "protocol": MCP_PROTOCOL_VERSION,
        "timestamp": chrono::Local::now().to_rfc3339(),
        "sessions_active": state.sessions.len(),
        "ffi": state.ffi_status,
    }))
}

// ── Server Start ────────────────────────────────────────────────────────

pub async fn start_http_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let (notification_tx, _) = broadcast::channel::<String>(256);

    let ffi_status = AppState::detect_ffi_status().await;
    tracing::info!(
        "FFI Status: zig={}, julia={}, mojo={}, pony={}, jax={}",
        ffi_status.zig,
        ffi_status.julia,
        ffi_status.mojo,
        ffi_status.pony,
        ffi_status.jax
    );

    let workspace_root = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    let app_state = Arc::new(AppState {
        info: MCPInfo {
            name: format!("{SERVER_NAME} v{SERVER_VERSION} MCP"),
            version: SERVER_VERSION.to_string(),
            capabilities: vec![
                "hybrid-search".into(),
                "vector-search".into(),
                "text-search".into(),
                "ffi-kernels".into(),
                "chaos-math".into(),
                "prediction".into(),
                "workspace-scan".into(),
                "shared-memory".into(),
                "auto-gestion".into(),
                "multi-agent".into(),
            ],
            engines: vec![
                "qdrant".into(),
                "faiss".into(),
                "scann".into(),
                "tantivy".into(),
                "lnx".into(),
                "toshi".into(),
                "meilisearch".into(),
                "julia_nlp".into(),
                "memory_bank".into(),
            ],
            tools: vec![
                "workspace_scan".into(),
                "search_text".into(),
                "search_vector".into(),
                "search_hybrid".into(),
                "chaos_analysis".into(),
                "predict_next_moves".into(),
                "predict_trajectory".into(),
                "decision_analyze".into(),
                "motor_route_query".into(),
                "ffi_status".into(),
                "ffi_test_all".into(),
                "list_engines".into(),
                "mojo_dot_product".into(),
                "mojo_cosine_similarity".into(),
                "zig_shared_buffer".into(),
                "julia_entropy".into(),
                "memory_stats".into(),
                "memory_persist".into(),
                "analysis_workspace_detailed".into(),
            ],
        },
        ffi_status,
        sessions: DashMap::new(),
        notification_tx: notification_tx.clone(),
        workspace_root,
    });

    // Auto-gestión: startup workspace scan
    let startup_state = app_state.clone();
    tokio::spawn(async move {
        let analysis = startup_state.analyze_workspace_chaos();
        tracing::info!(
            "Auto-gestión startup scan: {} files, entropy={:.4}, chaos={:.4}",
            analysis
                .get("total_files")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            analysis
                .get("chaos_metrics")
                .and_then(|m| m.get("entropy"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            analysis
                .get("chaos_metrics")
                .and_then(|m| m.get("lyapunov_exponent"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        );
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/mcp", post(mcp_post))
        .route("/mcp", get(mcp_get))
        .route("/mcp", delete(mcp_delete))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  MEMORY_P v{SERVER_VERSION} - MCP Streamable HTTP Server          ║");
    println!("║  Protocol: {MCP_PROTOCOL_VERSION}                             ║");
    println!("║  Transport: Streamable HTTP (POST/GET/DELETE /mcp)      ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("  Listening: http://127.0.0.1:{port}");
    println!("  GET  /health  - Health + FFI status");
    println!("  POST /mcp     - JSON-RPC 2.0 (MCP requests)");
    println!("  GET  /mcp     - SSE stream (server notifications)");
    println!("  DELETE /mcp   - Session termination");
    println!();
    println!("  14 tools registered | 9 search engines | 5 FFI bridges");

    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_list_count() {
        let tools = build_tools_list();
        assert!(
            tools.len() >= 14,
            "Expected at least 14 tools, got {}",
            tools.len()
        );
    }

    #[test]
    fn test_json_rpc_ok() {
        let resp = json_rpc_ok(
            Some(serde_json::json!(1)),
            serde_json::json!({"test": true}),
        );
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_json_rpc_err() {
        let resp = json_rpc_err(Some(serde_json::json!(1)), -32601, "Method not found");
        assert!(resp.error.is_some());
        assert_eq!(resp.error.as_ref().map(|e| e.code), Some(-32601));
    }

    #[test]
    fn test_ffi_status_serialization() {
        let status = FFIStatus {
            zig: true,
            julia: false,
            mojo: true,
            pony: false,
            jax: false,
        };
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["zig"], true);
    }
}
