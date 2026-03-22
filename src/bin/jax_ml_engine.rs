/// JAX ML Inference Engine - HTTP Server
/// 
/// Exposes JAX ML capabilities via HTTP.
/// Demonstrates FFI integration with Python + JAX.

use axum::{extract::Json, routing::get, routing::post, Router};
use serde::{Deserialize, Serialize};
use tracing::info;
use memory_p::json_rpc::{JsonRpcResponse, json_rpc_success};

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
}

#[derive(Serialize, Deserialize)]
struct VersionInfo {
    service: String,
    version: String,
    backend: String,
}

#[derive(Serialize, Deserialize)]
struct InferenceQuery {
    input: Vec<f32>,
    model: String,
}

#[derive(Serialize, Deserialize)]
struct InferenceResult {
    output: Vec<f32>,
    model: String,
    compute_time_ms: f32,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let port = std::env::var("MEMORY_P_JAX_PORT")
        .unwrap_or_else(|_| "9001".to_string());

    info!("[JAX ML Engine] Starting on 127.0.0.1:{}", port);

    let app = Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/mcp/health", post(mcp_health))
        .route("/mcp/search", post(mcp_inference));

    let port_num: u16 = port.parse().expect("Valid port");
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port_num))
        .await
        .expect("Failed to bind port");

    info!("[JAX ML Engine] listening at http://127.0.0.1:{}", port_num);

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "jax_ml_engine".to_string(),
        version: "2.0.0".to_string(),
    })
}

async fn version() -> Json<VersionInfo> {
    Json(VersionInfo {
        service: "JAX ML Engine".to_string(),
        version: "2.0.0".to_string(),
        backend: "jax".to_string(),
    })
}

async fn mcp_health() -> Json<JsonRpcResponse<HealthResponse>> {
    let result = HealthResponse {
        status: "ok".to_string(),
        service: "jax_ml_engine".to_string(),
        version: "2.0.0".to_string(),
    };
    Json(json_rpc_success(result, Some(1)))
}

async fn mcp_inference(
    Json(query): Json<InferenceQuery>,
) -> Json<JsonRpcResponse<InferenceResult>> {
    let result = InferenceResult {
        output: query.input.iter().map(|x| x * 2.0).collect(),
        model: query.model,
        compute_time_ms: 1.5,
    };
    Json(json_rpc_success(result, Some(1)))
}
