use axum::Router;
use mimalloc::MiMalloc;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Import everything from the library
use memory_p::*;
use memory_p::telemetry::{TelemetrySystem, TelemetryConfig};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Inicializar Telemetría Always-On
    let telemetry = Arc::new(TelemetrySystem::new(TelemetryConfig::default()));
    if let Err(e) = telemetry.start().await {
        eprintln!("Error al iniciar telemetría: {}", e);
    }

    if args.contains(&"--stdio".to_string()) || std::env::var("MCP_STDIO").is_ok() {
        if let Err(e) = mcp_stdio_mode(telemetry.clone()).await {
            eprintln!("❌ Error en modo stdio: {}", e);
        }
    } else {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_max_level(tracing::Level::INFO)
            .with_target(true)
            .init();

        if let Err(e) = http_server_mode(telemetry.clone()).await {
            tracing::error!("❌ Error en servidor HTTP: {}", e);
        }
    }

    // Shutdown FFI
    ffi::shutdown();
    let _ = telemetry.shutdown().await;
}

async fn http_server_mode(telemetry: Arc<TelemetrySystem>) -> error::Result<()> {
    tracing::info!("╔══════════════════════════════════════════════════╗");
    tracing::info!("║  MEMORY_P MCP Server 2.2 - EVOLVED EDITION     ║");
    tracing::info!("╚══════════════════════════════════════════════════╝");

    let shared_memory = Arc::new(shared_memory::SharedMemorySystem::new().await?);
    shared_memory.initialize().await?;

    let auto_manager = Arc::new(auto_manager::AutoManager::new(auto_manager::ManagerConfig::default()));
    if let Err(e) = auto_manager.auto_start(shared_memory.clone(), Some(telemetry.clone())).await {
        tracing::error!("❌ Error al iniciar AutoManager: {}", e);
    }

    let kpi_tracker = Arc::new(kpi_tracker::KpiTracker::new(kpi_tracker::KpiConfig::default()));
    let _ = kpi_tracker.start().await;

    let prediction_engine = Arc::new(prediction_engine::PredictionEngine::new(prediction_engine::PredictionConfig::default()));
    let decision_engine = Arc::new(decision_logic::DecisionEngine::new());

    let app = Router::new()
        .merge(mcp_api::routes())
        .fallback(error_404)
        .layer(axum::Extension(auto_manager))
        .layer(axum::Extension(kpi_tracker))
        .layer(axum::Extension(shared_memory))
        .layer(axum::Extension(prediction_engine))
        .layer(axum::Extension(decision_engine))
        .layer(axum::Extension(telemetry));

    let addr = SocketAddr::from(([127, 0, 0, 1], 4040));
    let listener = TcpListener::bind(addr).await.map_err(|e| error::MemoryPError::Io(e))?;

    tracing::info!("🚀 Servidor MEMORY_P v2.2 Evolved escuchando en http://{}", addr);
    axum::serve(listener, app).await.map_err(|e| error::MemoryPError::Io(e))?;
    Ok(())
}

async fn mcp_stdio_mode(telemetry: Arc<TelemetrySystem>) -> error::Result<()> {
    use crate::mcp::models::JsonRpcRequest;
    use crate::mcp_api::mcp_json_rpc_handler;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    let shared_memory = Arc::new(shared_memory::SharedMemorySystem::new().await?);
    shared_memory.initialize().await?;

    let prediction_engine = Arc::new(prediction_engine::PredictionEngine::new(prediction_engine::PredictionConfig::default()));

    let decision_engine = Arc::new(decision_logic::DecisionEngine::new());
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();
    let mut line = String::new();

    while stdin.read_line(&mut line).await? > 0 {
        if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&line) {
            let response = mcp_json_rpc_handler(
                axum::extract::Extension(shared_memory.clone()),
                axum::extract::Extension(prediction_engine.clone()),
                axum::extract::Extension(decision_engine.clone()),
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
