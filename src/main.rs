//! main.rs - MCP Toolkit HTTP 2025
//! Servidor Axum + MCP para análisis, edición y reparación masiva
//!
//! Características:
//! - Protocolo MCP oficialmente compatible
//! - Procesamiento paralelo con Rayon
//! - 100% Rust puro sin dependencias nativas
//! - Endpoints: /analyze, /edit, /repair

use axum::Router;
use mimalloc::MiMalloc;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Importar desde la librería
use memory_p::*;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--stdio".to_string()) || std::env::var("MCP_STDIO").is_ok() {
        if let Err(e) = mcp_stdio_mode().await {
            eprintln!("❌ Error en modo stdio: {}", e);
        }
    } else {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_max_level(tracing::Level::INFO)
            .with_target(true)
            .init();

        if let Err(e) = http_server_mode().await {
            tracing::error!("❌ Error en servidor HTTP: {}", e);
        }
    }
}

async fn http_server_mode() -> error::Result<()> {
    tracing::info!("╔══════════════════════════════════════════════════╗");
    tracing::info!("║  MEMORY_P MCP Server 2026 - ALWAYS-ON EDITION   ║");
    tracing::info!("╚══════════════════════════════════════════════════╝");

    let shared_memory = Arc::new(shared_memory::SharedMemorySystem::new().await?);
    let auto_manager = Arc::new(auto_manager::AutoManager::new(auto_manager::ManagerConfig::default()));

    if let Err(e) = auto_manager.auto_start(shared_memory.clone()).await {
        tracing::error!("❌ Error al iniciar AutoManager: {}", e);
    }

    let kpi_tracker = Arc::new(kpi_tracker::KpiTracker::new(kpi_tracker::KpiConfig::default()));
    let _ = kpi_tracker.start().await;

    let prediction_engine = Arc::new(prediction_engine::PredictionEngine::new(prediction_engine::PredictionConfig::default()));

    let app = Router::new()
        .merge(mcp_api::routes())
        .fallback(error_404)
        .layer(axum::Extension(auto_manager.clone()))
        .layer(axum::Extension(kpi_tracker.clone()))
        .layer(axum::Extension(shared_memory.clone()))
        .layer(axum::Extension(prediction_engine.clone()));

    let addr = SocketAddr::from(([127, 0, 0, 1], 4040));
    let listener = TcpListener::bind(addr).await.map_err(|e| error::MemoryPError::Io(e))?;

    axum::serve(listener, app).await.map_err(|e| error::MemoryPError::Io(e))?;
    Ok(())
}

async fn mcp_stdio_mode() -> error::Result<()> {
    use crate::mcp::models::JsonRpcRequest;
    use crate::mcp_api::mcp_json_rpc_handler;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    let shared_memory = Arc::new(shared_memory::SharedMemorySystem::new().await?);
    let prediction_engine = Arc::new(prediction_engine::PredictionEngine::new(prediction_engine::PredictionConfig::default()));

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();
    let mut line = String::new();

    while stdin.read_line(&mut line).await? > 0 {
        if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&line) {
            let response = mcp_json_rpc_handler(
                axum::extract::Extension(shared_memory.clone()),
                axum::extract::Extension(prediction_engine.clone()),
                axum::Json(req),
            ).await;
            let resp_json = serde_json::to_string(&response.0).map_err(error::MemoryPError::Json)?;
            stdout.write_all(format!("{}\n", resp_json).as_bytes()).await?;
            stdout.flush().await?;
        }
        line.clear();
    }
    Ok(())
}

async fn error_404() -> (axum::http::StatusCode, String) {
    (axum::http::StatusCode::NOT_FOUND, "Not Found".to_string())
}
