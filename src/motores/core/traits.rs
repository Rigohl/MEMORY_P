//! Core traits for all search engines in MEMORY_P
//!
//! This module defines the base SearchEngine trait that all 9 engines must implement
//! to ensure consistency and interoperability.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;

use super::types::{
    Document, EngineCapabilities, EngineHealth, EngineMetrics, SearchQuery, SearchResult,
};

/// Core trait that all search engines must implement
#[async_trait]
pub trait SearchEngine: Send + Sync {
    /// Execute a search query and return results
    async fn search(&self, _query: &SearchQuery) -> Result<Vec<SearchResult>, Box<dyn Error>>;

    /// Index documents into the search engine
    async fn index(&self, _documents: &[Document]) -> Result<(), Box<dyn Error>>;

    /// Delete documents by their IDs
    async fn delete(&self, _ids: &[String]) -> Result<(), Box<dyn Error>>;

    /// Update existing documents
    async fn update(&self, _documents: &[Document]) -> Result<(), Box<dyn Error>>;

    /// Get health status of the engine
    async fn health(&self) -> Result<EngineHealth, Box<dyn Error>>;

    /// Get performance metrics
    async fn metrics(&self) -> Result<EngineMetrics, Box<dyn Error>>;

    /// Get the name of the engine (e.g., "qdrant", "tantivy")
    fn engine_name(&self) -> &'static str;

    /// Get the capabilities of this engine
    fn capabilities(&self) -> EngineCapabilities;

    /// Initialize/bootstrap the engine
    async fn initialize(&mut self) -> Result<(), Box<dyn Error>>;

    /// Gracefully shutdown the engine
    async fn shutdown(&mut self) -> Result<(), Box<dyn Error>>;
}

/// Trait for engines that support vector search
#[async_trait]
pub trait VectorSearchEngine: SearchEngine {
    /// Search by vector similarity
    async fn vector_search(
        &self,
        _vector: &[f32],
        _limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>>;

    /// Get the dimension of vectors this engine expects
    fn vector_dimension(&self) -> usize;

    /// Get the distance metric used (cosine, euclidean, etc.)
    fn distance_metric(&self) -> &str;
}

/// Trait for engines that support distributed operations
#[async_trait]
pub trait DistributedEngine: SearchEngine {
    /// Get cluster information
    async fn cluster_info(&self) -> Result<ClusterInfo, Box<dyn Error>>;

    /// Get shard distribution
    async fn shard_status(&self) -> Result<Vec<ShardStatus>, Box<dyn Error>>;

    /// Trigger replication
    async fn replicate(&self) -> Result<(), Box<dyn Error>>;
}

/// Cluster information for distributed engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub node_count: usize,
    pub nodes: Vec<NodeInfo>,
    pub total_shards: usize,
    pub healthy: bool,
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub address: String,
    pub is_leader: bool,
    pub shard_count: usize,
}

/// Shard status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardStatus {
    pub id: String,
    pub primary_node: String,
    pub replica_nodes: Vec<String>,
    pub document_count: u64,
    pub size_bytes: u64,
    pub healthy: bool,
}

/// Trait for engines that support real-time indexing
#[async_trait]
pub trait RealTimeEngine: SearchEngine {
    /// Commit pending changes
    async fn commit(&self) -> Result<(), Box<dyn Error>>;

    /// Rollback pending changes
    async fn rollback(&self) -> Result<(), Box<dyn Error>>;

    /// Check if there are uncommitted changes
    async fn has_uncommitted(&self) -> bool;
}

/// Trait for engines with advanced query capabilities
#[async_trait]
pub trait AdvancedQueryEngine: SearchEngine {
    /// Execute a fuzzy search
    async fn fuzzy_search(
        &self,
        _query: &str,
        _fuzziness: u32,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>>;

    /// Execute a boolean query
    async fn boolean_search(
        &self,
        _query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>>;

    /// Get query suggestions
    async fn suggest(&self, _prefix: &str, _limit: usize) -> Result<Vec<String>, Box<dyn Error>>;
}
