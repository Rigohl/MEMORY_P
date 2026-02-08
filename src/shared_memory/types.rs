//! shared_memory/types.rs - Tipos core para memoria compartida

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// ID único de contexto
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextId(pub String);

impl ContextId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn generate() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        Self(format!("ctx_{}", timestamp))
    }
}

impl fmt::Display for ContextId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// ID único de agente
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata de contexto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    /// Timestamp de creación
    pub created_at: i64,

    /// Timestamp de última actualización
    pub updated_at: i64,

    /// Timestamp de último acceso
    pub last_accessed: i64,

    /// Número de accesos
    pub access_count: u64,

    /// Tags asociados
    pub tags: Vec<String>,

    /// Prioridad (0-100)
    pub priority: u8,

    /// Versión del contexto
    pub version: u32,
}

impl ContextMetadata {
    pub fn new() -> Self {
        let now = current_timestamp();
        Self {
            created_at: now,
            updated_at: now,
            last_accessed: now,
            access_count: 0,
            tags: Vec::new(),
            priority: 50, // prioridad media por defecto
            version: 1,
        }
    }

    pub fn touch(&mut self) {
        self.last_accessed = current_timestamp();
        self.access_count += 1;
    }

    pub fn update(&mut self) {
        self.updated_at = current_timestamp();
        self.version += 1;
    }
}

impl Default for ContextMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Contexto de agente individual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// Variables de estado del agente
    pub state: HashMap<String, serde_json::Value>,

    /// Memoria de trabajo (últimas N operaciones)
    pub working_memory: Vec<WorkingMemoryEntry>,

    /// Referencias a otros contextos
    pub context_refs: Vec<ContextId>,

    /// Configuración específica del agente
    pub config: HashMap<String, String>,
}

impl AgentContext {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
            working_memory: Vec::new(),
            context_refs: Vec::new(),
            config: HashMap::new(),
        }
    }

    pub fn add_to_working_memory(&mut self, entry: WorkingMemoryEntry) {
        self.working_memory.push(entry);

        // Limitar a últimas 100 entradas
        if self.working_memory.len() > 100 {
            self.working_memory.remove(0);
        }
    }
}

impl Default for AgentContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Entrada en memoria de trabajo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemoryEntry {
    pub timestamp: i64,
    pub operation: String,
    pub data: serde_json::Value,
}

/// Contexto compartido completo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedContext {
    /// ID del contexto
    pub context_id: ContextId,

    /// ID del agente propietario
    pub agent_id: AgentId,

    /// Metadata del contexto
    pub metadata: ContextMetadata,

    /// Datos del contexto del agente
    pub agent_context: AgentContext,

    /// Datos compartidos entre agentes (clave-valor)
    pub shared_data: HashMap<String, serde_json::Value>,

    /// Grafo de Conocimiento (Entidad -> Relaciones)
    pub knowledge_graph: HashMap<String, Vec<String>>,
}

impl SharedContext {
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            context_id: ContextId::generate(),
            agent_id,
            metadata: ContextMetadata::new(),
            agent_context: AgentContext::new(),
            shared_data: HashMap::new(),
            knowledge_graph: HashMap::new(),
        }
    }

    pub fn touch(&mut self) {
        self.metadata.touch();
    }

    pub fn update(&mut self) {
        self.metadata.update();
    }
}

/// Estadísticas del sistema de memoria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total de contextos activos
    pub active_contexts: usize,

    /// Total de contextos persistidos
    pub persisted_contexts: usize,

    /// Cache hits
    pub cache_hits: u64,

    /// Cache misses
    pub cache_misses: u64,

    /// Tasa de cache hit (0.0 - 1.0)
    pub cache_hit_rate: f64,

    /// Total de actualizaciones
    pub total_updates: u64,

    /// Memoria total usada (bytes)
    pub memory_usage_bytes: u64,

    /// Latencia promedio de operaciones (ms)
    pub avg_latency_ms: f64,

    /// Timestamp de las estadísticas
    pub timestamp: i64,

    /// Puntuación de agilidad de disco (0-100)
    pub disk_agility_score: f64,

    /// Precisión predictiva (0-1)
    pub predictive_accuracy: f64,
}

impl MemoryStats {
    pub fn new() -> Self {
        Self {
            active_contexts: 0,
            persisted_contexts: 0,
            cache_hits: 0,
            cache_misses: 0,
            cache_hit_rate: 0.0,
            total_updates: 0,
            memory_usage_bytes: 0,
            avg_latency_ms: 0.0,
            timestamp: current_timestamp(),
            disk_agility_score: 95.0, // Default optimistic
            predictive_accuracy: 0.85, // Default optimistic
        }
    }

    pub fn calculate_cache_hit_rate(&mut self) {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hit_rate = self.cache_hits as f64 / total as f64;
        }
    }
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Obtiene timestamp actual en segundos
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_id_generation() {
        let id1 = ContextId::generate();
        std::thread::sleep(std::time::Duration::from_millis(2)); // Ensure different timestamps
        let id2 = ContextId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_agent_id_creation() {
        let agent_id = AgentId::new("test-agent".to_string());
        assert_eq!(agent_id.to_string(), "test-agent");
    }

    #[test]
    fn test_shared_context_creation() {
        let agent_id = AgentId::new("test-agent".to_string());
        let context = SharedContext::new(agent_id.clone());
        assert_eq!(context.agent_id, agent_id);
        assert_eq!(context.metadata.version, 1);
    }

    #[test]
    fn test_metadata_touch() {
        let mut metadata = ContextMetadata::new();
        let initial_count = metadata.access_count;
        metadata.touch();
        assert_eq!(metadata.access_count, initial_count + 1);
    }

    #[test]
    fn test_working_memory_limit() {
        let mut context = AgentContext::new();
        for i in 0..150 {
            context.add_to_working_memory(WorkingMemoryEntry {
                timestamp: current_timestamp(),
                operation: format!("op_{}", i),
                data: serde_json::json!({}),
            });
        }
        assert_eq!(context.working_memory.len(), 100);
    }
}
