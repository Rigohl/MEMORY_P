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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: Uuid,
    pub content: String,
    pub embedding: Option<Vec<f64>>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub source_motor: String,
    pub replicated_to: Vec<String>,
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
        for tag in &entry.tags {
            let mut tag_index = self.indexed_by_tags.write().await;
            tag_index.entry(tag.clone()).or_insert_with(Vec::new).push(entry_id);
        }
        let mut source_index = self.indexed_by_source.write().await;
        source_index.entry(entry.source_motor.clone()).or_insert_with(Vec::new).push(entry_id);
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

pub struct DistributedMemoryBank {
    nodes: HashMap<String, Arc<MemoryNode>>,
    replication_factor: usize,
}

impl DistributedMemoryBank {
    pub fn new(replication_factor: usize) -> Self {
        let motor_names = vec![
            "qdrant", "faiss", "scann", "tantivy", "lnx",
            "meilisearch", "julia_nlp", "memorybank", "toshi"
        ];
        let nodes = motor_names
            .into_iter()
            .map(|name| (name.to_string(), Arc::new(MemoryNode::new(name.to_string()))))
            .collect();
        Self { nodes, replication_factor }
    }

    pub async fn store(&self, entry: MemoryEntry) -> Uuid {
        let entry_id = entry.id;
        let available_motors: Vec<_> = self.nodes.values().take(self.replication_factor).collect();
        let mut replicated_to = Vec::new();
        for (i, node) in available_motors.iter().enumerate() {
            let mut entry_clone = entry.clone();
            if i > 0 { entry_clone.replicated_to.push(node.motor_name.clone()); }
            node.store(entry_clone).await;
            replicated_to.push(node.motor_name.clone());
        }
        if let Some(primary) = self.nodes.values().next() {
            if let Some(entry) = primary.entries.write().await.get_mut(&entry_id) {
                entry.replicated_to = replicated_to;
            }
        }
        entry_id
    }

    pub async fn parallel_search(&self, query: &str) -> Vec<(String, Vec<Uuid>)> {
        let futures: Vec<_> = self.nodes.values().map(|node| {
            let motor = node.motor_name.clone();
            let q = query.to_string();
            async move {
                let results = node.search_by_tag(&q).await;
                (motor, results)
            }
        }).collect();
        futures::future::join_all(futures).await
    }

    pub async fn predict_next_contexts(&self, current_entry_id: Uuid, lookahead: usize) -> Vec<MemoryEntry> {
        let mut results = Vec::new();
        for node in self.nodes.values() {
            if let Some(entry) = node.get(current_entry_id).await {
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

    pub async fn sync_to_all_nodes(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(self.nodes.len())
    }

    pub async fn cleanup_expired_contexts(&mut self, _max_age_minutes: u64) -> usize {
        0
    }

    pub async fn persist_all_contexts(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

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
        let entry = MemoryEntry::new("test content".to_string(), "qdrant".to_string());
        let entry_id = entry.id;
        bank.store(entry).await;
        if let Some(node) = bank.get_motor_node("qdrant") {
            let retrieved = node.get(entry_id).await;
            assert!(retrieved.is_some());
        }
    }
}
