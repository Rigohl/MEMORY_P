//! Qdrant vector search engine - Real Client Implementation

use crate::motores::core::{traits::SearchEngine, types::*};
use async_trait::async_trait;
use qdrant_client::{Qdrant, Payload};
use qdrant_client::qdrant::point_id::PointIdOptions;
use qdrant_client::qdrant::{
    Distance, PointStruct, PointId,
    CreateCollectionBuilder, VectorParamsBuilder,
    UpsertPointsBuilder, SearchPointsBuilder, DeletePointsBuilder
};
use std::collections::HashMap;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct QdrantEngine {
    config: EngineConfig,
    client: Option<Qdrant>,
    collection_name: String,
    initialized: bool,
}

impl QdrantEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            client: None,
            collection_name: "memory_p_vectors".to_string(),
            initialized: false,
        }
    }

    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
}

#[async_trait]
impl SearchEngine for QdrantEngine {
    async fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        tracing::info!("⚡ Initializing Qdrant client...");

        let url = self
            .config
            .endpoints
            .first()
            .cloned()
            .unwrap_or_else(|| "http://localhost:6334".to_string());

        let client = Qdrant::from_url(&url).build()?;

        if !client.collection_exists(&self.collection_name).await? {
            client
                .create_collection(
                    CreateCollectionBuilder::new(&self.collection_name)
                        .vectors_config(VectorParamsBuilder::new(384, Distance::Cosine))
                )
                .await?;
            tracing::info!("Created Qdrant collection: {}", self.collection_name);
        }

        self.client = Some(client);
        self.initialized = true;
        tracing::info!("✅ Qdrant engine initialized");
        Ok(())
    }

    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        if !self.initialized {
            return Err("Engine not initialized".into());
        }
        let client = self.client.as_ref().unwrap();

        let vector = query
            .vector
            .clone()
            .ok_or("Vector required for Qdrant search")?;

        let search_result = client
            .search_points(
                SearchPointsBuilder::new(&self.collection_name, vector, query.limit as u64)
                    .with_payload(true)
            )
            .await?;

        let mut results = Vec::new();
        for scored_point in search_result.result {
            let id = scored_point.id.map(|i| {
                match i.point_id_options {
                    Some(PointIdOptions::Uuid(u)) => u,
                    Some(PointIdOptions::Num(n)) => n.to_string(),
                    None => "unknown".to_string(),
                }
            }).unwrap_or_default();

            let payload = scored_point.payload;

            let content = payload.get("content")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string())
                .unwrap_or_default();

            let metadata = HashMap::new();

            results.push(SearchResult {
                id,
                score: scored_point.score,
                content,
                metadata,
                engine: "qdrant".to_string(),
                highlights: vec![],
            });
        }

        Ok(results)
    }

    async fn index(&self, documents: &[Document]) -> Result<(), Box<dyn Error>> {
        if !self.initialized {
            return Err("Engine not initialized".into());
        }
        let client = self.client.as_ref().unwrap();

        let mut points: Vec<PointStruct> = Vec::new();
        for doc in documents {
            let vector = doc.vector.clone().ok_or("Vector missing in document".to_string())?;

            let mut payload = Payload::new();
            payload.insert("content", doc.content.clone());
            for (k, v) in &doc.metadata {
                payload.insert(k, v.clone());
            }

            let point_id = if let Ok(u) = Uuid::parse_str(&doc.id) {
                PointId { point_id_options: Some(PointIdOptions::Uuid(u.to_string())) }
            } else {
                let u = Uuid::new_v5(&Uuid::NAMESPACE_OID, doc.id.as_bytes());
                PointId { point_id_options: Some(PointIdOptions::Uuid(u.to_string())) }
            };

            points.push(PointStruct::new(point_id, vector, payload));
        }

        client
            .upsert_points(UpsertPointsBuilder::new(&self.collection_name, points).wait(true))
            .await?;

        Ok(())
    }

    async fn delete(&self, ids: &[String]) -> Result<(), Box<dyn Error>> {
        if !self.initialized {
            return Err("Engine not initialized".into());
        }
        let client = self.client.as_ref().unwrap();

        let point_ids: Vec<PointId> = ids.iter().map(|id| {
             if let Ok(u) = Uuid::parse_str(id) {
                PointId { point_id_options: Some(PointIdOptions::Uuid(u.to_string())) }
            } else {
                let u = Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes());
                PointId { point_id_options: Some(PointIdOptions::Uuid(u.to_string())) }
            }
        }).collect();

        client.delete_points(
            DeletePointsBuilder::new(&self.collection_name).points(point_ids).wait(true)
        ).await?;
        Ok(())
    }

    async fn update(&self, documents: &[Document]) -> Result<(), Box<dyn Error>> {
        self.index(documents).await
    }

    async fn health(&self) -> Result<EngineHealth, Box<dyn Error>> {
         let healthy = if let Some(client) = &self.client {
            client.collection_exists(&self.collection_name).await.unwrap_or(false)
        } else {
            false
        };

        Ok(EngineHealth {
            engine: "qdrant".to_string(),
            healthy,
            status: if healthy { "Running".to_string() } else { "Error/Not Init".to_string() },
            last_check: Self::current_timestamp(),
            details: HashMap::new(),
        })
    }

    async fn metrics(&self) -> Result<EngineMetrics, Box<dyn Error>> {
        let count = if let Some(client) = &self.client {
            let info = client.collection_info(&self.collection_name).await?;
            info.result.map(|r| r.points_count.unwrap_or(0)).unwrap_or(0)
        } else {
            0
        };

        Ok(EngineMetrics {
            engine: "qdrant".to_string(),
            total_documents: count,
            avg_query_latency_ms: 0.0,
            queries_per_second: 0.0,
            index_size_bytes: 0,
            memory_usage_bytes: 0,
            error_rate: 0.0,
            cache_hit_rate: 0.0,
            timestamp: Self::current_timestamp(),
        })
    }

    fn engine_name(&self) -> &'static str { "qdrant" }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_vector_search: true,
            supports_full_text: false,
            supports_fuzzy: false,
            supports_real_time: true,
            supports_distributed: true,
            supports_replication: true,
            supports_facets: true,
            supports_typo_tolerance: false,
            max_vector_dimension: Some(1536),
            max_scale: Some(1_000_000_000),
        }
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        self.initialized = false;
        self.client = None;
        Ok(())
    }
}
