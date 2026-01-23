//! Shared types for all search engines

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A document to be indexed or returned from search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique document ID
    pub id: String,
    /// Document content/text
    pub content: String,
    /// Optional vector embedding
    pub vector: Option<Vec<f32>>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Optional timestamp
    pub timestamp: Option<i64>,
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Query text
    pub text: String,
    /// Optional query vector for semantic search
    pub vector: Option<Vec<f32>>,
    /// Query type
    pub query_type: QueryType,
    /// Maximum number of results
    pub limit: usize,
    /// Offset for pagination
    pub offset: usize,
    /// Filters to apply
    pub filters: HashMap<String, serde_json::Value>,
    /// Minimum similarity score (0.0 - 1.0)
    pub min_score: f32,
}

/// Type of query to execute
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryType {
    /// Exact term matching
    Term,
    /// Boolean query (AND, OR, NOT)
    Boolean,
    /// Fuzzy/approximate matching
    Fuzzy,
    /// Vector similarity search
    Vector,
    /// Hybrid (text + vector)
    Hybrid,
    /// Phrase matching
    Phrase,
    /// Range query
    Range,
}

/// Search result returned from a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document ID
    pub id: String,
    /// Relevance score (0.0 - 1.0)
    pub score: f32,
    /// Document content
    pub content: String,
    /// Document metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Which engine produced this result
    pub engine: String,
    /// Highlights/snippets
    pub highlights: Vec<String>,
}

/// Capabilities of a search engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCapabilities {
    /// Supports vector/semantic search
    pub supports_vector_search: bool,
    /// Supports full-text search
    pub supports_full_text: bool,
    /// Supports fuzzy matching
    pub supports_fuzzy: bool,
    /// Supports real-time indexing
    pub supports_real_time: bool,
    /// Supports distributed mode
    pub supports_distributed: bool,
    /// Supports replication
    pub supports_replication: bool,
    /// Supports faceted search
    pub supports_facets: bool,
    /// Supports typo tolerance
    pub supports_typo_tolerance: bool,
    /// Maximum vector dimension (if applicable)
    pub max_vector_dimension: Option<usize>,
    /// Estimated max scale (documents)
    pub max_scale: Option<u64>,
}

/// Health status of an engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineHealth {
    /// Engine name
    pub engine: String,
    /// Is engine healthy
    pub healthy: bool,
    /// Status message
    pub status: String,
    /// Last check timestamp
    pub last_check: i64,
    /// Additional details
    pub details: HashMap<String, serde_json::Value>,
}

/// Performance metrics for an engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMetrics {
    /// Engine name
    pub engine: String,
    /// Total documents indexed
    pub total_documents: u64,
    /// Average query latency (ms)
    pub avg_query_latency_ms: f64,
    /// Queries per second
    pub queries_per_second: f64,
    /// Index size in bytes
    pub index_size_bytes: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Error rate (0.0 - 1.0)
    pub error_rate: f64,
    /// Cache hit rate (0.0 - 1.0)
    pub cache_hit_rate: f64,
    /// Timestamp of metrics
    pub timestamp: i64,
}

/// Query routing pattern for intelligent engine selection
#[derive(Debug, Clone, PartialEq)]
pub enum QueryPattern {
    /// Semantic similarity search
    SemanticSearch,
    /// Massive scale operations (billions+)
    MassiveScale,
    /// Exact text matching
    ExactMatch,
    /// Experimental/testing
    Experimental,
    /// Fuzzy/typo-tolerant search
    FuzzySearch,
    /// Personalized/user-specific search
    PersonalizedSearch,
    /// Mathematical/NLP analysis
    MathematicalAnalysis,
    /// Distributed coordination needed
    DistributedCoordination,
}

/// Engine selection strategy
#[derive(Debug, Clone)]
pub enum EngineSelection {
    /// Primary engine to use
    Primary(&'static str),
    /// Secondary/backup engine
    Secondary(&'static str),
    /// Fallback if primary fails
    Fallback(&'static str),
    /// For distributed queries
    Distributed(&'static str),
    /// For comparison/benchmarking
    Comparison(&'static str),
    /// For mathematical operations
    Mathematical(&'static str),
    /// For semantic operations
    Semantic(&'static str),
}

/// Configuration for an engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Engine name
    pub name: String,
    /// Is engine enabled
    pub enabled: bool,
    /// Connection endpoints
    pub endpoints: Vec<String>,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Performance limits
    pub limits: PerformanceLimits,
    /// Custom settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// Database configuration for an engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Storage type (native, postgres, redis, etc.)
    pub storage_type: String,
    /// Storage path or connection string
    pub storage_path: String,
    /// PostgreSQL schema name (if applicable)
    pub postgres_schema: Option<String>,
    /// Metadata storage
    pub metadata_storage: Option<String>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache type (redis, rocksdb, memory)
    pub cache_type: String,
    /// Cache endpoint/path
    pub cache_endpoint: String,
    /// Cache size limit in bytes
    pub max_size_bytes: u64,
    /// TTL in seconds
    pub ttl_seconds: u64,
}

/// Performance limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceLimits {
    /// Max query latency SLA (ms)
    pub max_latency_ms: u64,
    /// Max concurrent queries
    pub max_concurrent_queries: usize,
    /// Max batch size for indexing
    pub max_batch_size: usize,
    /// Max memory usage (bytes)
    pub max_memory_bytes: u64,
}
