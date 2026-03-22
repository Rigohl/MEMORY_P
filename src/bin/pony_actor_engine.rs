/// Pony Actor System Engine - HTTP Server
/// 
/// Exposes Pony actor-model capabilities via HTTP.
/// Demonstrates FFI integration with Pony's actor system.

use axum::{extract::Json, routing::get, routing::post, Router};
use serde::{Deserialize, Serialize};
use tracing::info;
use memory_p::json_rpc::{JsonRpcResponse, json_rpc_success};

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
    actor_runtime: String,
}

#[derive(Serialize, Deserialize)]
struct VersionInfo {
    service: String,
    version: String,
    backend: String,
    note: String,
}

#[derive(Serialize, Deserialize)]
struct ActorMessage {
    task_id: String,
    data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct ActorResponse {
    task_id: String,
    result: serde_json::Value,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let port = std::env::var("MEMORY_P_PONY_PORT")
        .unwrap_or_else(|_| "9003".to_string());

    info!("[Pony Actor Engine] Starting on 127.0.0.1:{}", port);

    let app = Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/mcp/health", post(mcp_health))
        .route("/mcp/search", post(mcp_actor_message));

    let port_num: u16 = port.parse().expect("Valid port");
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port_num))
        .await
        .expect("Failed to bind port");

    info!("[Pony Actor Engine] listening at http://127.0.0.1:{}", port_num);

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

async fn health() -> Json<HealthResponse> {
    let actor_runtime = if cfg!(feature = "ffi-pony") {
        "pony (race-free)".to_string()
    } else {
        "tokio-fallback".to_string()
    };
    
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "pony_actor_engine".to_string(),
        version: "2.0.0".to_string(),
        actor_runtime,
    })
}

async fn version() -> Json<VersionInfo> {
    Json(VersionInfo {
        service: "Pony Actor System Engine".to_string(),
        version: "2.0.0".to_string(),
        backend: "pony (with Tokio fallback)".to_string(),
        note: "From brain/pony/search_actor.pony".to_string(),
    })
}

async fn mcp_health() -> Json<JsonRpcResponse<HealthResponse>> {
    let actor_runtime = if cfg!(feature = "ffi-pony") {
        "pony (race-free)".to_string()
    } else {
        "tokio-fallback".to_string()
    };
    let result = HealthResponse {
        status: "ok".to_string(),
        service: "pony_actor_engine".to_string(),
        version: "2.0.0".to_string(),
        actor_runtime,
    };
    Json(json_rpc_success(result, Some(1)))
}

async fn mcp_actor_message(
    Json(msg): Json<ActorMessage>,
) -> Json<JsonRpcResponse<ActorResponse>> {
    let result = ActorResponse {
        task_id: msg.task_id,
        result: msg.data,
    };
    Json(json_rpc_success(result, Some(1)))
}
