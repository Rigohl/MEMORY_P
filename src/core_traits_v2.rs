//! Core traits for all 9 search engines

use crate::motores::core::types::*;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait SearchEngine: Send + Sync {
    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>>;
    async fn index(&mut self, documents: &[Document]) -> Result<(), Box<dyn std::error::Error>>;
    async fn index_batch(&mut self, documents: Vec<Document>) -> Result<usize, Box<dyn std::error::Error>> {
        self.index(&documents).await?;
        Ok(documents.len())
    }
    async fn delete(&mut self, ids: &[String]) -> Result<(), Box<dyn std::error::Error>>;
    fn engine_name(&self) -> &'static str;
    fn capabilities(&self) -> EngineCapabilities;
    async fn health_check(&self) -> EngineHealth;
    fn get_stats(&self) -> EngineMetrics;
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

#[async_trait]
pub trait VectorSearchEngine: SearchEngine {
    async fn vector_search(
        &self, vector: &[f32], limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>>;
    fn vector_dimension(&self) -> usize;
}

#[async_trait]
pub trait FullTextSearchEngine: SearchEngine {
    async fn full_text_search(
        &self, term: &str, limit: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>>;
}

#[async_trait]
pub trait DistributedSearchEngine: SearchEngine {
    async fn distributed_search(
        &self, query: &SearchQuery, nodes: &[String],
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>>;
    async fn get_nodes(&self) -> Vec<String>;
}
