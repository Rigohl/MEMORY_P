//! Shared traits for all search engines.

use super::types::{
	Document, EngineCapabilities, EngineHealth, EngineMetrics, SearchQuery, SearchResult,
};
use async_trait::async_trait;
use std::error::Error;

#[derive(Debug, Clone, Default)]
pub struct NodeInfo {
	pub id: String,
	pub address: String,
	pub is_leader: bool,
	pub shard_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ClusterInfo {
	pub node_count: usize,
	pub nodes: Vec<NodeInfo>,
	pub total_shards: usize,
	pub healthy: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ShardStatus {
	pub shard_id: String,
	pub node_id: String,
	pub healthy: bool,
	pub document_count: u64,
}

#[async_trait]
pub trait SearchEngine: Send + Sync {
	async fn search(
		&self,
		query: &SearchQuery,
	) -> Result<Vec<SearchResult>, Box<dyn Error + Send + Sync>>;

	async fn index(
		&self,
		documents: &[Document],
	) -> Result<(), Box<dyn Error + Send + Sync>>;

	async fn delete(&self, ids: &[String]) -> Result<(), Box<dyn Error + Send + Sync>>;

	async fn update(
		&self,
		documents: &[Document],
	) -> Result<(), Box<dyn Error + Send + Sync>>;

	async fn health(&self) -> Result<EngineHealth, Box<dyn Error + Send + Sync>>;

	async fn metrics(&self) -> Result<EngineMetrics, Box<dyn Error + Send + Sync>>;

	fn engine_name(&self) -> &'static str;

	fn capabilities(&self) -> EngineCapabilities;

	async fn initialize(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;

	async fn shutdown(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait VectorSearchEngine: SearchEngine {
	async fn vector_search(
		&self,
		vector: &[f32],
		limit: usize,
	) -> Result<Vec<SearchResult>, Box<dyn Error + Send + Sync>>;

	fn vector_dimension(&self) -> usize;

	fn distance_metric(&self) -> &str;
}

#[async_trait]
pub trait DistributedEngine: SearchEngine {
	async fn cluster_info(&self) -> Result<ClusterInfo, Box<dyn Error + Send + Sync>>;

	async fn shard_status(&self) -> Result<Vec<ShardStatus>, Box<dyn Error + Send + Sync>>;

	async fn replicate(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}
