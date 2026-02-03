// Memory MCP Models - Simplified for initial integration
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Memory context with predictive capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryContext {
    pub id: Uuid,
    pub content: String,
    pub embedding: Option<Vec<f64>>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub access_count: u32,
    pub last_accessed: Option<DateTime<Utc>>,
    pub prediction_score: Option<f64>,
    pub created_at: DateTime<Utc>,
}

impl MemoryContext {
    pub fn new(content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            embedding: None,
            metadata: HashMap::new(),
            access_count: 0,
            last_accessed: None,
            prediction_score: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_embedding(mut self, embedding: Vec<f64>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    pub fn increment_access(&mut self) {
        self.access_count += 1;
        self.last_accessed = Some(Utc::now());
    }
}

/// Prediction result from memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub predicted_contexts: Vec<MemoryContext>,
    pub confidence: f64,
    pub computation_time_ms: u64,
    pub predictor_used: String,
}

impl PredictionResult {
    pub fn new(predictor: String) -> Self {
        Self {
            predicted_contexts: Vec::new(),
            confidence: 0.0,
            computation_time_ms: 0,
            predictor_used: predictor,
        }
    }
}

/// Event types for memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    ContextStored,
    ContextRetrieved,
    PredictionRequested,
    ReorderExecuted,
    CleanupPerformed,
}

/// Memory event for audit and analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    pub id: Uuid,
    pub event_type: EventType,
    pub context_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl MemoryEvent {
    pub fn new(event_type: EventType, context_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            context_id,
            payload: serde_json::json!({}),
            timestamp: Utc::now(),
        }
    }
}

/// Strategy for context reordering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReorderStrategy {
    MostAccessed,
    MostRecent,
    HighestPredictionScore,
    Combined,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_contexts: usize,
    pub cache_hit_rate: f64,
    pub avg_prediction_time_ms: f64,
    pub total_predictions: u64,
    pub total_events: u64,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            total_contexts: 0,
            cache_hit_rate: 0.0,
            avg_prediction_time_ms: 0.0,
            total_predictions: 0,
            total_events: 0,
        }
    }
}

/// Configuration for memory engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEngineConfig {
    pub enable_julia: bool,
    pub enable_mojo: bool,
    pub enable_zig_buffers: bool,
    pub prediction_cache_size: usize,
    pub max_context_age_hours: i64,
    pub auto_cleanup_interval_secs: u64,
}

impl Default for MemoryEngineConfig {
    fn default() -> Self {
        Self {
            enable_julia: false, // Optional FFI features
            enable_mojo: false,
            enable_zig_buffers: false,
            prediction_cache_size: 1000,
            max_context_age_hours: 24,
            auto_cleanup_interval_secs: 3600,
        }
    }
}

/// Error types for memory operations
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Prediction error: {0}")]
    PredictionError(String),
    
    #[error("FFI error: {0}")]
    FfiError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
