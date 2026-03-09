//! MeiliSearch typo-tolerant search engine

use crate::motores::core::{traits::SearchEngine, types::*};
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use dashmap::DashMap;

pub struct MeiliSearchEngine {
    #[allow(dead_code)]
    config: EngineConfig,
    initialized: bool,
    // Using a local document store to simulate search operations with pseudo typo-tolerance
    documents: Arc<DashMap<String, Document>>,
}

impl MeiliSearchEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            initialized: false,
            documents: Arc::new(DashMap::new()),
        }
    }

    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    /// Simple levenshtein distance for typo tolerance simulation
    fn levenshtein(a: &str, b: &str) -> usize {
        let len_a = a.chars().count();
        let len_b = b.chars().count();
        if len_a == 0 { return len_b; }
        if len_b == 0 { return len_a; }

        let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

        for i in 0..=len_a { matrix[i][0] = i; }
        for j in 0..=len_b { matrix[0][j] = j; }

        for (i, ca) in a.chars().enumerate() {
            for (j, cb) in b.chars().enumerate() {
                let cost = if ca.to_lowercase().to_string() == cb.to_lowercase().to_string() { 0 } else { 1 };
                matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                    .min(matrix[i + 1][j] + 1)
                    .min(matrix[i][j] + cost);
            }
        }
        matrix[len_a][len_b]
    }
}

#[async_trait]
impl SearchEngine for MeiliSearchEngine {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        if !self.initialized {
            return Err("Engine not initialized".into());
        }

        let mut results = Vec::new();
        let query_words: Vec<&str> = query.text.split_whitespace().collect();

        for entry in self.documents.iter() {
            let doc = entry.value();
            let doc_words: Vec<&str> = doc.content.split_whitespace().collect();

            let mut match_score = 0.0;

            for qw in &query_words {
                let mut best_word_score = 0.0;
                for dw in &doc_words {
                    let dist = Self::levenshtein(qw, dw);
                    let max_len = qw.len().max(dw.len()) as f32;
                    let sim = if max_len > 0.0 { 1.0 - (dist as f32 / max_len) } else { 0.0 };

                    if sim > best_word_score {
                        best_word_score = sim;
                    }
                }
                match_score += best_word_score;
            }

            let final_score = if !query_words.is_empty() {
                match_score / query_words.len() as f32
            } else {
                0.0
            };

            // Typo tolerance threshold
            if final_score > 0.4 {
                results.push(SearchResult {
                    id: doc.id.clone(),
                    score: final_score,
                    content: doc.content.clone(),
                    metadata: doc.metadata.clone(),
                    engine: self.engine_name().to_string(),
                    highlights: vec![],
                });
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(query.limit);

        Ok(results)
    }

    async fn index(&self, documents: &[Document]) -> Result<(), Box<dyn Error>> {
        if !self.initialized {
            return Err("Engine not initialized".into());
        }
        for doc in documents {
            self.documents.insert(doc.id.clone(), doc.clone());
        }
        Ok(())
    }

    async fn delete(&self, ids: &[String]) -> Result<(), Box<dyn Error>> {
        for id in ids {
            self.documents.remove(id);
        }
        Ok(())
    }

    async fn update(&self, documents: &[Document]) -> Result<(), Box<dyn Error>> {
        self.index(documents).await
    }

    async fn health(&self) -> Result<EngineHealth, Box<dyn Error>> {
        Ok(EngineHealth {
            engine: "meilisearch".to_string(),
            healthy: self.initialized,
            status: if self.initialized {
                "Running (Local Fallback)".to_string()
            } else {
                "Not initialized".to_string()
            },
            last_check: Self::current_timestamp(),
            details: HashMap::new(),
        })
    }

    async fn metrics(&self) -> Result<EngineMetrics, Box<dyn Error>> {
        Ok(EngineMetrics {
            engine: "meilisearch".to_string(),
            total_documents: self.documents.len() as u64,
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

    async fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        tracing::info!("⚡ Initializing MeiliSearch engine local fallback");
        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {
        self.documents.clear();
        self.initialized = false;
        Ok(())
    }
}
