//! MEMORY_P v2.0 - Nuclear MCP Toolkit with Advanced Learning System
//!
//! High-performance search infrastructure with:
//! - 3 Vector Search Engines (Qdrant, FAISS, SCANN)
//! - Advanced Vector Search with HNSW indices
//! - 4 Text Search Engines (Tantivy, LNX, Toshi, MeiliSearch)
//! - 2 Specialized Engines (Julia NLP, MemoryBank Ultra)
//! - JAX integration for embeddings with cache
//! 
//! ## NEW v2.0: Continuous Learning System
//! - Auto-Manager with predictive diagnostics
//! - Pattern detection and user profiling
//! - Chaos-based auto-correction (Julia FFI)
//! - Adaptive parameter optimization
//! - Telemetry system with ClickHouse + Prometheus

pub mod analyzer;
pub mod auto_manager;       // Sistema de auto-gestión MCP 2026 + Learning v2.0
pub mod config;
pub mod error;
pub mod ffi;                // FFI multi-lenguaje (Julia, JAX, Mojo, Pony, Zig)
pub mod kpi_tracker;        // Sistema de KPIs Always-On + Six Sigma
pub mod mcp;
pub mod mcp_api;
pub mod mega_simulator;
pub mod motores;            // Search engines (fixed trait issues with async_trait)
pub mod parallel_engine;
pub mod pattern_detector;   // NEW: Detector de patrones de usuario
pub mod prediction_engine;  // Motor de predicción con Julia + Mojo
pub mod shared_memory;      // Sistema de memoria compartida entre agentes
pub mod telemetry;          // NEW: Sistema de telemetría completo
pub mod workspace;

#[cfg(test)]
mod vector_search_tests;    // Integration tests for vector search

pub use motores::*;
