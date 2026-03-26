//! Shared types for all search engines.

use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub vector: Option<Vec<f32>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub enum QueryType {
    #[default]
    Term,
    Phrase,
    Vector,
    Hybrid,
    Fuzzy,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub vector: Option<Vec<f32>>,
    pub query_type: QueryType,
    pub limit: usize,
    pub offset: usize,
    pub filters: HashMap<String, serde_json::Value>,
    pub min_score: f32,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: String::new(),
            vector: None,
            query_type: QueryType::Term,
            limit: 10,
            offset: 0,
            filters: HashMap::new(),
            min_score: 0.0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub engine: String,
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct EngineConfig {
    pub engine_name: String,
    pub endpoints: Vec<String>,
    pub settings: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct EngineHealth {
    pub engine: String,
    pub healthy: bool,
    pub status: String,
    pub last_check: i64,
    pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct EngineMetrics {
    pub engine: String,
    pub total_documents: u64,
    pub avg_query_latency_ms: f64,
    pub queries_per_second: f64,
    pub index_size_bytes: u64,
    pub memory_usage_bytes: u64,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct EngineCapabilities {
    pub supports_vector_search: bool,
    pub supports_full_text: bool,
    pub supports_fuzzy: bool,
    pub supports_real_time: bool,
    pub supports_distributed: bool,
    pub supports_replication: bool,
    pub supports_facets: bool,
    pub supports_typo_tolerance: bool,
    pub max_vector_dimension: Option<usize>,
    pub max_scale: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryPattern {
    SemanticSearch,
    MassiveScale,
    ExactMatch,
    Experimental,
    FuzzySearch,
    PersonalizedSearch,
    MathematicalAnalysis,
    DistributedCoordination,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineSelection {
    Primary(&'static str),
    Secondary(&'static str),
    Fallback(&'static str),
    Distributed(&'static str),
    Comparison(&'static str),
    Mathematical(&'static str),
    Semantic(&'static str),
}
