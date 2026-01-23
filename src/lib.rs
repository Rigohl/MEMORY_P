//! MEMORY_P v2.0 - Nuclear MCP Toolkit with 9 Search Engines
//!
//! High-performance search infrastructure with:
//! - 3 Vector Search Engines (Qdrant, FAISS, SCANN)
//! - 4 Text Search Engines (Tantivy, LNX, Toshi, MeiliSearch)
//! - 2 Specialized Engines (Julia NLP, MemoryBank Ultra)

pub mod analyzer;
pub mod config;
pub mod error;
pub mod mcp;
pub mod mcp_api;
pub mod mega_simulator;
// TODO: Fix trait object safety issues in motores module
// The SearchEngine trait has async methods which prevents it from being object-safe
// Need to refactor using enum dispatch or async-trait with Box<dyn Future>
// pub mod motores;
pub mod parallel_engine;
pub mod workspace;

// pub use motores::*;
