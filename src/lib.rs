//! MEMORY_P v2.0 - Nuclear MCP Toolkit with 9 Search Engines
//!
//! High-performance search infrastructure with:
//! - 3 Vector Search Engines (Qdrant, FAISS, SCANN)
//! - 4 Text Search Engines (Tantivy, LNX, Toshi, MeiliSearch)
//! - 2 Specialized Engines (Julia NLP, MemoryBank Ultra)

pub mod analyzer;
pub mod backpack; // La "Mochila" del Agente v2.1
pub mod auto_manager; // Sistema de auto-gestión MCP 2026
pub mod cli; // CLI stub para JAR
pub mod config;
pub mod error;
pub mod ffi; // FFI multi-lenguaje (Julia, JAX, Mojo, Pony, Zig)
pub mod kpi_tracker; // Sistema de KPIs Always-On + Six Sigma
pub mod mcp;
pub mod mcp_api;
pub mod mega_simulator;
pub mod prediction_engine;
pub mod autonomous_daemon;
pub mod context_detector;
pub mod predictive_engine;
pub mod hyper_memory;
pub mod pattern_detector;
pub mod telemetry;
pub mod decision_logic;
pub mod nuclear_crawler; // Nuclear Crawler Hybrid System
pub mod parallel_engine;
pub mod shared_memory; // Sistema de memoria compartida para coordinación de agentes
pub mod workspace;

// pub use motores::*;

// FORCED_REBUILDS: Sistema de auto-ajuste de módulos
// Los módulos se activan/desactivan automáticamente según métricas
// Ver: nuclear_crawler::auto_rebuild para configuración dinámica
pub mod motores;
pub use motores::*;
