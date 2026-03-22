//! Core traits for all search engines
use super::types::*;
use async_trait::async_trait;
use std::error::Error as StdError;

#[derive(Clone, Debug)]
pub struct ClusterInfo {
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub total_shards: usize,
    pub healthy: bool,
    pub node_count: usize,
    pub nodes: Vec<NodeInfo>,
}

#[derive(Clone, Debug)]
pub struct ShardStatus {
    pub shard_id: String,
    pub replicas: usize,
}

#[derive(Clone, Debug)]
pub struct NodeInfo {
    pub node_id: String,
    pub status: String,
    pub is_leader: bool,
    pub shard_count: usize,
    pub id: String,
    pub address: String,
}

#[async_trait]
pub trait SearchEngine: Send + Sync {
    async fn initialize(&mut self) -> Result<(), Box<dyn StdError + Send + Sync>>;
    async fn shutdown(&mut self) -> Result<(), Box<dyn StdError + Send + Sync>>;
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, Box<dyn StdError + Send + Sync>>;
    async fn index(&self, documents: &[Document]) -> Result<(), Box<dyn StdError + Send + Sync>>;
    async fn delete(&self, ids: &[String]) -> Result<(), Box<dyn StdError + Send + Sync>>;
    async fn update(&self, documents: &[Document]) -> Result<(), Box<dyn StdError + Send + Sync>>;
    async fn health(&self) -> Result<EngineHealth, Box<dyn StdError + Send + Sync>>;
    async fn metrics(&self) -> Result<EngineMetrics, Box<dyn StdError + Send + Sync>>;
    async fn capabilities(&self) -> Result<EngineCapabilities, Box<dyn StdError + Send + Sync>>;
    fn engine_name(&self) -> &'static str;
}

#[async_trait]
pub trait VectorSearchEngine: SearchEngine {
    async fn vector_search(&self, vector: &[f32], limit: usize) -> Result<Vec<SearchResult>, Box<dyn StdError + Send + Sync>>;
    fn vector_dimension(&self) -> usize { 768 }
    fn distance_metric(&self) -> &'static str { "cosine" }
}

#[async_trait]
pub trait FullTextSearchEngine: SearchEngine {
    async fn full_text_search(&self, term: &str, limit: usize) -> Result<Vec<SearchResult>, Box<dyn StdError + Send + Sync>>;
}

#[async_trait]
pub trait DistributedEngine: SearchEngine {
    async fn distributed_search(&self, query: &SearchQuery, nodes: &[String]) -> Result<Vec<SearchResult>, Box<dyn StdError + Send + Sync>>;
    async fn cluster_info(&self) -> Result<ClusterInfo, Box<dyn StdError + Send + Sync>>;
    async fn shard_status(&self) -> Result<Vec<ShardStatus>, Box<dyn StdError + Send + Sync>>;
    async fn replicate(&self) -> Result<(), Box<dyn StdError + Send + Sync>>;
}
