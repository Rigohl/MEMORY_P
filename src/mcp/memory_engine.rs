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
            events: Arc::new(RwLock::new(Vec::new())),
            prediction_cache: Arc::new(RwLock::new(PredictionCache::new(config.prediction_cache_size))),
            stats: Arc::new(RwLock::new(MemoryStats::default())),
            config,
        }
    }

    /// Store an event in the system
    async fn store_event(&self, event: MemoryEvent) {
        let mut events = self.events.write().await;
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
        candidates.sort_by(|a, b| {
            let a_score = a.access_count as f64 
                + a.prediction_score.unwrap_or(0.0) * 10.0;
            let b_score = b.access_count as f64 
                + b.prediction_score.unwrap_or(0.0) * 10.0;
            b_score.partial_cmp(&a_score).unwrap()
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
        result.confidence = 0.85;
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
                    b_score.partial_cmp(&a_score).unwrap()
                });
            }
            ReorderStrategy::Combined => {
                context_vec.sort_by(|a, b| {
                    let a_score = a.access_count as f64 * 0.4
                        + a.prediction_score.unwrap_or(0.0) * 0.6;
                    let b_score = b.access_count as f64 * 0.4
                        + b.prediction_score.unwrap_or(0.0) * 0.6;
                    b_score.partial_cmp(&a_score).unwrap()
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
