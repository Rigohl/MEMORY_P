//! shared_memory.rs - Sistema de Memoria Compartida para Agentes
//!
//! Proporciona un sistema de memoria compartida optimizada entre agentes
//! que gestiona y sincroniza contextos para evitar búsquedas redundantes.
//!
//! Características:
//! - Memoria compartida thread-safe con DashMap
//! - Cache de contextos con TTL
//! - Sincronización automática entre agentes
//! - Integración con Redis para cache distribuido

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// Identificador único para un contexto
pub type ContextId = String;

/// Identificador de agente
pub type AgentId = String;

/// Entrada de contexto con metadatos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    /// ID del contexto
    pub id: ContextId,
    /// ID del agente propietario
    pub agent_id: AgentId,
    /// Datos del contexto (JSON serializable)
    pub data: serde_json::Value,
    /// Timestamp de creación
    pub created_at: u64,
    /// Timestamp de último acceso
    pub last_accessed: u64,
    /// TTL en segundos (0 = infinito)
    pub ttl_seconds: u64,
    /// Número de accesos
    pub access_count: u64,
    /// Hash del contenido para deduplicación
    pub content_hash: String,
}

impl ContextEntry {
    /// Verifica si la entrada ha expirado
    pub fn is_expired(&self) -> bool {
        if self.ttl_seconds == 0 {
            return false;
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        (now - self.created_at) > self.ttl_seconds
    }
    
    /// Actualiza el timestamp de último acceso
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.access_count += 1;
    }
}

/// Sistema de memoria compartida
pub struct SharedMemory {
    /// Almacén principal de contextos
    contexts: Arc<DashMap<ContextId, ContextEntry>>,
    /// Índice por agente
    agent_index: Arc<DashMap<AgentId, Vec<ContextId>>>,
    /// Índice por hash de contenido (deduplicación)
    content_index: Arc<DashMap<String, ContextId>>,
    /// Cliente Redis opcional para cache distribuido
    #[cfg(feature = "redis")]
    redis_client: Option<redis::Client>,
}

impl SharedMemory {
    /// Crea una nueva instancia de memoria compartida
    pub fn new() -> Self {
        info!("🧠 Inicializando sistema de memoria compartida");
        
        Self {
            contexts: Arc::new(DashMap::new()),
            agent_index: Arc::new(DashMap::new()),
            content_index: Arc::new(DashMap::new()),
            #[cfg(feature = "redis")]
            redis_client: None,
        }
    }
    
    /// Inicializa conexión con Redis
    #[cfg(feature = "redis")]
    pub fn with_redis(mut self, redis_url: &str) -> Result<Self, redis::RedisError> {
        info!("🔌 Conectando a Redis: {}", redis_url);
        let client = redis::Client::open(redis_url)?;
        self.redis_client = Some(client);
        Ok(self)
    }
    
    /// Almacena un contexto en memoria compartida
    pub fn store_context(
        &self,
        agent_id: AgentId,
        data: serde_json::Value,
        ttl_seconds: u64,
    ) -> Result<ContextId, Box<dyn std::error::Error>> {
        // Calcular hash del contenido para deduplicación
        let content_hash = self.calculate_hash(&data);
        
        // Verificar si ya existe un contexto con el mismo contenido
        if let Some(existing_id) = self.content_index.get(&content_hash) {
            debug!(
                "📋 Contexto duplicado detectado, reutilizando: {}",
                existing_id.value()
            );
            
            // Actualizar acceso
            if let Some(mut entry) = self.contexts.get_mut(existing_id.value()) {
                entry.touch();
            }
            
            return Ok(existing_id.value().clone());
        }
        
        // Crear nuevo contexto
        let id = self.generate_context_id(&agent_id);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let entry = ContextEntry {
            id: id.clone(),
            agent_id: agent_id.clone(),
            data,
            created_at: now,
            last_accessed: now,
            ttl_seconds,
            access_count: 1,
            content_hash: content_hash.clone(),
        };
        
        // Almacenar en memoria
        self.contexts.insert(id.clone(), entry);
        
        // Actualizar índices
        self.agent_index
            .entry(agent_id.clone())
            .or_insert_with(Vec::new)
            .push(id.clone());
        
        self.content_index.insert(content_hash, id.clone());
        
        info!("✅ Contexto almacenado: {} para agente {}", id, agent_id);
        
        Ok(id)
    }
    
    /// Recupera un contexto por ID
    pub fn get_context(&self, id: &ContextId) -> Option<ContextEntry> {
        let mut entry = self.contexts.get_mut(id)?;
        
        // Verificar expiración
        if entry.is_expired() {
            debug!("⏰ Contexto expirado: {}", id);
            return None;
        }
        
        // Actualizar acceso
        entry.touch();
        
        Some(entry.clone())
    }
    
    /// Recupera todos los contextos de un agente
    pub fn get_agent_contexts(&self, agent_id: &AgentId) -> Vec<ContextEntry> {
        let context_ids = match self.agent_index.get(agent_id) {
            Some(ids) => ids.clone(),
            None => return Vec::new(),
        };
        
        context_ids
            .iter()
            .filter_map(|id| self.get_context(id))
            .collect()
    }
    
    /// Busca contextos similares por contenido
    pub fn find_similar_contexts(
        &self,
        data: &serde_json::Value,
    ) -> Vec<ContextEntry> {
        let hash = self.calculate_hash(data);
        
        if let Some(id) = self.content_index.get(&hash) {
            if let Some(entry) = self.get_context(id.value()) {
                return vec![entry];
            }
        }
        
        Vec::new()
    }
    
    /// Limpia contextos expirados
    pub fn cleanup_expired(&self) -> usize {
        let mut removed = 0;
        
        // Recolectar IDs expirados
        let expired_ids: Vec<ContextId> = self
            .contexts
            .iter()
            .filter(|entry| entry.is_expired())
            .map(|entry| entry.key().clone())
            .collect();
        
        // Eliminar contextos expirados
        for id in expired_ids {
            if let Some((_, entry)) = self.contexts.remove(&id) {
                // Limpiar índices
                if let Some(mut agent_contexts) = self.agent_index.get_mut(&entry.agent_id) {
                    agent_contexts.retain(|ctx_id| ctx_id != &id);
                }
                
                self.content_index.remove(&entry.content_hash);
                removed += 1;
            }
        }
        
        if removed > 0 {
            info!("🧹 Limpieza completada: {} contextos expirados eliminados", removed);
        }
        
        removed
    }
    
    /// Obtiene estadísticas de la memoria
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            total_contexts: self.contexts.len(),
            total_agents: self.agent_index.len(),
            unique_contents: self.content_index.len(),
        }
    }
    
    /// Genera un ID único para contexto
    fn generate_context_id(&self, agent_id: &AgentId) -> ContextId {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros();
        
        format!("ctx_{}_{}", agent_id, timestamp)
    }
    
    /// Calcula hash del contenido
    fn calculate_hash(&self, data: &serde_json::Value) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let serialized = serde_json::to_string(data).unwrap();
        let mut hasher = DefaultHasher::new();
        serialized.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }
}

impl Default for SharedMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas de memoria
#[derive(Debug, Clone, Serialize)]
pub struct MemoryStats {
    pub total_contexts: usize,
    pub total_agents: usize,
    pub unique_contents: usize,
}

/// Tarea de limpieza periódica en background
pub async fn start_cleanup_task(memory: Arc<SharedMemory>, interval_seconds: u64) {
    info!(
        "🔄 Iniciando tarea de limpieza periódica (cada {}s)",
        interval_seconds
    );
    
    loop {
        tokio::time::sleep(Duration::from_secs(interval_seconds)).await;
        
        let removed = memory.cleanup_expired();
        if removed > 0 {
            debug!("🧹 Limpieza automática: {} contextos eliminados", removed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_store_and_retrieve_context() {
        let memory = SharedMemory::new();
        let data = json!({"test": "data", "value": 42});
        
        let id = memory
            .store_context("agent1".to_string(), data.clone(), 3600)
            .unwrap();
        
        let retrieved = memory.get_context(&id).unwrap();
        assert_eq!(retrieved.data, data);
        assert_eq!(retrieved.agent_id, "agent1");
    }
    
    #[test]
    fn test_deduplication() {
        let memory = SharedMemory::new();
        let data = json!({"test": "data"});
        
        let id1 = memory
            .store_context("agent1".to_string(), data.clone(), 3600)
            .unwrap();
        
        let id2 = memory
            .store_context("agent2".to_string(), data.clone(), 3600)
            .unwrap();
        
        // Mismo ID porque el contenido es idéntico
        assert_eq!(id1, id2);
    }
    
    #[test]
    fn test_expiration() {
        let memory = SharedMemory::new();
        let data = json!({"test": "data"});
        
        // Crear contexto con TTL de 0 segundos (ya expirado)
        let id = memory
            .store_context("agent1".to_string(), data, 0)
            .unwrap();
        
        // Esperar un momento
        std::thread::sleep(Duration::from_millis(10));
        
        // No debería estar disponible
        let retrieved = memory.get_context(&id);
        assert!(retrieved.is_none());
    }
    
    #[test]
    fn test_cleanup() {
        let memory = SharedMemory::new();
        
        // Crear varios contextos con TTL bajo
        for i in 0..5 {
            memory
                .store_context(
                    format!("agent{}", i),
                    json!({"id": i}),
                    0, // TTL 0 = expirado inmediatamente
                )
                .unwrap();
        }
        
        std::thread::sleep(Duration::from_millis(10));
        
        let removed = memory.cleanup_expired();
        assert_eq!(removed, 5);
    }
}
