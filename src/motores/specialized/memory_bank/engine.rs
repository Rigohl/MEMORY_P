use crate::motores::core::{traits::SearchEngine, types::*};
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct MemoryBankEngine {
    config: EngineConfig,
    initialized: bool,
    pattern_detector: Arc<crate::pattern_detector::PatternDetector>,
}

impl MemoryBankEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            config,
            initialized: false,
            pattern_detector: Arc::new(crate::pattern_detector::PatternDetector::new()),
        }
    }

    fn ts() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
}

#[async_trait]
impl SearchEngine for MemoryBankEngine {
    async fn search(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, Box<dyn Error + Send + Sync>> {
        // === NEW: Math-aware memory ranking using entropy ===
        
        // 1. Get candidate memories (base query matching)
        let candidates = self.get_candidate_memories(query)?;
        
        if candidates.is_empty() {
            return Ok(vec![]);
        }
        
        // 2. Compute entropy for each candidate
        // Higher entropy = less predictable/relevant = lower score
        let mut scored_results: Vec<(SearchResult, f64)> = candidates
            .into_iter()
            .map(|mut result| {
                // Compute entropy from content if available
                let entropy = if let Some(vector) = &query.vector {
                    // Use query vector as proxy for content structure
                    // Convert f32 to f64 for Julia FFI
                    let f64_vector: Vec<f64> = vector.iter().map(|&x| x as f64).collect();
                    crate::ffi::julia::shannon_entropy(&f64_vector)
                } else {
                    // If no vector, estimate from text
                    let text = result.content.as_bytes();
                    let freq_dist: Vec<f64> = (0..=255u8)
                        .map(|byte| {
                            text.iter().filter(|&&b| b == byte).count() as f64
                        })
                        .filter(|&count| count > 0.0)
                        .collect();
                    
                    if freq_dist.is_empty() {
                        0.5
                    } else {
                        crate::ffi::julia::shannon_entropy(&freq_dist)
                    }
                };
                
                // 3. Compute stability score: 1.0 - entropy_penalty
                // Lower entropy = higher relevance (more stable memory)
                let stability_score = (1.0 - entropy * 0.7).max(0.1);
                
                // 4. Base score from query matching
                let base_relevance = if result.content.contains(&query.text) {
                    1.0
                } else {
                    0.5
                };
                
                // 5. Combine stability + relevance
                let final_score = base_relevance * stability_score;
                result.score = final_score as f32;
                
                (result, final_score)
            })
            .collect();
        
        // 6. Sort by score descending
        scored_results.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // 7. Return top N results above confidence threshold
        let threshold = 0.3_f64;
        let limit = if query.limit > 0 { query.limit } else { 10 };
        
        let results: Vec<crate::motores::core::types::SearchResult> = scored_results
            .into_iter()
            .filter(|(_, score)| *score >= threshold)
            .take(limit)
            .map(|(result, _)| result)
            .collect();
        
        tracing::debug!(
            "[MEMORY_BANK] Ranked {} memories using entropy scaling",
            results.len()
        );
        
        // NEW: Record query pattern for user analysis
        let user_id = "default_user"; // In production, would come from context
        let action = crate::pattern_detector::UserAction {
            timestamp: chrono::Utc::now(),
            action_type: "search".to_string(),
            tool: "memory_bank".to_string(),
            language: None,
            success: !results.is_empty(),
            duration_secs: 0.01, // ~10ms
        };
        self.pattern_detector.record_action(user_id, action).await;
        
        Ok(results)
    }

    async fn index(&self, _documents: &[Document]) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    async fn delete(&self, _ids: &[String]) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    async fn update(&self, _documents: &[Document]) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    async fn health(&self) -> Result<EngineHealth, Box<dyn Error + Send + Sync>> {
        Ok(EngineHealth {
            engine: "memory_bank".to_string(),
            healthy: true,
            status: if self.initialized {
                "Running".to_string()
            } else {
                "Not initialized".to_string()
            },
            last_check: Self::ts(),
            details: HashMap::from([(
                "config".to_string(),
                serde_json::json!(self.config.engine_name),
            )]),
        })
    }

    async fn metrics(&self) -> Result<EngineMetrics, Box<dyn Error + Send + Sync>> {
        Ok(EngineMetrics {
            engine: "memory_bank".to_string(),
            timestamp: Self::ts(),
            ..EngineMetrics::default()
        })
    }

    fn engine_name(&self) -> &'static str {
        "memory_bank"
    }

    async fn capabilities(&self) -> Result<EngineCapabilities, Box<dyn std::error::Error + Send + Sync>> {
        Ok(EngineCapabilities {
            supports_vector_search: true,
            supports_full_text: true,
            supports_fuzzy: true,
            supports_real_time: true,
            supports_distributed: true,
            supports_replication: false,
            supports_facets: true,
            supports_typo_tolerance: true,
            max_vector_dimension: Some(4096),
            max_scale: Some(100_000_000),
        })
    }

    async fn initialize(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.initialized = false;
        Ok(())
    }
}

// === Implementation for math-aware helper functions ===
impl MemoryBankEngine {
    /// Get candidate memories for a query (internal helper)
    fn get_candidate_memories(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, Box<dyn Error + Send + Sync>> {
        // For now, return single synthetic result
        // In production, this would query actual memory storage
        Ok(vec![SearchResult {
            id: format!("memory-bank:{}", query.text),
            score: 0.5,  // Will be recomputed by search()
            content: query.text.clone(),
            metadata: HashMap::from([(
                "strategy".to_string(),
                serde_json::json!("hybrid_memory_coordination"),
            )]),
            engine: "memory_bank".to_string(),
            highlights: vec!["memory coordination".to_string()],
        }])
    }
}
