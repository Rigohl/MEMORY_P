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
use tokio::net::TcpListener;
use tracing_subscriber;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod accelerator_bridge;
mod analyzer;
mod config;
mod error;
mod mcp;
mod mcp_api;
mod mega_simulator; // 3-phase mega simulation engine
mod parallel_engine;
mod simulation_engine; // Legacy native engine
mod workspace;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--stdio".to_string()) || std::env::var("MCP_STDIO").is_ok() {
        // En modo stdio, NO enviamos nada a stdout excepto JSON puro.
        if let Err(e) = mcp_stdio_mode().await {
            eprintln!("❌ Error en modo stdio: {}", e);
        }
    } else {
        // Inicializar logging solo en modo HTTP
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

async fn http_server_mode() -> crate::error::Result<()> {
    // Construir router
    let app = Router::new().merge(mcp_api::routes()).fallback(error_404);

    let addr = SocketAddr::from(([127, 0, 0, 1], 4040));

    tracing::info!("🚀 MCP Toolkit HTTP iniciando");
    tracing::info!(
        "📡 Escuchando en http://{}:{} (MCP Protocol 2024-11-05)",
        addr.ip(),
        addr.port()
    );

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| crate::error::MemoryPError::Io(e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| crate::error::MemoryPError::Io(e))?;

    Ok(())
}

async fn mcp_stdio_mode() -> crate::error::Result<()> {
    use crate::mcp::models::JsonRpcRequest;
    use crate::mcp_api::mcp_json_rpc_handler;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    tracing::info!("✅ MEMORY_P MCP Stdio listo");

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();
    let mut line = String::new();

    while stdin.read_line(&mut line).await? > 0 {
        if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&line) {
            let response = mcp_json_rpc_handler(axum::Json(req)).await;
            let resp_json =
                serde_json::to_string(&response.0).map_err(crate::error::MemoryPError::Json)?;
            stdout
                .write_all(format!("{}\n", resp_json).as_bytes())
                .await?;
            stdout.flush().await?;
        }
        line.clear();
    }
    Ok(())
}

/// Manejador para rutas no encontradas
async fn error_404() -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::NOT_FOUND,
        "Endpoint no encontrado. Usa: /analyze, /edit, /repair, /status".to_string(),
    )
}
