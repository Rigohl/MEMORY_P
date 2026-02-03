//! MEMORY_P v2.0 - Nuclear MCP Toolkit with 9 Search Engines
//!
//! High-performance search infrastructure with:
//! - 3 Vector Search Engines (Qdrant, FAISS, SCANN)
//! - 4 Text Search Engines (Tantivy, LNX, Toshi, MeiliSearch)
//! - 2 Specialized Engines (Julia NLP, MemoryBank Ultra)
//!
//! NEW v2.0: Autonomous MCP System
//! - Fully autonomous daemon with self-execution
//! - Extended predictive capabilities
//! - Hyper-structured memory management
//! - Advanced workflow automation

pub mod analyzer;
pub mod auto_manager; // Sistema de auto-gestión MCP 2026
pub mod autonomous_daemon; // Sistema de daemon autónomo autoejecutable
pub mod config;
pub mod context_detector; // Detector dinámico de contextos
pub mod error;
pub mod ffi; // FFI multi-lenguaje (Julia, JAX, Mojo, Pony, Zig)
pub mod hyper_memory; // Sistema de memoria hiperestructurada
pub mod kpi_tracker; // Sistema de KPIs Always-On + Six Sigma
pub mod mcp;
pub mod mcp_api;
pub mod mega_simulator;
// TODO: Fix trait object safety issues in motores module
// The SearchEngine trait has async methods which prevents it from being object-safe
// Need to refactor using enum dispatch or async-trait with Box<dyn Future>
// pub mod motores;
pub mod parallel_engine;
pub mod predictive_engine; // Motor de predicción extendida
pub mod workflow_automation; // Sistema de automatización de workflows
pub mod workspace;

// pub use motores::*;
