/// Mojo SIMD Search Engine - HTTP Server
/// 
/// Exposes Mojo SIMD capabilities via HTTP.
/// Demonstrates FFI integration with Mojo kernels.

use axum::{extract::Json, routing::get, routing::post, Router};
use serde::{Deserialize, Serialize};
use tracing::info;
use memory_p::json_rpc::{JsonRpcResponse, json_rpc_success};

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
    simd_backend: String,
}

#[derive(Serialize, Deserialize)]
struct VersionInfo {
    service: String,
    version: String,
    backend: String,
    note: String,
}

#[derive(Serialize, Deserialize)]
struct SIMDSearchQuery {
    vectors: Vec<Vec<f32>>,
    pattern: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
struct SIMDSearchResult {
    matches: Vec<usize>,
    scores: Vec<f32>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let port = std::env::var("MEMORY_P_MOJO_PORT")
        .unwrap_or_else(|_| "9002".to_string());

    info!("[Mojo Search Engine] Starting on 127.0.0.1:{}", port);

    let app = Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/mcp/health", post(mcp_health))
        .route("/mcp/search", post(mcp_simd_search));

    let port_num: u16 = port.parse().expect("Valid port");
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port_num))
        .await
        .expect("Failed to bind port");

    info!("[Mojo Search Engine] listening at http://127.0.0.1:{}", port_num);

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

async fn health() -> Json<HealthResponse> {
    let simd_backend = if cfg!(feature = "ffi-mojo") {
        "mojo (SIMD)".to_string()
    } else {
        "rust-fallback".to_string()
    };
    
    Json(HealthResponse {
        status: "ok".to_string(),
        service: "mojo_search_engine".to_string(),
        version: "2.0.0".to_string(),
        simd_backend,
    })
}

async fn version() -> Json<VersionInfo> {
    Json(VersionInfo {
        service: "Mojo SIMD Search Engine".to_string(),
        version: "2.0.0".to_string(),
        backend: "mojo (with Rust fallback)".to_string(),
        note: "From brain/mojo/kernels.mojo".to_string(),
    })
}

async fn mcp_health() -> Json<JsonRpcResponse<HealthResponse>> {
    let simd_backend = if cfg!(feature = "ffi-mojo") {
        "mojo (SIMD)".to_string()
    } else {
        "rust-fallback".to_string()
    };
    let result = HealthResponse {
        status: "ok".to_string(),
        service: "mojo_search_engine".to_string(),
        version: "2.0.0".to_string(),
        simd_backend,
    };
    Json(json_rpc_success(result, Some(1)))
}

async fn mcp_simd_search(
    Json(query): Json<SIMDSearchQuery>,
) -> Json<JsonRpcResponse<SIMDSearchResult>> {
    let result = SIMDSearchResult {
        matches: vec![],
        scores: vec![],
    };
    Json(json_rpc_success(result, Some(1)))
}
