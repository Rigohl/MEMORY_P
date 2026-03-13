//! Search engines module
//!
//! This module contains all 9 search engines for MEMORY_P:
//! - 3 Vector Search Engines (Qdrant, FAISS, SCANN)
//! - 4 Text Search Engines (Tantivy, LNX, Toshi, MeiliSearch)
//! - 2 Specialized Engines (Julia NLP, MemoryBank Ultra)

pub mod core;
pub mod health;
pub mod routing;

pub use core::*;
pub use health::*;
pub use routing::*;

pub async fn start_all() -> Result<(), String> {
    Ok(())
}
pub mod factory;
pub mod hybrid;
pub mod specialized;
pub mod text_search;
pub mod vector_search;

pub use core::{health_monitor::HealthMonitor, routing_ai::RoutingAI, traits::*, types::*};
pub mod persistence;
