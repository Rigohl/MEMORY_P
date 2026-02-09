use axum::Router;
use mimalloc::MiMalloc;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod analyzer;
mod backpack;
mod auto_manager;
mod config;
mod error;
mod ffi;
mod kpi_tracker;
mod mcp;
mod mcp_api;
mod mega_simulator;
mod parallel_engine;
mod prediction_engine;
mod shared_memory;
mod workspace;
mod autonomous_daemon;
mod context_detector;
mod predictive_engine;
mod hyper_memory;
mod pattern_detector;
mod telemetry;
mod decision_logic;

#[tokio::main]
async fn main() {
    if let Err(e) = http_server_mode().await {
        eprintln!("Error: {}", e);
    }
}

async fn http_server_mode() -> crate::error::Result<()> {
    let shared_memory = Arc::new(shared_memory::SharedMemorySystem::new().await?);
    shared_memory.initialize().await?;

    let auto_manager = Arc::new(auto_manager::AutoManager::new(auto_manager::ManagerConfig::default()));
    auto_manager.auto_start(shared_memory.clone()).await?;

    let kpi_tracker = Arc::new(kpi_tracker::KpiTracker::new(kpi_tracker::KpiConfig::default()));
    let prediction_engine = Arc::new(prediction_engine::PredictionEngine::new(prediction_engine::PredictionConfig::default()));
    let decision_engine = Arc::new(decision_logic::DecisionEngine::new());

    let app = Router::new()
        .merge(mcp_api::routes())
        .layer(axum::Extension(auto_manager))
        .layer(axum::Extension(kpi_tracker))
        .layer(axum::Extension(shared_memory))
        .layer(axum::Extension(prediction_engine))
        .layer(axum::Extension(decision_engine));

    let addr = SocketAddr::from(([127, 0, 0, 1], 4040));
    let listener = TcpListener::bind(addr).await.map_err(|e| crate::error::MemoryPError::Io(e))?;
    axum::serve(listener, app).await.map_err(|e| crate::error::MemoryPError::Io(e))?;

    Ok(())
}

async fn mcp_stdio_mode() -> crate::error::Result<()> {
    // Implementación simplificada para el build
    Ok(())
}

async fn error_404() -> (axum::http::StatusCode, String) {
    (axum::http::StatusCode::NOT_FOUND, "Not Found".into())
}
