//! Core traits for all 9 search engines
//!
//! This module defines the interfaces that all search motors must implement.

use crate::motores::core::types::*;
use async_trait::async_trait;
use std::collections::HashMap;

/// Main trait that all search engines must implement
#[async_trait]
pub trait SearchEngine: Send + Sync {
    /// Initialize the engine (connect, load indices, etc.)
    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Shutdown the engine gracefully
    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Execute a search query
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>>;

    /// Index/store documents for searching
    async fn index(&mut self, documents: &[Document]) -> Result<(), Box<dyn std::error::Error>>;

    /// Index a batch of documents (often with different timing)
    async fn index_batch(&mut self, documents: Vec<Document>) -> Result<usize, Box<dyn std::error::Error>> {
        self.index(&documents).await?;
        Ok(documents.len())
    }

    /// Delete documents by ID
    async fn delete(&mut self, ids: &[String]) -> Result<(), Box<dyn std::error::Error>>;

    /// Get engine name/identifier
    fn engine_name(&self) -> &'static str;

    /// Get engine capabilities
    fn capabilities(&self) -> EngineCapabilities;

    /// Get current engine health status
    async fn health_check(&self) -> EngineHealth;

    /// Get engine metrics/statistics
    fn get_stats(&self) -> EngineMetrics;

    /// Get statistics (async version)
    async fn get_statistics(&self) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
        let metrics = self.get_stats();
        let mut stats = HashMap::new();
        stats.insert("total_documents".to_string(), metrics.total_documents as f64);
        stats.insert("avg_latency_ms".to_string(), metrics.avg_query_latency_ms);
        stats.insert("qps".to_string(), metrics.queries_per_second);
        stats.insert("index_size_mb".to_string(), metrics.index_size_bytes as f64 / 1024.0 / 1024.0);
        stats.insert("memory_mb".to_string(), metrics.memory_usage_bytes as f64 / 1024.0 / 1024.0);
        stats.insert("error_rate".to_string(), metrics.error_rate);
        stats.insert("cache_hit_rate".to_string(), metrics.cache_hit_rate);
        Ok(stats)
    }
}

/// Trait for vector-specific search capabilities
#[async_trait]
pub trait VectorSearchEngine: SearchEngine {
    /// Search using vectors directly
    async fn vector_search(
        &self,
        vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>>;

    /// Get vector dimension
    fn vector_dimension(&self) -> usize;
}

/// Trait for text-specific search capabilities
#[async_trait]
pub trait FullTextSearchEngine: SearchEngine {
    /// Full-text search with BM25 ranking
    async fn full_text_search(
        &self,
        term: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>>;
}

/// Trait for distributed search capabilities
#[async_trait]
pub trait DistributedSearchEngine: SearchEngine {
    /// Distributed search across multiple nodes
    async fn distributed_search(
        &self,
        query: &SearchQuery,
        nodes: &[String],
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>>;

    /// Get list of connected nodes
    async fn get_nodes(&self) -> Vec<String>;
}

