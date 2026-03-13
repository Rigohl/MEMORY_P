use mimalloc::MiMalloc;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use memory_p::routes::create_router;
use memory_p::telemetry::{TelemetryConfig, TelemetrySystem};
use memory_p::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting MEMORY_P v2.0");

    // Initialize telemetry system
    let telemetry = Arc::new(TelemetrySystem::new(TelemetryConfig::default()));
    if let Err(e) = telemetry.start().await {
        eprintln!("Telemetry error: {}", e);
    }

    // Run HTTP server
    if let Err(e) = http_server(telemetry.clone()).await {
        eprintln!("Server error: {}", e);
    }

    // Graceful shutdown
    ffi::shutdown();
    let _ = telemetry.shutdown().await;
    Ok(())
}

async fn http_server(telemetry: Arc<TelemetrySystem>) -> error::Result<()> {
    tracing::info!("Starting MEMORY_P HTTP Server v2.2");

    // Initialize all FFI subsystems FIRST before any other initialization
    ffi::init().await?;

    let shared_memory = Arc::new(shared_memory::SharedMemorySystem::new().await?);
    shared_memory.initialize().await?;

    let auto_manager = Arc::new(auto_manager::AutoManager::new(
        auto_manager::ManagerConfig::default(),
    ));
    let _ = auto_manager
        .auto_start(shared_memory.clone(), Some(telemetry.clone()))
        .await;

    let kpi_tracker = Arc::new(kpi_tracker::KpiTracker::new(
        kpi_tracker::KpiConfig::default(),
    ));
    let _ = kpi_tracker.start().await;

    let prediction_engine = Arc::new(prediction_engine::PredictionEngine::new());
    let decision_engine = Arc::new(decision_logic::DecisionEngine::new());

    // Create router with all routes
    let app = create_router(
        auto_manager,
        kpi_tracker,
        shared_memory,
        prediction_engine,
        decision_engine,
        telemetry,
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("🚀 MEMORY_P MCP HTTP Server listening on http://{}", addr);

    axum::serve(listener, app)
        .await
        .map_err(error::MemoryPError::Io)?;
    Ok(())
}
