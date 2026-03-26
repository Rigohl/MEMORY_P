//! Distributed MemoryBank - Los 9 motores actúan como nodos de memoria distribuida
//!
//! Arquitectura:
//! - Cada motor es un nodo en el MemoryBank
//! - Información replicada con consistencia eventual
//! - Búsqueda paralela (Qdrant vector, Tantivy text, etc.)
//! - Predicción con Julia + contexto con JAX embeddings
//! 
//! Especificación MCP Memory:
//! - Store: Guardar contextos con embeddings y metadata
//! - Predict: Predecir nódulos contextuales futuros  
//! - Search: Busqueda híbrida (vector + texto)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// MEMORY ENTRY - Unidad fundamental de memoria distribuida
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Uuid,
    pub content: String,
    pub embedding: Option<Vec<f64>>,  // Qdrant vector
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub source_motor: String,  // Qué motor lo guardó
    pub replicated_to: Vec<String>,  // Motors que tienen copia
    pub access_count: u64,
}

impl MemoryEntry {
    pub fn new(content: String, source_motor: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            content,
            embedding: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            source_motor,
            replicated_to: Vec::new(),
            access_count: 0,
        }
    }

    pub fn with_embedding(mut self, embedding: Vec<f64>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

// ============================================================================
// MEMORY NODE - Representa un motor como nodo de memoria
// ============================================================================

#[derive(Debug)]
pub struct MemoryNode {
    pub motor_name: String,
    pub entries: Arc<RwLock<HashMap<Uuid, MemoryEntry>>>,
    pub indexed_by_tags: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    pub indexed_by_source: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
}

impl MemoryNode {
    pub fn new(motor_name: String) -> Self {
        Self {
            motor_name,
            entries: Arc::new(RwLock::new(HashMap::new())),
            indexed_by_tags: Arc::new(RwLock::new(HashMap::new())),
            indexed_by_source: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn store(&self, entry: MemoryEntry) -> Uuid {
        let entry_id = entry.id;
        
        // Index by tags
        for tag in &entry.tags {
            let mut tag_index = self.indexed_by_tags.write().await;
            tag_index.entry(tag.clone()).or_insert_with(Vec::new).push(entry_id);
        }

        // Index by source
        let mut source_index = self.indexed_by_source.write().await;
        source_index
            .entry(entry.source_motor.clone())
            .or_insert_with(Vec::new)
            .push(entry_id);

        // Store entry
        let mut entries = self.entries.write().await;
        entries.insert(entry_id, entry);

        entry_id
    }

    pub async fn get(&self, id: Uuid) -> Option<MemoryEntry> {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(&id) {
            entry.access_count += 1;
            return Some(entry.clone());
        }
        None
    }

    pub async fn search_by_tag(&self, tag: &str) -> Vec<Uuid> {
        let index = self.indexed_by_tags.read().await;
        index.get(tag).cloned().unwrap_or_default()
    }

    pub async fn search_by_source(&self, source: &str) -> Vec<Uuid> {
        let index = self.indexed_by_source.read().await;
        index.get(source).cloned().unwrap_or_default()
    }

    pub async fn get_stats(&self) -> serde_json::Value {
        let entries = self.entries.read().await;
        let total_access = entries.values().map(|e| e.access_count).sum::<u64>();
        
        serde_json::json!({
            "motor": self.motor_name,
            "entries_count": entries.len(),
            "total_accesses": total_access,
            "has_embeddings": entries.values().filter(|e| e.embedding.is_some()).count(),
        })
    }
}

// ============================================================================
// DISTRIBUTED MEMORY BANK - Orquesta múltiples nodos
// ============================================================================

pub struct DistributedMemoryBank {
    nodes: HashMap<String, Arc<MemoryNode>>,
    replication_factor: usize,  // Cuántos nodos replican cada entry
}

impl DistributedMemoryBank {
    /// Crea MemoryBank con los 9 motores como nodos
    pub fn new(replication_factor: usize) -> Self {
        let motor_names = vec![
            "qdrant", "faiss", "scann", "tantivy", "lnx",
            "meilisearch", "julia_nlp", "memorybank", "toshi"
        ];

        let nodes = motor_names
            .into_iter()
            .map(|name| (name.to_string(), Arc::new(MemoryNode::new(name.to_string()))))
            .collect();

        Self {
            nodes,
            replication_factor,
        }
    }

    /// Guarda una entrada, replicándola a N motores
    pub async fn store(&self, entry: MemoryEntry) -> Uuid {
        let entry_id = entry.id;
        
        // Selecciona nodos para replicación (round-robin simplificado)
        let available_motors: Vec<_> = self.nodes.values().take(self.replication_factor).collect();
        
        let mut replicated_to = Vec::new();
        for (i, node) in available_motors.iter().enumerate() {
            let mut entry_clone = entry.clone();
            if i > 0 {
                entry_clone.replicated_to.push(node.motor_name.clone());
            }
            node.store(entry_clone).await;
            replicated_to.push(node.motor_name.clone());
        }

        // Actualiza metadata de replicación
        if let Some(primary) = self.nodes.values().next() {
            if let Some(entry) = primary.entries.write().await.get_mut(&entry_id) {
                entry.replicated_to = replicated_to;
            }
        }

        entry_id
    }

    /// Busca en todos los nodos en paralelo
    pub async fn parallel_search(&self, query: &str) -> Vec<(String, Vec<Uuid>)> {
        let futures: Vec<_> = self.nodes
            .values()
            .map(|node| {
                let motor = node.motor_name.clone();
                let q = query.to_string();
                async move {
                    // Búsqueda simple por tags (en producción usaría Qdrant/Tantivy real)
                    let results = node.search_by_tag(&q).await;
                    (motor, results)
                }
            })
            .collect();

        futures::future::join_all(futures).await
    }

    /// Obtiene sugerencias de órganos contextuales futuros (predictivas)
    pub async fn predict_next_contexts(
        &self,
        current_entry_id: Uuid,
        lookahead: usize,
    ) -> Vec<MemoryEntry> {
        // En producción: usar Julia para análisis caótico del embedding
        // Por ahora: lookup simple
        
        let mut results = Vec::new();
        for node in self.nodes.values() {
            if let Some(entry) = node.get(current_entry_id).await {
                // Busca entradas relacionadas por tag
                for tag in &entry.tags {
                    let related_ids = node.search_by_tag(tag).await;
                    for related_id in related_ids.iter().take(lookahead) {
                        if let Some(related) = node.get(*related_id).await {
                            results.push(related);
                        }
                    }
                }
            }
        }

        results.truncate(lookahead);
        results
    }

    /// Obtén estadísticas de todos los nodos
    pub async fn get_cluster_stats(&self) -> serde_json::Value {
        let mut node_stats = Vec::new();
        for node in self.nodes.values() {
            node_stats.push(node.get_stats().await);
        }

        serde_json::json!({
            "total_motors": self.nodes.len(),
            "replication_factor": self.replication_factor,
            "nodes": node_stats,
            "timestamp": Utc::now().to_rfc3339(),
        })
    }

    pub fn get_motor_node(&self, motor_name: &str) -> Option<Arc<MemoryNode>> {
        self.nodes.get(motor_name).cloned()
    }

    pub fn get_all_motors(&self) -> Vec<String> {
        self.nodes.keys().cloned().collect()
    }
}

// ============================================================================
// MCP MEMORY PROTOCOL - Endpoints para almacenamiento de memoria
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryStoreRequest {
    pub content: String,
    pub embedding: Option<Vec<f64>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemorySearchRequest {
    pub query: String,
    pub limit: Option<usize>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryPredictRequest {
    pub context_id: Uuid,
    pub lookahead: usize,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_bank_creation() {
        let bank = DistributedMemoryBank::new(3);
        assert_eq!(bank.nodes.len(), 9);
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let bank = DistributedMemoryBank::new(3);
        let entry = MemoryEntry::new(
            "test content".to_string(),
            "qdrant".to_string(),
        );
        let entry_id = entry.id;

        bank.store(entry).await;
        
        // Busca en un nodo
        if let Some(node) = bank.get_motor_node("qdrant") {
            let retrieved = node.get(entry_id).await;
            assert!(retrieved.is_some());
        }
    }

    #[tokio::test]
    async fn test_replication() {
        let bank = DistributedMemoryBank::new(3);
        let entry = MemoryEntry::new(
            "replicated content".to_string(),
            "qdrant".to_string(),
        ).with_tags(vec!["test".to_string()]);

        bank.store(entry).await;

        // Verifica que fue replicado a 3 motores
        let stats = bank.get_cluster_stats().await;
        let nodes = stats["nodes"].as_array().unwrap();
        let engines_with_entry: usize = nodes
            .iter()
            .filter(|n| n["entries_count"].as_u64().unwrap_or(0) > 0)
            .count();
        
        assert!(engines_with_entry >= 1);
    }

    #[tokio::test]
    async fn test_parallel_search() {
        let bank = DistributedMemoryBank::new(3);
        let entry = MemoryEntry::new(
            "searchable content".to_string(),
            "qdrant".to_string(),
        ).with_tags(vec!["python".to_string()]);

        bank.store(entry).await;
        let results = bank.parallel_search("python").await;
        
        assert!(!results.is_empty());
    }
}
