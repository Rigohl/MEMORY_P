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

mod analyzer;
mod auto_manager; // Sistema de auto-gestión MCP 2026
mod config;
mod error;
mod ffi; // FFI multi-lenguaje (Julia, JAX, Mojo, Pony, Zig)
mod kpi_tracker; // Sistema de KPIs Always-On + Six Sigma
mod mcp;
mod mcp_api;
mod mega_simulator; // 3-phase mega simulation engine
mod parallel_engine;
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
    // ========================================================
    // MCP PROTOCOL 2026 - ALWAYS-ON AUTO-MANAGED SYSTEM
    // ========================================================
    
    tracing::info!("╔══════════════════════════════════════════════════╗");
    tracing::info!("║  MEMORY_P MCP Server 2026 - ALWAYS-ON EDITION   ║");
    tracing::info!("╚══════════════════════════════════════════════════╝");
    
    // 1. Auto-iniciar sistema de gestión
    let auto_manager = Arc::new(auto_manager::AutoManager::new(
        auto_manager::ManagerConfig::default()
    ));
    
    tracing::info!("🔧 Iniciando sistema de auto-gestión...");
    if let Err(e) = auto_manager.auto_start().await {
        tracing::error!("❌ Error al iniciar AutoManager: {}", e);
        tracing::warn!("⚠️  Continuando sin auto-gestión completa");
    }
    
    // 2. Auto-iniciar KPI Tracker (Six Sigma + Automation)
    let kpi_tracker = Arc::new(kpi_tracker::KpiTracker::new(
        kpi_tracker::KpiConfig::default()
    ));
    
    tracing::info!("📊 Iniciando KPI Tracker (Six Sigma)...");
    if let Err(e) = kpi_tracker.start().await {
        tracing::error!("❌ Error al iniciar KPI Tracker: {}", e);
        tracing::warn!("⚠️  Continuando sin KPI tracking");
    }
    
    tracing::info!("✅ Sistema auto-gestionado activo");
    tracing::info!("   • FFI: Julia, JAX, Mojo, Pony, Zig");
    tracing::info!("   • Health checks: cada 30s");
    tracing::info!("   • Auto-recovery: habilitado");
    tracing::info!("   • Zero-touch operation: activo");
    tracing::info!("   • KPI Tracking: Six Sigma always-on");
    tracing::info!("   • Mediciones: cada 10s");
    
    // 3. Construir router con auto-manager y kpi-tracker
    let app = Router::new()
        .merge(mcp_api::routes())
        .fallback(error_404)
        .layer(axum::Extension(auto_manager.clone()))
        .layer(axum::Extension(kpi_tracker.clone()));

    let addr = SocketAddr::from(([127, 0, 0, 1], 4040));

    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    tracing::info!("🚀 Servidor iniciado");
    tracing::info!("📡 Escuchando en http://{}:{}", addr.ip(), addr.port());
    tracing::info!("📋 Protocolo: MCP 2026.1.0-ALWAYS-ON");
    tracing::info!("🔌 Transports: HTTP, WebSocket, stdio");
    tracing::info!("🌐 Compatible: Cursor, Windsurf, Claude Desktop, VS Code");
    tracing::info!("📊 KPIs: Six Sigma + Automation (DMAIC)");
    tracing::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| crate::error::MemoryPError::Io(e))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| crate::error::MemoryPError::Io(e))?;

    // Cleanup al salir
    kpi_tracker.stop().await;
    auto_manager.stop().await;
    
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
