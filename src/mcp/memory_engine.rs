// Memory Engine - Core predictive memory system for MEMORY_P v2.0
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::memory_models::*;

/// Trait for predictive memory operations
#[async_trait]
pub trait PredictiveMemory: Send + Sync {
    async fn store_context(&self, ctx: MemoryContext) -> Result<Uuid, MemoryError>;
    async fn get_context(&self, id: Uuid) -> Result<Option<MemoryContext>, MemoryError>;
    async fn predict_next(&self, current: &MemoryContext, lookahead: usize) -> Result<PredictionResult, MemoryError>;
    async fn auto_reorder(&self, strategy: ReorderStrategy) -> Result<usize, MemoryError>;
    async fn cleanup_stale(&self, threshold: Duration) -> Result<usize, MemoryError>;
    async fn get_stats(&self) -> Result<MemoryStats, MemoryError>;
}

/// In-memory cache for predictions
struct PredictionCache {
    cache: HashMap<Uuid, PredictionResult>,
    max_size: usize,
    hits: u64,
    misses: u64,
}

impl PredictionCache {
    fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            hits: 0,
            misses: 0,
        }
    }

    fn get(&mut self, id: &Uuid) -> Option<&PredictionResult> {
        if let Some(result) = self.cache.get(id) {
            self.hits += 1;
            Some(result)
        } else {
            self.misses += 1;
            None
        }
    }

    fn put(&mut self, id: Uuid, result: PredictionResult) {
        if self.cache.len() >= self.max_size {
            // Simple LRU: remove oldest entry
            if let Some(oldest_key) = self.cache.keys().next().cloned() {
                self.cache.remove(&oldest_key);
            }
        }
        self.cache.insert(id, result);
    }

    fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Main predictive memory engine
pub struct PredictiveMemoryEngine {
    contexts: Arc<RwLock<HashMap<Uuid, MemoryContext>>>,
    events: Arc<RwLock<Vec<MemoryEvent>>>,
    config: MemoryEngineConfig,
    prediction_cache: Arc<RwLock<PredictionCache>>,
    stats: Arc<RwLock<MemoryStats>>,
}

impl PredictiveMemoryEngine {
    pub fn new(config: MemoryEngineConfig) -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::with_capacity(config.max_events))),
            prediction_cache: Arc::new(RwLock::new(PredictionCache::new(config.prediction_cache_size))),
            stats: Arc::new(RwLock::new(MemoryStats::default())),
            config,
        }
    }

    /// Store an event in the system (with bounded storage)
    async fn store_event(&self, event: MemoryEvent) {
        let mut events = self.events.write().await;
        
        // Circular buffer behavior: keep only last max_events
        if events.len() >= self.config.max_events {
            events.remove(0);  // Remove oldest
        }
        events.push(event);
        
        let mut stats = self.stats.write().await;
        stats.total_events += 1;
    }

    /// Predict next contexts using simple heuristic (can be replaced with FFI predictors)
    async fn predict_contexts(&self, current: &MemoryContext, lookahead: usize) -> Result<Vec<MemoryContext>, MemoryError> {
        let contexts = self.contexts.read().await;
        
        // Simple prediction: return most recently accessed contexts similar to current
        let mut candidates: Vec<_> = contexts
            .values()
            .filter(|ctx| ctx.id != current.id)
            .cloned()
            .collect();
        
        // Sort by access count and recency
        // Weighting: prediction_score is weighted 10x more than access_count
        // This prioritizes contexts predicted to be relevant over simply frequently accessed ones
        candidates.sort_by(|a, b| {
            let a_score = a.access_count as f64 
                + a.prediction_score.unwrap_or(0.0) * 10.0;  // 10x weight for predictions
            let b_score = b.access_count as f64 
                + b.prediction_score.unwrap_or(0.0) * 10.0;
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        candidates.truncate(lookahead);
        Ok(candidates)
    }
}

#[async_trait]
impl PredictiveMemory for PredictiveMemoryEngine {
    async fn store_context(&self, mut ctx: MemoryContext) -> Result<Uuid, MemoryError> {
        let id = ctx.id;
        ctx.created_at = Utc::now();
        
        let mut contexts = self.contexts.write().await;
        contexts.insert(id, ctx);
        
        // Store event
        let event = MemoryEvent::new(EventType::ContextStored, Some(id));
        self.store_event(event).await;
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_contexts = contexts.len();
        
        Ok(id)
    }

    async fn get_context(&self, id: Uuid) -> Result<Option<MemoryContext>, MemoryError> {
        let mut contexts = self.contexts.write().await;
        
        if let Some(ctx) = contexts.get_mut(&id) {
            ctx.increment_access();
            
            // Store event
            let event = MemoryEvent::new(EventType::ContextRetrieved, Some(id));
            self.store_event(event).await;
            
            Ok(Some(ctx.clone()))
        } else {
            Ok(None)
        }
    }

    async fn predict_next(&self, current: &MemoryContext, lookahead: usize) -> Result<PredictionResult, MemoryError> {
        let start = std::time::Instant::now();
        
        // Check cache first
        let mut cache = self.prediction_cache.write().await;
        if let Some(cached) = cache.get(&current.id) {
            return Ok(cached.clone());
        }
        drop(cache);
        
        // Predict using heuristic or FFI
        let predicted = self.predict_contexts(current, lookahead).await?;
        
        let computation_time_ms = start.elapsed().as_millis() as u64;
        
        let mut result = PredictionResult::new("heuristic".to_string());
        result.predicted_contexts = predicted;
        
        // Calculate confidence based on prediction quality
        let confidence = if lookahead == 0 {
            if result.predicted_contexts.is_empty() {
                0.0
            } else {
                0.5
            }
        } else if result.predicted_contexts.is_empty() {
            0.0
        } else {
            let covered = result.predicted_contexts.len().min(lookahead) as f64;
            let requested = lookahead as f64;
            let coverage = covered / requested;
            0.5 + 0.5 * coverage  // Base 0.5 + up to 0.5 based on coverage
        };
        result.confidence = confidence;
        result.computation_time_ms = computation_time_ms;
        
        // Cache result
        let mut cache = self.prediction_cache.write().await;
        cache.put(current.id, result.clone());
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_predictions += 1;
        let total = stats.total_predictions;
        let current_avg = stats.avg_prediction_time_ms;
        stats.avg_prediction_time_ms = (current_avg * (total - 1) as f64 + computation_time_ms as f64) / total as f64;
        
        // Store event
        let event = MemoryEvent::new(EventType::PredictionRequested, Some(current.id));
        self.store_event(event).await;
        
        Ok(result)
    }

    async fn auto_reorder(&self, strategy: ReorderStrategy) -> Result<usize, MemoryError> {
        let mut contexts = self.contexts.write().await;
        let mut context_vec: Vec<_> = contexts.values_mut().collect();
        
        match strategy {
            ReorderStrategy::MostAccessed => {
                context_vec.sort_by(|a, b| b.access_count.cmp(&a.access_count));
            }
            ReorderStrategy::MostRecent => {
                context_vec.sort_by(|a, b| {
                    b.last_accessed.unwrap_or(b.created_at)
                        .cmp(&a.last_accessed.unwrap_or(a.created_at))
                });
            }
            ReorderStrategy::HighestPredictionScore => {
                context_vec.sort_by(|a, b| {
                    let a_score = a.prediction_score.unwrap_or(0.0);
                    let b_score = b.prediction_score.unwrap_or(0.0);
                    b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            ReorderStrategy::Combined => {
                context_vec.sort_by(|a, b| {
                    let a_score = a.access_count as f64 * 0.4
                        + a.prediction_score.unwrap_or(0.0) * 0.6;
                    let b_score = b.access_count as f64 * 0.4
                        + b.prediction_score.unwrap_or(0.0) * 0.6;
                    b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
        }
        
        let count = context_vec.len();
        
        // Update prediction scores based on new order
        for (idx, ctx) in context_vec.iter_mut().enumerate() {
            ctx.prediction_score = Some(1.0 - (idx as f64 / count.max(1) as f64));
        }
        
        // Store event
        let event = MemoryEvent::new(EventType::ReorderExecuted, None);
        self.store_event(event).await;
        
        Ok(count)
    }

    async fn cleanup_stale(&self, threshold: Duration) -> Result<usize, MemoryError> {
        let mut contexts = self.contexts.write().await;
        let now = Utc::now();
        
        let before_count = contexts.len();
        contexts.retain(|_, ctx| {
            let age = now - ctx.created_at;
            age.num_hours() < threshold.num_hours()
        });
        let removed = before_count - contexts.len();
        
        // Store event
        let mut event = MemoryEvent::new(EventType::CleanupPerformed, None);
        event.payload = serde_json::json!({ "removed": removed });
        self.store_event(event).await;
        
        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_contexts = contexts.len();
        
        Ok(removed)
    }

    async fn get_stats(&self) -> Result<MemoryStats, MemoryError> {
        let stats = self.stats.read().await;
        let cache = self.prediction_cache.read().await;
        
        let mut result = stats.clone();
        result.cache_hit_rate = cache.hit_rate();
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve_context() {
        let config = MemoryEngineConfig::default();
        let engine = PredictiveMemoryEngine::new(config);
        
        let ctx = MemoryContext::new("Test content".to_string());
        let id = ctx.id;
        
        engine.store_context(ctx).await.unwrap();
        
        let retrieved = engine.get_context(id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Test content");
    }
    
    #[tokio::test]
    async fn test_access_count_increments() {
        let config = MemoryEngineConfig::default();
        let engine = PredictiveMemoryEngine::new(config);
        
        let ctx = MemoryContext::new("Test".to_string());
        let id = ctx.id;
        
        engine.store_context(ctx).await.unwrap();
        
        // Access multiple times
        for _ in 0..5 {
            engine.get_context(id).await.unwrap();
        }
        
        let retrieved = engine.get_context(id).await.unwrap().unwrap();
        assert_eq!(retrieved.access_count, 6);  // 5 + 1 from final get
    }
    
    #[tokio::test]
    async fn test_prediction_confidence_calculation() {
        let config = MemoryEngineConfig::default();
        let engine = PredictiveMemoryEngine::new(config);
        
        // Store some contexts
        for i in 0..5 {
            let ctx = MemoryContext::new(format!("Context {}", i));
            engine.store_context(ctx).await.unwrap();
        }
        
        let current = MemoryContext::new("Current".to_string());
        let result = engine.predict_next(&current, 3).await.unwrap();
        
        // Confidence should be calculated based on coverage
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
    }
    
    #[tokio::test]
    async fn test_event_storage_bounded() {
        let mut config = MemoryEngineConfig::default();
        config.max_events = 10;  // Small limit for testing
        
        let engine = PredictiveMemoryEngine::new(config);
        
        // Store more contexts than max_events
        for i in 0..15 {
            let ctx = MemoryContext::new(format!("Context {}", i));
            engine.store_context(ctx).await.unwrap();
        }
        
        // Check that events are bounded
        let events = engine.events.read().await;
        assert!(events.len() <= 10, "Events should be bounded to max_events");
    }
    
    #[tokio::test]
    async fn test_cleanup_stale_contexts() {
        let config = MemoryEngineConfig::default();
        let engine = PredictiveMemoryEngine::new(config);
        
        // Store contexts
        for i in 0..5 {
            let ctx = MemoryContext::new(format!("Context {}", i));
            engine.store_context(ctx).await.unwrap();
        }
        
        // Cleanup with very short threshold (should remove all)
        let removed = engine.cleanup_stale(Duration::seconds(1)).await.unwrap();
        
        // All contexts should be removed as they're older than 1 second
        assert_eq!(removed, 5);
    }
    
    #[tokio::test]
    async fn test_cache_behavior() {
        let config = MemoryEngineConfig::default();
        let engine = PredictiveMemoryEngine::new(config);
        
        let ctx = MemoryContext::new("Test".to_string());
        engine.store_context(ctx.clone()).await.unwrap();
        
        // First prediction - should cache
        let result1 = engine.predict_next(&ctx, 2).await.unwrap();
        
        // Second prediction - should hit cache
        let result2 = engine.predict_next(&ctx, 2).await.unwrap();
        
        // Results should be identical (from cache)
        assert_eq!(result1.predicted_contexts.len(), result2.predicted_contexts.len());
        
        let stats = engine.get_stats().await.unwrap();
        assert!(stats.cache_hit_rate > 0.0);
    }
    
    #[tokio::test]
    async fn test_reordering_strategies() {
        let config = MemoryEngineConfig::default();
        let engine = PredictiveMemoryEngine::new(config);
        
        // Store contexts with different access patterns
        for i in 0..5 {
            let mut ctx = MemoryContext::new(format!("Context {}", i));
            ctx.access_count = i as u64 * 10;  // Different access counts
            engine.store_context(ctx).await.unwrap();
        }
        
        // Test MostAccessed strategy
        let count = engine.auto_reorder(ReorderStrategy::MostAccessed).await.unwrap();
        assert_eq!(count, 5);
        
        // Test Combined strategy
        let count = engine.auto_reorder(ReorderStrategy::Combined).await.unwrap();
        assert_eq!(count, 5);
    }
    
    #[tokio::test]
    async fn test_u64_access_count_no_overflow() {
        let config = MemoryEngineConfig::default();
        let engine = PredictiveMemoryEngine::new(config);
        
        let mut ctx = MemoryContext::new("Test".to_string());
        ctx.access_count = u64::MAX - 5;  // Near max
        let id = ctx.id;
        
        engine.store_context(ctx).await.unwrap();
        
        // Access a few times - should not panic
        for _ in 0..3 {
            engine.get_context(id).await.unwrap();
        }
        
        let retrieved = engine.get_context(id).await.unwrap().unwrap();
        // Should wrap around or be near max without panic
        assert!(retrieved.access_count >= u64::MAX - 5);
    }
}
