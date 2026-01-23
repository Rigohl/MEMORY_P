// motores/core/search_engine.rs
// Common trait for all search engines in MEMORY_P v2.0

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main SearchEngine trait that all 8 engines must implement
#[async_trait]
pub trait SearchEngine: Send + Sync {
    /// Execute search query
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, EngineError>;
    
    /// Get engine type (Vector, Text, Specialized)
    fn engine_type(&self) -> EngineType;
    
    /// Get engine name
    fn engine_name(&self) -> &str;
    
    /// Health check
    async fn health_check(&self) -> HealthStatus;
    
    /// Get current metrics
    fn get_metrics(&self) -> EngineMetrics;
}

/// Types of search engines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineType {
    VectorSearch,
    TextSearch,
    Specialized,
    Hybrid,
}

/// Search query structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub vector: Option<Vec<f32>>,
    pub filters: Option<HashMap<String, String>>,
    pub limit: usize,
    pub offset: usize,
    pub timeout_ms: u64,
    pub min_results: usize,  // Minimum results for cascade fusion
}

/// Search result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub content: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub engine_name: String,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub latency_ms: f32,
    pub message: Option<String>,
}

/// Engine metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetrics {
    pub current_qps: u32,
    pub avg_latency_ms: f32,
    pub p99_latency_ms: f32,
    pub error_rate: f32,
    pub memory_usage_mb: f32,
    pub cpu_usage_percent: f32,
}

/// Engine errors
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Search failed: {0}")]
    SearchFailed(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Timeout after {0}ms")]
    Timeout(u64),
    
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    
    #[error("Engine unavailable: {0}")]
    Unavailable(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
