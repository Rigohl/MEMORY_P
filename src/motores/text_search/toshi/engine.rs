//! Toshi distributed text search engine — Real HTTP REST client.
//!
//! Connects to a running Toshi instance (https://github.com/toshi-search/Toshi).
//!   Default endpoint: http://localhost:8080
//!   Add docs: POST /{index}/_add
//!   Search:   POST /{index}/_search
//!   Summary:  GET  /{index}/_summary

use crate::motores::core::{
    traits::{DistributedEngine, SearchEngine},
    types::*,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ToshiEngine {
    base_url: String,
    index_name: String,
    cluster_nodes: Vec<String>,
    initialized: bool,
    http: reqwest::Client,
}

impl ToshiEngine {
    pub fn new(config: EngineConfig) -> Self {
        let base_url = config
            .endpoints
            .first()
            .cloned()
            .unwrap_or_else(|| "http://localhost:8080".to_string());
        let index_name = config
            .settings
            .get("index_name")
            .and_then(|v| v.as_str())
            .unwrap_or("memory_p")
            .to_string();
        let cluster_nodes = config.endpoints.clone();
        Self {
            base_url,
            index_name,
            cluster_nodes,
            initialized: false,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
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

    /// Create the index in Toshi with a basic schema.
    async fn ensure_index(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = format!("{}/{}", self.base_url, self.index_name);
        let body = serde_json::json!({
            "settings": { "opstamp": 0 },
            "schema": [
                { "name": "id",      "type": "text",
                  "options": { "indexing": {"record": "position"}, "stored": true } },
                { "name": "content", "type": "text",
                  "options": { "indexing": {"record": "position", "tokenizer": "default"}, "stored": true } }
            ]
        });
        // Ignore errors — index may already exist
        let _ = self.http.put(&url).json(&body).send().await;
        Ok(())
    }
}

#[async_trait]
impl SearchEngine for ToshiEngine {
    async fn search(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, Box<dyn Error + Send + Sync>> {
        if !self.initialized {
            return Err("Toshi engine not initialized".into());
        }
        let url = format!("{}/{}/_search", self.base_url, self.index_name);
        let body = serde_json::json!({
            "query": {
                "fuzzy": {
                    "content": {
                        "value": query.text,
                        "distance": 1
                    }
                }
            },
            "limit": query.limit
        });
        let resp = self.http.post(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Toshi search error {status}: {text}").into());
        }
        let json: serde_json::Value = resp.json().await?;
        let mut results = Vec::new();
        if let Some(hits) = json.get("hits").and_then(|v| v.as_array()) {
            for hit in hits {
                let score = hit.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let doc = hit.get("doc");
                let id = doc
                    .and_then(|d| d.get("id"))
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let content = doc
                    .and_then(|d| d.get("content"))
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if score >= query.min_score {
                    results.push(SearchResult {
                        id,
                        score,
                        content,
                        metadata: HashMap::new(),
                        engine: "toshi".to_string(),
                        highlights: vec![],
                    });
                }
            }
        }
        Ok(results)
    }

    async fn index(&self, documents: &[Document]) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !self.initialized {
            return Err("Toshi engine not initialized".into());
        }
        let url = format!("{}/{}/_add", self.base_url, self.index_name);
        for doc in documents {
            let body = serde_json::json!({
                "options": { "commit": true },
                "document": { "id": doc.id, "content": doc.content }
            });
            let resp = self.http.post(&url).json(&body).send().await?;
            if !resp.status().is_success() {
                let status = resp.status();
                tracing::warn!("Toshi: index warning for doc {}: {status}", doc.id);
            }
        }
        Ok(())
    }

    async fn delete(&self, _ids: &[String]) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Open-source Toshi does not expose a delete endpoint
        tracing::warn!("Toshi: document delete not supported in open-source Toshi");
        Ok(())
    }

    async fn update(&self, documents: &[Document]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.index(documents).await
    }

    async fn health(&self) -> Result<EngineHealth, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/{}", self.base_url, self.index_name);
        let (healthy, status) = match self.http.get(&url).send().await {
            Ok(resp) => {
                let st = resp.status();
                if st.is_success() {
                    (true, "Running".to_string())
                } else {
                    let msg = match st {
                        reqwest::StatusCode::NOT_FOUND => "Index not found".to_string(),
                        _ => format!("Unexpected error: {}", st),
                    };
                    (false, msg)
                }
            }
            Err(_) => (false, "Unreachable".to_string()),
        };

        Ok(EngineHealth {
            engine: "toshi".to_string(),
            healthy: self.initialized && healthy,
            status,
            last_check: Self::current_timestamp(),
            details: HashMap::from([
                ("endpoint".to_string(), serde_json::json!(self.base_url)),
                (
                    "nodes".to_string(),
                    serde_json::json!(self.cluster_nodes.len()),
                ),
            ]),
        })
    }

    async fn metrics(&self) -> Result<EngineMetrics, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/{}/_summary", self.base_url, self.index_name);
        let total = match self.http.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => resp
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|j| j.get("num_docs").and_then(|v| v.as_u64()))
                .unwrap_or(0),
            _ => 0,
        };
        Ok(EngineMetrics {
            engine: "toshi".to_string(),
            total_documents: total,
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
        "toshi"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_vector_search: false,
            supports_full_text: true,
            supports_fuzzy: true,
            supports_real_time: true,
            supports_distributed: true,
            supports_replication: true,
            supports_facets: false,
            supports_typo_tolerance: false,
            max_vector_dimension: None,
            max_scale: Some(1_000_000_000),
        }
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.ensure_index().await?;
        self.initialized = true;
        tracing::info!(
            "Toshi engine ready \u{2192} {}/{}",
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
impl DistributedEngine for ToshiEngine {
    async fn cluster_info(
        &self,
    ) -> Result<super::super::super::core::traits::ClusterInfo, Box<dyn Error + Send + Sync>> {
        let nodes: Vec<super::super::super::core::traits::NodeInfo> = self
            .cluster_nodes
            .iter()
            .map(|addr| super::super::super::core::traits::NodeInfo {
                id: addr.clone(),
                address: addr.clone(),
                is_leader: false,
                shard_count: 1,
            })
            .collect();
        Ok(super::super::super::core::traits::ClusterInfo {
            node_count: nodes.len(),
            nodes,
            total_shards: 0,
            healthy: self.initialized,
        })
    }

    async fn shard_status(
        &self,
    ) -> Result<Vec<super::super::super::core::traits::ShardStatus>, Box<dyn Error + Send + Sync>>
    {
        Ok(vec![])
    }

    async fn replicate(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}
