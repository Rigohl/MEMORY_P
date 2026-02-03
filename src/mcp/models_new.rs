// src/mcp/models.rs
// Modelos de datos para el sistema de memoria predictiva

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Contexto de memoria con embeddings y metadata
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MemoryContext {
    pub id: Uuid,
    pub content: String,
    #[sqlx(try_from = "Vec<f32>")]
    pub embedding: Option<Vec<f64>>,
    pub metadata: Option<serde_json::Value>,
    pub access_count: i32,
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
            metadata: None,
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

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn increment_access(&mut self) {
        self.access_count += 1;
        self.last_accessed = Some(Utc::now());
    }
}

/// Resultado de predicción del sistema
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

/// Evento del sistema para event-driven storage
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MemoryEvent {
    pub id: Option<i64>,
    pub event_type: EventType,
    pub context_id: Option<Uuid>,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    ContextStored,
    ContextAccessed,
    ContextPredicted,
    ContextReordered,
    ContextDeleted,
    PredictionCompleted,
    CleanupExecuted,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::ContextStored => write!(f, "context_stored"),
            EventType::ContextAccessed => write!(f, "context_accessed"),
            EventType::ContextPredicted => write!(f, "context_predicted"),
            EventType::ContextReordered => write!(f, "context_reordered"),
            EventType::ContextDeleted => write!(f, "context_deleted"),
            EventType::PredictionCompleted => write!(f, "prediction_completed"),
            EventType::CleanupExecuted => write!(f, "cleanup_executed"),
        }
    }
}

/// Request para almacenar contexto
#[derive(Debug, Deserialize)]
pub struct StoreContextRequest {
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub generate_embedding: Option<bool>,
}

/// Response de almacenamiento
#[derive(Debug, Serialize)]
pub struct StoreContextResponse {
    pub id: Uuid,
    pub prediction_score: f64,
    pub stored_at: DateTime<Utc>,
}

/// Request para predicción
#[derive(Debug, Deserialize)]
pub struct PredictRequest {
    pub context_id: Uuid,
    pub lookahead: Option<usize>,
    pub use_julia: Option<bool>,
    pub use_mojo: Option<bool>,
}

/// Response de predicción
#[derive(Debug, Serialize)]
pub struct PredictResponse {
    pub predictions: Vec<ContextSummary>,
    pub confidence: f64,
    pub computation_time_ms: u64,
    pub predictor_used: String,
}

#[derive(Debug, Serialize)]
pub struct ContextSummary {
    pub id: Uuid,
    pub content_preview: String,
    pub prediction_score: f64,
    pub similarity: f64,
}

/// Request para reordenamiento
#[derive(Debug, Deserialize)]
pub struct ReorderRequest {
    pub strategy: ReorderStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReorderStrategy {
    Frequency,
    Recency,
    Score,
    Auto,
}

/// Response de reordenamiento
#[derive(Debug, Serialize)]
pub struct ReorderResponse {
    pub reordered_count: usize,
    pub strategy_used: String,
    pub execution_time_ms: u64,
}

/// Request para limpieza
#[derive(Debug, Deserialize)]
pub struct CleanupRequest {
    pub threshold_hours: Option<i64>,
    pub min_access_count: Option<i32>,
}

/// Response de limpieza
#[derive(Debug, Serialize)]
pub struct CleanupResponse {
    pub deleted_count: usize,
    pub freed_bytes: u64,
    pub execution_time_ms: u64,
}

/// Métricas del sistema
#[derive(Debug, Serialize)]
pub struct MemoryStats {
    pub total_contexts: i64,
    pub total_events: i64,
    pub avg_prediction_score: f64,
    pub cache_hit_rate: f64,
    pub memory_usage_mb: u64,
    pub uptime_seconds: u64,
}

/// Configuración del motor de memoria
#[derive(Debug, Clone, Deserialize)]
pub struct MemoryEngineConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub enable_julia: bool,
    pub enable_mojo: bool,
    pub enable_zig_buffers: bool,
    pub auto_cleanup_threshold_hours: i64,
    pub prediction_cache_size: usize,
    pub embedding_dimensions: usize,
}

impl Default for MemoryEngineConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost/memory_p".to_string(),
            max_connections: 20,
            enable_julia: true,
            enable_mojo: true,
            enable_zig_buffers: true,
            auto_cleanup_threshold_hours: 168, // 7 días
            prediction_cache_size: 10000,
            embedding_dimensions: 1536, // OpenAI ada-002
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_context_creation() {
        let ctx = MemoryContext::new("test content".to_string());
        assert_eq!(ctx.content, "test content");
        assert_eq!(ctx.access_count, 0);
        assert!(ctx.embedding.is_none());
    }

    #[test]
    fn test_context_with_embedding() {
        let ctx = MemoryContext::new("test".to_string())
            .with_embedding(vec![0.1, 0.2, 0.3]);
        assert!(ctx.embedding.is_some());
        assert_eq!(ctx.embedding.unwrap().len(), 3);
    }

    #[test]
    fn test_increment_access() {
        let mut ctx = MemoryContext::new("test".to_string());
        ctx.increment_access();
        assert_eq!(ctx.access_count, 1);
        assert!(ctx.last_accessed.is_some());
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(EventType::ContextStored.to_string(), "context_stored");
        assert_eq!(EventType::PredictionCompleted.to_string(), "prediction_completed");
    }
}
