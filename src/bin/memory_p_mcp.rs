// src/bin/memory_p_mcp.rs - MEMORY_P MCP DAEMON (ALWAYS-ON)
// 
// ARQUITECTURA: 3 BINARIOS = 1 SISTEMA UNIFICADO
// ====================================================
// Propósito: Servidor MCP HTTP que SIEMPRE está activo (daemon mode)
// - Recibe requests inmediatos del usuario (via CLI o Chat)
// - Usa Julia chaos theory para decidir qué motor (Qdrant/Tantivy/SCANN/etc)
// - Formula respuesta EN VIVO basada en métricas matemáticas
// 
// Puerto MCP: 4040 (HTTP + JSON-RPC 2.0)
// Integración: Conecta a memory_p core (puerto 3000) para motor queries
//
// MODO: DAEMON - Inicia una sola vez, acepta múltiples requests en paralelo
// FFI BRIDGES: 
//   - Julia: chaos_analysis + optimize_weights + decide_search_strategy
//   - Zig: shared_memory_buffer (zero-copy)
//   - JAX: embeddings
//   - Mojo: SIMD kernels
//   - Pony: actor model (fallback)

use clap::Parser;
use log;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global state para el MCP daemon
pub struct McpDaemonState {
    /// Core endpoint donde corren los 9 motores
    pub core_endpoint: String,
    /// Flag para shutdown graceful
    pub running: Arc<RwLock<bool>>,
    /// Métricas de caos actuales
    pub chaos_metrics: Arc<RwLock<ChaosMetrics>>,
}

/// Métricas de caos para enrutamiento automático
#[derive(Clone, Debug)]
pub struct ChaosMetrics {
    pub entropy: f64,
    pub lyapunov_exponent: f64,
    pub stability: f64,
    pub last_update: i64,
}

impl Default for ChaosMetrics {
    fn default() -> Self {
        Self {
            entropy: 2.0,
            lyapunov_exponent: 0.3,
            stability: 0.7,
            last_update: 0,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "MEMORY_P MCP Daemon")]
#[command(about = "MCP HTTP Server with Julia Chaos-Driven Routing (Always-On)")]
#[command(long_about = "
MEMORY_P v2.0 - MCP DAEMON (Sistema Unificado)
================================================

Los 3 binarios forman UN SOLO sistema:
1. memory_p.exe (puerto 3000) - Core con 9 motores + FFI + brain
2. memory_p_mcp.exe (puerto 4040) - Servidor MCP DAEMON [ESTE]
3. memory_p_cli.exe - CLI que consulta MCP para respuesta inmediata

Cuando CLI o usuario escribe prompt + enter:
1. Solicitud llega a este MCP daemon
2. Julia chaos analysis calcula métricas (entropía, Lyapunov)
3. Routing automático elige motor óptimo
4. Consulta memory_p core (puerto 3000)
5. Retorna respuesta EN VIVO al usuario

Nunca se elimina código - TODO es investigado vía MCP tools.
")]
struct Args {
    #[arg(short, long, default_value = "4040")]
    port: u16,

    #[arg(short, long, default_value = "http://localhost:3000")]
    core_endpoint: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Args::parse();

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  MEMORY_P v2.0 - MCP DAEMON (ALWAYS-ON)                   ║");
    println!("║  Chaos-Driven Immediate Response System                   ║");
    println!("║  Part of 3-Binary Unified Architecture                    ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    log::info!("🚀 MCP DAEMON iniciando...");
    log::info!("📡 Puerto MCP: {}", args.port);
    log::info!("🔗 Core endpoint: {}", args.core_endpoint);
    log::info!("🧮 Julia FFI: chaos_analysis + optimize_weights + routing");
    log::info!("⚡ Modo: DAEMON (siempre activo)");
    log::info!("🎯 Arquitectura: 3 binarios = 1 sistema");
    println!();

    // Inicializar FFI bridges
    log::info!("🌉 Inicializando FFI bridges (Julia, Zig, JAX, Mojo, Pony)...");
    match memory_p::ffi::initialize_all().await {
        Ok(_) => log::info!("✅ FFI bridges inicializados"),
        Err(e) => log::warn!("⚠️ FFI init warning: {}", e),
    }
    // Note: FFI initialization may fail if native libraries not compiled locally
    // This is OK - fallback Rust implementations will be used instead

    // Crear estado global del daemon
    let daemon_state = Arc::new(McpDaemonState {
        core_endpoint: args.core_endpoint.clone(),
        running: Arc::new(RwLock::new(true)),
        chaos_metrics: Arc::new(RwLock::new(ChaosMetrics::default())),
    });

    // Iniciar servidor MCP
    match memory_p::mcp::init_http_mcp(args.port).await {
        Ok(_) => {
            log::info!("✅ MCP Server listening on port {}", args.port);
            log::info!("✅ DAEMON READY - Accepting requests (Chaos-Driven Routing)");
            log::info!("📌 Respuesta inmediata basada en Julia chaos metrics");
            println!();
            log::info!("🔄 SIEMPRE activo - Presiona Ctrl+C para shutdown");

            // Mantener daemon activo
            tokio::signal::ctrl_c().await?;
            log::info!("🛑 Shutdown graceful iniciado...");

            // Limpiar FFI resources
            {
                let mut running = daemon_state.running.write().await;
                *running = false;
            }
            log::info!("✅ MCP Daemon shutdown completo");
        }
        Err(e) => {
            log::error!("❌ Error iniciando MCP: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
