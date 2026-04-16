//! MeiliSearch typo-tolerant search engine — Real HTTP REST client.
//!
//! Connects to a running Meilisearch instance (https://www.meilisearch.com).
//!   Default endpoint: http://localhost:7700
//!   Search:  POST /indexes/{uid}/search
//!   Index:   POST /indexes/{uid}/documents
//!   Health:  GET  /health

use crate::motores::core::{traits::SearchEngine, types::*};
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct MeiliSearchEngine {
    base_url: String,
    index_uid: String,
    api_key: Option<String>,
    initialized: bool,
    http: reqwest::Client,
}

impl MeiliSearchEngine {
    pub fn new(config: EngineConfig) -> Self {
        let base_url = config
            .endpoints
            .first()
            .cloned()
            .unwrap_or_else(|| "http://localhost:7700".to_string());
        let index_uid = config
            .settings
            .get("index_uid")
            .and_then(|v| v.as_str())
            .unwrap_or("memory_p")
            .to_string();
        let api_key = config
            .settings
            .get("api_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        Self {
            base_url,
            index_uid,
            api_key,
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

    fn add_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(key) = &self.api_key {
            req.header("Authorization", format!("Bearer {key}"))
        } else {
            req
        }
    }

    async fn ensure_index(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = format!("{}/indexes", self.base_url);
        let body = serde_json::json!({ "uid": self.index_uid, "primaryKey": "id" });
        // Ignore result — index may already exist
        let _ = self.add_auth(self.http.post(&url)).json(&body).send().await;
        Ok(())
    }
}

#[async_trait]
impl SearchEngine for MeiliSearchEngine {
    async fn search(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, Box<dyn Error + Send + Sync>> {
        if !self.initialized {
            return Err("MeiliSearch engine not initialized".into());
        }
        let url = format!("{}/indexes/{}/search", self.base_url, self.index_uid);
        let body = serde_json::json!({
            "q": query.text,
            "limit": query.limit,
            "offset": query.offset,
            "attributesToRetrieve": ["*"]
        });
        let resp = self
            .add_auth(self.http.post(&url))
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("MeiliSearch error {status}: {text}").into());
        }
        let json: serde_json::Value = resp.json().await?;
        let mut results = Vec::new();
        if let Some(hits) = json.get("hits").and_then(|v| v.as_array()) {
            for (rank, hit) in hits.iter().enumerate() {
                let id = hit
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let content = hit
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                // MeiliSearch basic search ranks by relevance; convert rank to score
                let score = 1.0_f32 / (rank as f32 + 1.0);
                if score >= query.min_score {
                    results.push(SearchResult {
                        id,
                        score,
                        content,
                        metadata: HashMap::new(),
                        engine: "meilisearch".to_string(),
                        highlights: vec![],
                    });
                }
            }
        }
        Ok(results)
    }

    async fn index(&self, documents: &[Document]) -> Result<(), Box<dyn Error + Send + Sync>> {
        if !self.initialized {
            return Err("MeiliSearch engine not initialized".into());
        }
        if documents.is_empty() {
            return Ok(());
        }
        let url = format!("{}/indexes/{}/documents", self.base_url, self.index_uid);
        let docs: Vec<serde_json::Value> = documents
            .iter()
            .map(|d| serde_json::json!({ "id": d.id, "content": d.content }))
            .collect();
        let resp = self
            .add_auth(self.http.post(&url))
            .json(&docs)
            .send()
            .await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("MeiliSearch index error {status}: {text}").into());
        }
        Ok(())
    }

    async fn delete(&self, ids: &[String]) -> Result<(), Box<dyn Error + Send + Sync>> {
        for id in ids {
            let url = format!(
                "{}/indexes/{}/documents/{}",
                self.base_url, self.index_uid, id
            );
            let _ = self.add_auth(self.http.delete(&url)).send().await;
        }
        Ok(())
    }

    async fn update(&self, documents: &[Document]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.index(documents).await
    }

    async fn health(&self) -> Result<EngineHealth, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/health", self.base_url);
        let (healthy, status) = match self.add_auth(self.http.get(&url)).send().await {
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
            engine: "meilisearch".to_string(),
            healthy: self.initialized && healthy,
            status,
            last_check: Self::current_timestamp(),
            details: HashMap::from([("endpoint".to_string(), serde_json::json!(self.base_url))]),
        })
    }

    async fn metrics(&self) -> Result<EngineMetrics, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/indexes/{}/stats", self.base_url, self.index_uid);
        let total = match self.add_auth(self.http.get(&url)).send().await {
            Ok(resp) if resp.status().is_success() => resp
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|j| j.get("numberOfDocuments").and_then(|v| v.as_u64()))
                .unwrap_or(0),
            _ => 0,
        };
        Ok(EngineMetrics {
            engine: "meilisearch".to_string(),
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
        "meilisearch"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_vector_search: false,
            supports_full_text: true,
            supports_fuzzy: true,
            supports_real_time: true,
            supports_distributed: false,
            supports_replication: false,
            supports_facets: true,
            supports_typo_tolerance: true,
            max_vector_dimension: None,
            max_scale: Some(100_000_000),
        }
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.ensure_index().await?;
        self.initialized = true;
        tracing::info!(
            "MeiliSearch engine ready \u{2192} {}/{}",
            self.base_url,
            self.index_uid
        );
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.initialized = false;
        Ok(())
    }
}
