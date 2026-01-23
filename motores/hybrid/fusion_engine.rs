// motores/hybrid/fusion_engine.rs
// Multi-engine fusion with AI routing

use crate::core::search_engine::{SearchEngine, SearchQuery, SearchResult, EngineError};
use async_trait::async_trait;
use std::sync::Arc;

/// Fusion strategies for combining multiple engines
#[derive(Debug, Clone)]
pub enum FusionStrategy {
    /// Execute searches in parallel and combine results
    Parallel,
    /// Try engines in order until threshold met
    Cascade,
    /// Dynamically adjust based on query analysis
    Adaptive,
}

/// Multi-engine fusion coordinator
pub struct FusionEngine {
    engines: Vec<Arc<dyn SearchEngine>>,
    strategy: FusionStrategy,
    ranker: HybridRanker,
}

impl FusionEngine {
    pub fn new(engines: Vec<Arc<dyn SearchEngine>>, strategy: FusionStrategy) -> Self {
        FusionEngine {
            engines,
            strategy,
            ranker: HybridRanker::new(),
        }
    }
    
    /// Execute fusion search based on strategy
    pub async fn fusion_search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, EngineError> {
        match self.strategy {
            FusionStrategy::Parallel => self.parallel_fusion(query).await,
            FusionStrategy::Cascade => self.cascade_fusion(query).await,
            FusionStrategy::Adaptive => self.adaptive_fusion(query).await,
        }
    }
    
    /// Parallel fusion: search all engines simultaneously
    async fn parallel_fusion(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, EngineError> {
        use futures::future::join_all;
        
        // Execute searches in parallel
        let futures: Vec<_> = self.engines
            .iter()
            .map(|engine| engine.search(query))
            .collect();
        
        let results = join_all(futures).await;
        
        // Collect successful results
        let all_results: Vec<Vec<SearchResult>> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();
        
        // Fuse using Reciprocal Rank Fusion
        Ok(self.ranker.reciprocal_rank_fusion(all_results))
    }
    
    /// Cascade fusion: try engines in order
    async fn cascade_fusion(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, EngineError> {
        let min_results = query.min_results;
        
        for engine in &self.engines {
            if let Ok(results) = engine.search(query).await {
                if results.len() >= min_results {
                    return Ok(results);
                }
            }
        }
        
        Err(EngineError::SearchFailed("All engines returned insufficient results".into()))
    }
    
    /// Adaptive fusion: adjust based on query characteristics
    async fn adaptive_fusion(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, EngineError> {
        // Analyze query to determine best strategy
        let confidence = self.analyze_query_confidence(query);
        
        if confidence > 0.9 {
            // High confidence: use single best engine (assumed pre-ordered by AI router)
            if let Some(engine) = self.engines.first() {
                engine.search(query).await
            } else {
                Err(EngineError::SearchFailed("No search engines configured for adaptive fusion".into()))
            }
        } else {
            // Low confidence: use parallel fusion
            self.parallel_fusion(query).await
        }
    }
    
    fn analyze_query_confidence(&self, query: &SearchQuery) -> f32 {
        // Basic heuristic-based confidence scoring
        // TODO: Replace with ML-based scoring in production
        
        let mut confidence = 0.5;
        
        // Higher confidence if we have a vector
        if query.vector.is_some() {
            confidence += 0.2;
        }
        
        // Higher confidence for simple text queries
        if query.text.len() > 10 && query.text.len() < 100 {
            confidence += 0.1;
        }
        
        // Lower confidence for complex queries
        if query.filters.is_some() {
            confidence -= 0.1;
        }
        
        confidence.clamp(0.0, 1.0)
    }
}

/// Hybrid ranking for combining results from multiple engines
pub struct HybridRanker {
    weights: Vec<f32>,
}

impl HybridRanker {
    pub fn new() -> Self {
        HybridRanker {
            weights: vec![1.0; 8], // Equal weights for all 8 engines initially
        }
    }
    
    /// Reciprocal Rank Fusion algorithm
    /// score(d) = Σ 1 / (k + rank_i(d))
    pub fn reciprocal_rank_fusion(&self, results: Vec<Vec<SearchResult>>) -> Vec<SearchResult> {
        use std::collections::HashMap;
        
        const K: f32 = 60.0;
        let mut scores: HashMap<String, f32> = HashMap::new();
        let mut doc_map: HashMap<String, SearchResult> = HashMap::new();
        
        for (engine_idx, engine_results) in results.iter().enumerate() {
            let weight = self.weights.get(engine_idx).copied().unwrap_or(1.0);
            
            for (rank, result) in engine_results.iter().enumerate() {
                let rrf_score = weight / (K + rank as f32 + 1.0);
                
                *scores.entry(result.id.clone()).or_insert(0.0) += rrf_score;
                doc_map.entry(result.id.clone()).or_insert_with(|| result.clone());
            }
        }
        
        // Sort by fused score
        let mut fused: Vec<_> = scores.into_iter().collect();
        fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Return top results
        fused.into_iter()
            .filter_map(|(doc_id, score)| {
                doc_map.get_mut(&doc_id).map(|mut result| {
                    result.score = score;
                    result.clone()
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reciprocal_rank_fusion() {
        let ranker = HybridRanker::new();
        
        // Simulate results from 2 engines
        let results1 = vec![
            SearchResult { id: "doc1".into(), score: 0.9, content: None, metadata: None, engine_name: "engine1".into() },
            SearchResult { id: "doc2".into(), score: 0.8, content: None, metadata: None, engine_name: "engine1".into() },
        ];
        
        let results2 = vec![
            SearchResult { id: "doc2".into(), score: 0.95, content: None, metadata: None, engine_name: "engine2".into() },
            SearchResult { id: "doc3".into(), score: 0.85, content: None, metadata: None, engine_name: "engine2".into() },
        ];
        
        let fused = ranker.reciprocal_rank_fusion(vec![results1, results2]);
        
        assert!(!fused.is_empty());
        assert_eq!(fused[0].id, "doc2"); // doc2 appears in both, should rank first
    }
}
