//! LNX distributed text search engine — Real HTTP REST client.
//!
//! Connects to a running lnx server (https://github.com/lnx-search/lnx).
//!   Default endpoint: http://localhost:8000
//!   API: POST /indexes/{name}/search  |  POST /indexes/{name}/documents

use crate::motores::core::{
    traits::{DistributedEngine, SearchEngine},
    types::*,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LnxEngine {
    base_url: String,
    index_name: String,
    initialized: bool,
    http: reqwest::Client,
}

impl LnxEngine {
    pub fn new(config: EngineConfig) -> Self {
        let base_url = config
            .endpoints
            .first()
            .cloned()
            .unwrap_or_else(|| "http://localhost:8000".to_string());
        let index_name = config
            .settings
            .get("index_name")
            .and_then(|v| v.as_str())
            .unwrap_or("memory_p")
            .to_string();
        Self {
            base_url,
            index_name,
            initialized: false,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    /// Create the index in lnx if it does not exist yet.
    async fn ensure_index(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = format!("{}/indexes", self.base_url);
        let body = serde_json::json!({
            "override": false,
            "name": self.index_name,
            "storage_type": "memory",
            "fields": {
                "content": { "type": "text", "stored": true },
                "title":   { "type": "text", "stored": true }
            },
            "boost_fields": {},
            "reader_threads": 1,
            "max_concurrency": 4,
            "writer_threads": 1,
            "writer_buffer": 3_000_000
        });
        // Ignore errors: index may already exist
        let _ = self.http.post(&url).json(&body).send().await;
        Ok(())
    }
}

#[async_trait]
impl SearchEngine for LnxEngine {
    async fn search(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, Box<dyn Error + Send + Sync>> {
        if !self.initialized {
            return Err("LNX engine not initialized".into());
        }
        let url = format!("{}/indexes/{}/search", self.base_url, self.index_name);
        let body = serde_json::json!({
            "query": query.text,
            "limit": query.limit,
            "offset": query.offset
        });
        let resp = self.http.post(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("LNX search error {status}: {text}").into());
        }
        let json: serde_json::Value = resp.json().await?;
        let mut results = Vec::new();
        if let Some(hits) = json.get("hits").and_then(|v| v.as_array()) {
            for hit in hits {
                let id = hit
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let score = hit.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let content = hit
                    .pointer("/document/content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if score >= query.min_score {
                    results.push(SearchResult {
                        id,
                        score,
                        content,
                        metadata: HashMap::new(),
                        engine: "lnx".to_string(),
                        highlights: vec![],
                    });
                }
            }
        }
        Ok(results)
    }

    async fn index(&self, documents: &[Document]) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !self.initialized {
            return Err("LNX engine not initialized".into());
        }
        if documents.is_empty() {
            return Ok(());
        }
        let url = format!("{}/indexes/{}/documents", self.base_url, self.index_name);
        let docs: Vec<serde_json::Value> = documents
            .iter()
            .map(|d| {
                serde_json::json!({
                    "id": d.id,
                    "content": d.content,
                    "title": d.metadata.get("title").and_then(|v| v.as_str()).unwrap_or("")
                })
            })
            .collect();
        let resp = self.http.post(&url).json(&docs).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("LNX index error {status}: {text}").into());
        }
        Ok(())
    }

    async fn delete(&self, ids: &[String]) -> Result<(), Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(());
        }
        let url = format!("{}/indexes/{}/documents", self.base_url, self.index_name);
        let body = serde_json::json!({ "ids": ids });
        let _ = self.http.delete(&url).json(&body).send().await;
        Ok(())
    }

    async fn update(&self, documents: &[Document]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.index(documents).await
    }

    async fn health(&self) -> Result<EngineHealth, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/ping", self.base_url);
        let healthy = self
            .http
            .get(&url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);
        Ok(EngineHealth {
            engine: "lnx".to_string(),
            healthy: self.initialized && healthy,
            status: if healthy {
                "Running".to_string()
            } else {
                "Unreachable".to_string()
            },
            last_check: Self::current_timestamp(),
            details: HashMap::from([("endpoint".to_string(), serde_json::json!(self.base_url))]),
        })
    }

    async fn metrics(&self) -> Result<EngineMetrics, Box<dyn Error + Send + Sync>> {
        Ok(EngineMetrics {
            engine: "lnx".to_string(),
            total_documents: 0,
            avg_query_latency_ms: 0.0,
            queries_per_second: 0.0,
            index_size_bytes: 0,
            memory_usage_bytes: 0,
            error_rate: 0.0,
            cache_hit_rate: 0.0,
            timestamp: Self::current_timestamp(),
        })
    }

    fn engine_name(&self) -> &'static str {
        "lnx"
    }

    async fn capabilities(&self) -> Result<EngineCapabilities, Box<dyn std::error::Error + Send + Sync>> {
        Ok(EngineCapabilities {
            supports_vector_search: false,
            supports_full_text: true,
            supports_fuzzy: true,
            supports_real_time: true,
            supports_distributed: true,
            supports_replication: true,
            supports_facets: true,
            supports_typo_tolerance: false,
            max_vector_dimension: None,
            max_scale: Some(10_000_000_000),
        })
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.ensure_index().await?;
        self.initialized = true;
        tracing::info!(
            "LNX engine ready \u{2192} {}/{}",
            self.base_url,
            self.index_name
        );
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.initialized = false;
        Ok(())
    }
}

#[async_trait]
impl DistributedEngine for LnxEngine {
    async fn cluster_info(
        &self,
    ) -> Result<super::super::super::core::traits::ClusterInfo, Box<dyn Error + Send + Sync>> {
        Ok(super::super::super::core::traits::ClusterInfo {
            total_nodes: 1,
            active_nodes: 1,
            total_shards: 1,
            healthy: self.initialized,
            node_count: 1,
            nodes: vec![super::super::super::core::traits::NodeInfo {
                node_id: self.index_name.clone(),
                status: "active".to_string(),
                is_leader: true,
                shard_count: 1,
                id: self.index_name.clone(),
                address: self.base_url.clone(),
            }],
        })
    }

    async fn shard_status(
        &self,
    ) -> Result<Vec<super::super::super::core::traits::ShardStatus>, Box<dyn Error + Send + Sync>>
    {
        Ok(vec![])
    }

    async fn distributed_search(&self, query: &SearchQuery, _nodes: &[String]) -> Result<Vec<SearchResult>, Box<dyn Error + Send + Sync>> {
        // For LNX single node, distributed search is same as regular search
        self.search(query).await
    }

    async fn replicate(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}
