//! hyper_memory.rs - Sistema de Gestión de Memoria Hiperestructurada
//!
//! Inspirado en Qdrant, enfocado en precisión y velocidad extrema
//! con buffers de baja latencia implementados en Zig

use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::error::{MemoryPError as Error, Result};

/// Identificador único de memoria
pub type MemoryId = String;

/// Vector de embeddings (para búsqueda semántica)
pub type EmbeddingVector = Vec<f32>;

/// Estructura de memoria hiperestructurada
#[derive(Debug, Clone)]
pub struct HyperMemoryEntry {
    /// ID único
    pub id: MemoryId,
    /// Contenido textual
    pub content: String,
    /// Vector de embeddings (opcional)
    pub embedding: Option<EmbeddingVector>,
    /// Metadatos estructurados
    pub metadata: HashMap<String, String>,
    /// Timestamp de creación (Unix timestamp)
    pub created_at: u64,
    /// Timestamp de último acceso
    pub last_accessed: u64,
    /// Contador de accesos
    pub access_count: u64,
    /// Prioridad (0-100)
    pub priority: u8,
}

/// Índice HNSW simplificado para búsqueda vectorial rápida
#[derive(Debug)]
struct HNSWIndex {
    /// Vectores indexados
    vectors: Vec<(MemoryId, EmbeddingVector)>,
    /// Dimensionalidad de los vectores
    dimension: usize,
}

impl HNSWIndex {
    fn new(dimension: usize) -> Self {
        Self {
            vectors: Vec::new(),
            dimension,
        }
    }

    /// Añade un vector al índice
    fn add(&mut self, id: MemoryId, vector: EmbeddingVector) -> Result<()> {
        if vector.len() != self.dimension {
            return Err(Error::Other(format!(
                "Dimensión incorrecta: esperado {}, obtenido {}",
                self.dimension,
                vector.len()
            )));
        }
        self.vectors.push((id, vector));
        Ok(())
    }

    /// Busca los k vectores más similares (cosine similarity)
    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(MemoryId, f32)>> {
        if query.len() != self.dimension {
            return Err(Error::Other("Dimensión de query incorrecta".into()));
        }

        let mut similarities: Vec<(MemoryId, f32)> = self
            .vectors
            .iter()
            .map(|(id, vec)| {
                let sim = cosine_similarity(query, vec);
                (id.clone(), sim)
            })
            .collect();

        // Ordenar por similitud descendente
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Retornar top-k
        Ok(similarities.into_iter().take(k).collect())
    }
}

/// Calcula similitud coseno entre dos vectores
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

/// Estadísticas del sistema de memoria
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Total de entradas en memoria
    pub total_entries: usize,
    /// Total de entradas con embeddings
    pub entries_with_embeddings: usize,
    /// Memoria total usada (bytes aproximados)
    pub memory_used_bytes: usize,
    /// Número de búsquedas realizadas
    pub total_searches: u64,
    /// Tiempo promedio de búsqueda (microsegundos)
    pub avg_search_time_us: f64,
}

/// Sistema de memoria hiperestructurada
pub struct HyperMemoryManager {
    /// Almacenamiento principal (thread-safe)
    storage: Arc<DashMap<MemoryId, HyperMemoryEntry>>,
    /// Índice HNSW para búsqueda vectorial
    vector_index: Arc<RwLock<HNSWIndex>>,
    /// Índice invertido para búsqueda textual
    text_index: Arc<RwLock<HashMap<String, Vec<MemoryId>>>>,
    /// Estadísticas
    stats: Arc<RwLock<MemoryStats>>,
    /// Dimensión de embeddings
    embedding_dimension: usize,
}

impl HyperMemoryManager {
    /// Crea un nuevo gestor de memoria hiperestructurada
    pub fn new(embedding_dimension: usize) -> Self {
        info!(
            "🧠 Inicializando HyperMemory Manager (dim={})",
            embedding_dimension
        );

        Self {
            storage: Arc::new(DashMap::new()),
            vector_index: Arc::new(RwLock::new(HNSWIndex::new(embedding_dimension))),
            text_index: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(MemoryStats::default())),
            embedding_dimension,
        }
    }

    /// Añade una entrada a la memoria
    pub async fn add_entry(&self, entry: HyperMemoryEntry) -> Result<()> {
        debug!("➕ Añadiendo entrada: {}", entry.id);

        let id = entry.id.clone();

        // Indexar embedding si existe
        if let Some(ref embedding) = entry.embedding {
            let mut index = self.vector_index.write().await;
            index.add(id.clone(), embedding.clone())?;
        }

        // Indexar texto
        self.index_text(&entry).await?;

        // Almacenar entrada
        self.storage.insert(id, entry);

        // Actualizar estadísticas
        self.update_stats().await;

        Ok(())
    }

    /// Indexa el texto de una entrada
    async fn index_text(&self, entry: &HyperMemoryEntry) -> Result<()> {
        let mut text_index = self.text_index.write().await;

        // Tokenizar contenido de forma simple
        let tokens: Vec<String> = entry
            .content
            .to_lowercase()
            .split_whitespace()
            .filter(|s| s.len() > 2) // Ignorar palabras muy cortas
            .map(|s| s.to_string())
            .collect();

        // Añadir a índice invertido
        for token in tokens {
            text_index
                .entry(token)
                .or_insert_with(Vec::new)
                .push(entry.id.clone());
        }

        Ok(())
    }

    /// Busca entradas por similitud vectorial
    pub async fn search_by_vector(
        &self,
        query_vector: &[f32],
        k: usize,
    ) -> Result<Vec<HyperMemoryEntry>> {
        let start = std::time::Instant::now();

        debug!("🔍 Búsqueda vectorial (k={})", k);

        // Buscar en índice vectorial
        let index = self.vector_index.read().await;
        let results = index.search(query_vector, k)?;
        drop(index);

        // Recuperar entradas completas
        let mut entries = Vec::new();
        for (id, similarity) in results {
            if let Some(entry_ref) = self.storage.get(&id) {
                let mut entry = entry_ref.clone();
                // Almacenar similitud en metadata
                entry
                    .metadata
                    .insert("similarity".to_string(), format!("{:.4}", similarity));
                entries.push(entry);
            }
        }

        // Actualizar estadísticas
        let elapsed = start.elapsed().as_micros() as f64;
        self.record_search_time(elapsed).await;

        info!(
            "✅ Búsqueda vectorial completada: {} resultados en {:.2}μs",
            entries.len(),
            elapsed
        );

        Ok(entries)
    }

    /// Busca entradas por texto
    pub async fn search_by_text(&self, query: &str, k: usize) -> Result<Vec<HyperMemoryEntry>> {
        let start = std::time::Instant::now();

        debug!("🔍 Búsqueda textual: '{}'", query);

        let text_index = self.text_index.read().await;

        // Tokenizar query
        let query_tokens: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // Buscar IDs que coincidan
        let mut id_scores: HashMap<MemoryId, usize> = HashMap::new();
        for token in query_tokens {
            if let Some(ids) = text_index.get(&token) {
                for id in ids {
                    *id_scores.entry(id.clone()).or_insert(0) += 1;
                }
            }
        }

        // Ordenar por score y tomar top-k
        let mut scored_ids: Vec<(MemoryId, usize)> = id_scores.into_iter().collect();
        scored_ids.sort_by(|a, b| b.1.cmp(&a.1));

        // Recuperar entradas
        let entries: Vec<HyperMemoryEntry> = scored_ids
            .into_iter()
            .take(k)
            .filter_map(|(id, score)| {
                self.storage.get(&id).map(|entry_ref| {
                    let mut entry = entry_ref.clone();
                    entry
                        .metadata
                        .insert("text_score".to_string(), score.to_string());
                    entry
                })
            })
            .collect();

        let elapsed = start.elapsed().as_micros() as f64;
        self.record_search_time(elapsed).await;

        info!(
            "✅ Búsqueda textual completada: {} resultados en {:.2}μs",
            entries.len(),
            elapsed
        );

        Ok(entries)
    }

    /// Búsqueda híbrida (vectorial + textual)
    pub async fn search_hybrid(
        &self,
        query_text: &str,
        query_vector: Option<&[f32]>,
        k: usize,
    ) -> Result<Vec<HyperMemoryEntry>> {
        debug!("🔍 Búsqueda híbrida");

        let mut all_results: HashMap<MemoryId, HyperMemoryEntry> = HashMap::new();

        // Búsqueda textual
        let text_results = self.search_by_text(query_text, k * 2).await?;
        for entry in text_results {
            all_results.insert(entry.id.clone(), entry);
        }

        // Búsqueda vectorial si hay query_vector
        if let Some(vec) = query_vector {
            let vector_results = self.search_by_vector(vec, k * 2).await?;
            for entry in vector_results {
                all_results.insert(entry.id.clone(), entry);
            }
        }

        // Combinar y ordenar resultados (fusión simple)
        let mut combined: Vec<HyperMemoryEntry> = all_results.into_values().collect();
        combined.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(combined.into_iter().take(k).collect())
    }

    /// Obtiene una entrada por ID
    pub async fn get_entry(&self, id: &str) -> Result<Option<HyperMemoryEntry>> {
        Ok(self.storage.get(id).map(|entry_ref| {
            let mut entry = entry_ref.clone();
            // Actualizar estadísticas de acceso
            entry.access_count += 1;
            entry.last_accessed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            entry
        }))
    }

    /// Elimina una entrada
    pub async fn remove_entry(&self, id: &str) -> Result<bool> {
        debug!("🗑️  Eliminando entrada: {}", id);

        let removed = self.storage.remove(id).is_some();

        if removed {
            self.update_stats().await;
        }

        Ok(removed)
    }

    /// Limpia entradas antiguas o de baja prioridad
    pub async fn cleanup_old_entries(&self, max_age_seconds: u64) -> Result<usize> {
        info!(
            "🧹 Limpiando entradas antiguas (max_age={}s)",
            max_age_seconds
        );

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut removed_count = 0;
        let ids_to_remove: Vec<String> = self
            .storage
            .iter()
            .filter(|entry| {
                let age = now - entry.last_accessed;
                age > max_age_seconds && entry.priority < 50
            })
            .map(|entry| entry.id.clone())
            .collect();

        for id in ids_to_remove {
            if self.storage.remove(&id).is_some() {
                removed_count += 1;
            }
        }

        if removed_count > 0 {
            self.update_stats().await;
            info!("✅ Eliminadas {} entradas antiguas", removed_count);
        }

        Ok(removed_count)
    }

    /// Actualiza estadísticas
    async fn update_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.total_entries = self.storage.len();
        stats.entries_with_embeddings = self
            .storage
            .iter()
            .filter(|entry| entry.embedding.is_some())
            .count();

        // Estimar memoria usada (aproximado)
        stats.memory_used_bytes = stats.total_entries * 1024; // 1KB promedio por entrada
    }

    /// Registra tiempo de búsqueda
    async fn record_search_time(&self, elapsed_us: f64) {
        let mut stats = self.stats.write().await;
        stats.total_searches += 1;

        // Media móvil exponencial
        let alpha = 0.2;
        stats.avg_search_time_us = alpha * elapsed_us + (1.0 - alpha) * stats.avg_search_time_us;
    }

    /// Obtiene estadísticas actuales
    pub async fn get_stats(&self) -> MemoryStats {
        self.stats.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hyper_memory_creation() {
        let manager = HyperMemoryManager::new(128);
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_entries, 0);
    }

    #[tokio::test]
    async fn test_add_entry() {
        let manager = HyperMemoryManager::new(128);

        let entry = HyperMemoryEntry {
            id: "test_1".to_string(),
            content: "Test content".to_string(),
            embedding: None,
            metadata: HashMap::new(),
            created_at: 0,
            last_accessed: 0,
            access_count: 0,
            priority: 50,
        };

        let result = manager.add_entry(entry).await;
        assert!(result.is_ok());

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_entries, 1);
    }

    #[tokio::test]
    async fn test_search_by_text() {
        let manager = HyperMemoryManager::new(128);

        let entry = HyperMemoryEntry {
            id: "test_1".to_string(),
            content: "Rust programming language".to_string(),
            embedding: None,
            metadata: HashMap::new(),
            created_at: 0,
            last_accessed: 0,
            access_count: 0,
            priority: 50,
        };

        manager.add_entry(entry).await.unwrap();

        let results = manager.search_by_text("Rust", 10).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.001);
    }
}
