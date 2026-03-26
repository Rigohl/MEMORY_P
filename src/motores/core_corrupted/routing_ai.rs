//! AI-powered query routing for intelligent engine selection
//!
//! Analyzes queries and routes them to the most appropriate engine(s) based on
//! characteristics, scale requirements, and engine capabilities.

use super::types::{EngineSelection, QueryPattern, QueryType, SearchQuery};
use std::sync::Arc;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EnginePerformanceStats {
    pub avg_latency_ms: f64,
    pub success_rate: f64,
    pub recent_errors: u32,
}

/// Query analyzer and router
pub struct RoutingAI {
    pub engine_stats: HashMap<String, EnginePerformanceStats>,
    #[allow(unused)]
    predictive_engine: Arc<crate::prediction_engine::PredictionEngine>,
}

impl RoutingAI {
    /// Create a new routing AI instance with predictive engine
    pub fn new(predictive_engine: Arc<crate::prediction_engine::PredictionEngine>) -> Self {
        Self {
            engine_stats: HashMap::new(),
            predictive_engine,
        }
    }

    /// Analyze query and determine optimal routing
    pub fn route_query(&self, query: &SearchQuery) -> Vec<EngineSelection> {
        let pattern = self.analyze_query_characteristics(query);
        self.select_engines_for_pattern(&pattern)
    }

    /// Analyze query characteristics to determine pattern
    pub fn analyze_query_characteristics(&self, query: &SearchQuery) -> QueryPattern {
        // Vector search detection
        if query.vector.is_some() && query.query_type == QueryType::Vector {
            return QueryPattern::SemanticSearch;
        }

        // Hybrid search detection
        if query.vector.is_some() && !query.text.is_empty() {
            return QueryPattern::SemanticSearch;
        }

        // Massive scale detection (based on filters or known large datasets)
        if self.requires_massive_scale(query) {
            return QueryPattern::MassiveScale;
        }

        // Fuzzy search detection
        if query.query_type == QueryType::Fuzzy || self.has_typos(&query.text) {
            return QueryPattern::FuzzySearch;
        }

        // Exact matching
        if query.query_type == QueryType::Term || query.query_type == QueryType::Phrase {
            return QueryPattern::ExactMatch;
        }

        // Distributed coordination needed
        if self.requires_distributed(query) {
            return QueryPattern::DistributedCoordination;
        }

        // Mathematical/NLP analysis
        if self.requires_mathematical_analysis(&query.text) {
            return QueryPattern::MathematicalAnalysis;
        }

        // Default to exact match
        QueryPattern::ExactMatch
    }

    /// Route query using chaos metrics for intelligent load balancing (NEW)
    pub async fn route_with_chaos(
        &self,
        query: &SearchQuery,
        system_metrics: &[f64],  // Historical latencies or load
    ) -> Vec<EngineSelection> {
        // 1. Analyze system state using chaos theory
        let lyapunov = match crate::ffi::julia::chaos_analysis(system_metrics) {
            Ok(l) => l,
            Err(_) => {
                tracing::debug!("[ROUTING] Julia unavailable, using fallback routing");
                return self.route_query(query);
            }
        };
        
        let entropy = crate::ffi::julia::shannon_entropy(system_metrics);
        
        // 2. Get base pattern
        let pattern = self.analyze_query_characteristics(query);
        let mut selections = self.select_engines_for_pattern(&pattern);
        
        // 3. NEW: Get predictive suggestions (predict_optimizations not yet implemented)
        // TODO: Add predict_optimizations() to PredictionEngine
        // let optimizations = match self.predictive_engine.predict_optimizations(query).await {
        //     Ok(opts) => opts,
        //     Err(_) => vec![],
        // };
        // if !optimizations.is_empty() {
        //     tracing::debug!("🔮 Predictive suggestions: {:?}", optimizations.iter().map(|o| &o.description).collect::<Vec<_>>());
        // }
        
        // 4. Check for adverse results and warn (detect_adverse_results not yet implemented)
        // TODO: Implement detect_adverse_results in PredictionEngine
        // if let Ok(adverse) = self.predictive_engine.detect_adverse_results().await {
        //     if !adverse.is_empty() {
        //         tracing::warn!("⚠️ Potential adverse results detected: {:?}", adverse);
        //     }
        // }
        
        // 5. Adapt based on chaos (λ = Lyapunov exponent, H = Shannon entropy)
        if lyapunov > 0.5 && entropy > 0.6 {
            // High chaos detected → prefer most stable engines
            tracing::info!("[ROUTING] Chaotic state (λ={:.2}, H={:.2}) → using stable engines", lyapunov, entropy);
            selections = selections.into_iter()
                .filter(|e| self.is_stable_engine(e))
                .collect();
        } else if lyapunov < 0.1 && entropy < 0.3 {
            // Very stable state → can parallelize aggressively
            tracing::debug!("[ROUTING] Stable state (λ={:.2}, H={:.2}) → parallelizing", lyapunov, entropy);
            selections.push(EngineSelection::Secondary("faiss"));
            selections.push(EngineSelection::Secondary("tantivy"));
        }
        
        // 6. NEW: Apply predictive optimizations if available (predict_optimizations not yet implemented)  
        // TODO: Implement predict_optimizations()
        // if let Some(opt) = optimizations.first() { ...}
        
        selections
    }

    /// Check if engine is stable under chaotic conditions
    fn is_stable_engine(&self, selection: &EngineSelection) -> bool {
        match selection {
            EngineSelection::Primary(name) => {
                // Most stable under chaos: strict ordering + low latency variance
                matches!(name, &"tantivy" | &"qdrant")
            },
            EngineSelection::Secondary(name) => {
                // Secondary stable option
                matches!(name, &"lnx" | &"meilisearch")
            },
            _ => false,
        }
    }

    /// Select engines based on query pattern
    pub fn select_engines_for_pattern(&self, pattern: &QueryPattern) -> Vec<EngineSelection> {
        match pattern {
            QueryPattern::SemanticSearch => vec![
                EngineSelection::Primary("qdrant"),
                EngineSelection::Fallback("faiss"),
            ],
            QueryPattern::MassiveScale => vec![
                EngineSelection::Primary("scann"),
                EngineSelection::Secondary("faiss"),
            ],
            QueryPattern::ExactMatch => vec![
                EngineSelection::Primary("tantivy"),
                EngineSelection::Distributed("lnx"),
            ],
            QueryPattern::Experimental => vec![
                EngineSelection::Primary("toshi"),
                EngineSelection::Comparison("lnx"),
            ],
            QueryPattern::FuzzySearch => vec![
                EngineSelection::Primary("meilisearch"),
                EngineSelection::Mathematical("julia_nlp"),
            ],
            QueryPattern::PersonalizedSearch => vec![
                EngineSelection::Primary("memory_bank"),
                EngineSelection::Semantic("qdrant"),
            ],
            QueryPattern::MathematicalAnalysis => vec![
                EngineSelection::Primary("julia_nlp"),
                EngineSelection::Secondary("memory_bank"),
            ],
            QueryPattern::DistributedCoordination => vec![
                EngineSelection::Primary("lnx"),
                EngineSelection::Secondary("toshi"),
            ],
        }
    }

    /// Check if query requires massive scale processing
    fn requires_massive_scale(&self, query: &SearchQuery) -> bool {
        // Check if filter suggests large dataset
        query
            .filters
            .get("dataset_size")
            .and_then(|v: &serde_json::Value| v.as_u64())
            .map(|size| size > 1_000_000_000) // > 1 billion
            .unwrap_or(false)
    }

    /// Check if text likely has typos
    fn has_typos(&self, text: &str) -> bool {
        // Simple heuristic: check for common typo patterns
        let typo_indicators = ["teh", "thier", "recieve", "occurence"];
        typo_indicators.iter().any(|&typo| text.contains(typo))
    }

    /// Check if distributed processing is needed
    fn requires_distributed(&self, query: &SearchQuery) -> bool {
        query
            .filters
            .get("distributed")
            .and_then(|v: &serde_json::Value| v.as_bool())
            .unwrap_or(false)
    }

    /// Check if mathematical analysis is required
    fn requires_mathematical_analysis(&self, text: &str) -> bool {
        // Check for mathematical terms
        let math_indicators = [
            "similarity",
            "distance",
            "correlation",
            "entropy",
            "statistics",
            "mathematical",
        ];
        math_indicators
            .iter()
            .any(|&indicator| text.to_lowercase().contains(indicator))
    }

    /// Update engine performance statistics
    pub fn update_engine_stats(&mut self, engine: &str, latency_ms: f64, success: bool) {
        let stats = self
            .engine_stats
            .entry(engine.to_string())
            .or_insert(EnginePerformanceStats {
                avg_latency_ms: 0.0,
                success_rate: 1.0,
                recent_errors: 0,
            });

        // Update exponential moving average
        stats.avg_latency_ms = stats.avg_latency_ms * 0.9 + latency_ms * 0.1;

        if success {
            stats.success_rate = stats.success_rate * 0.95 + 0.05;
            stats.recent_errors = stats.recent_errors.saturating_sub(1);
        } else {
            stats.success_rate *= 0.95;
            stats.recent_errors = stats.recent_errors.saturating_add(1);
        }
    }

    /// Get best performing engine for a given pattern
    pub fn get_best_engine(&self, pattern: &QueryPattern) -> Option<&'static str> {
        let engines = self.select_engines_for_pattern(pattern);
        engines.first().and_then(|selection| match selection {
            EngineSelection::Primary(name) => Some(*name),
            _ => None,
        })
    }
}

impl Default for RoutingAI {
    fn default() -> Self {
        // Create dummy PredictionEngine for tests/defaults
        let dummy_predictor = Arc::new(crate::prediction_engine::PredictionEngine::new());
        Self::new(dummy_predictor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_semantic_search_routing() {
        let router = RoutingAI::default();
        let query = SearchQuery {
            text: "find similar documents".to_string(),
            vector: Some(vec![0.1, 0.2, 0.3]),
            query_type: QueryType::Vector,
            limit: 10,
            offset: 0,
            filters: HashMap::new(),
            min_score: 0.7,
        };

        let engines = router.route_query(&query);
        assert!(!engines.is_empty());

        if let EngineSelection::Primary(name) = engines[0] {
            assert_eq!(name, "qdrant");
        }
    }

    #[test]
    fn test_fuzzy_search_routing() {
        let router = RoutingAI::default();
        let query = SearchQuery {
            text: "teh quick brown fox".to_string(),
            vector: None,
            query_type: QueryType::Fuzzy,
            limit: 10,
            offset: 0,
            filters: HashMap::new(),
            min_score: 0.5,
        };

        let pattern = router.analyze_query_characteristics(&query);
        assert_eq!(pattern, QueryPattern::FuzzySearch);
    }

    #[test]
    fn test_exact_match_routing() {
        let router = RoutingAI::default();
        let query = SearchQuery {
            text: "exact phrase match".to_string(),
            vector: None,
            query_type: QueryType::Term,
            limit: 10,
            offset: 0,
            filters: HashMap::new(),
            min_score: 0.9,
        };

        let engines = router.route_query(&query);
        if let EngineSelection::Primary(name) = engines[0] {
            assert_eq!(name, "tantivy");
        }
    }
}
