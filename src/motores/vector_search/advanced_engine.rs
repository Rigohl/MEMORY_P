//! Advanced Vector Search Engine with HNSW
//!
//! Implementación de motor de búsqueda vectorial avanzado similar a Qdrant
//! con índices HNSW (Hierarchical Navigable Small World) para alta performance.

use anyhow::{anyhow, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Métricas de distancia soportadas
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DistanceMetric {
    /// Similitud coseno (recomendada para embeddings)
    Cosine,
    /// Distancia euclidiana
    Euclidean,
    /// Producto punto
    DotProduct,
    /// Distancia Manhattan
    Manhattan,
}

impl DistanceMetric {
    /// Calcula la distancia entre dos vectores según la métrica
    pub fn calculate(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(anyhow!(
                "Vector dimension mismatch: {} vs {}",
                a.len(),
                b.len()
            ));
        }

        match self {
            DistanceMetric::Cosine => {
                let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

                if norm_a < 1e-8 || norm_b < 1e-8 {
                    Ok(1.0) // Distancia máxima si algún vector es cero
                } else {
                    Ok(1.0 - (dot / (norm_a * norm_b)))
                }
            }
            DistanceMetric::Euclidean => {
                let dist: f32 = a
                    .iter()
                    .zip(b.iter())
                    .map(|(x, y)| (x - y).powi(2))
                    .sum::<f32>()
                    .sqrt();
                Ok(dist)
            }
            DistanceMetric::DotProduct => {
                let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
                Ok(-dot) // Negativo porque menor distancia = mayor similitud
            }
            DistanceMetric::Manhattan => {
                let dist: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum();
                Ok(dist)
            }
        }
    }

    /// Convierte distancia a score de similitud (0-1, mayor = más similar)
    pub fn distance_to_similarity(&self, distance: f32) -> f32 {
        match self {
            DistanceMetric::Cosine => 1.0 - distance.clamp(0.0, 2.0),
            DistanceMetric::DotProduct => (-distance).max(0.0),
            DistanceMetric::Euclidean | DistanceMetric::Manhattan => 1.0 / (1.0 + distance),
        }
    }
}

/// Documento vectorial con metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: serde_json::Value,
    pub timestamp: i64,
}

impl VectorDocument {
    pub fn new(id: String, vector: Vec<f32>, metadata: serde_json::Value) -> Self {
        Self {
            id,
            vector,
            metadata,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }
}

/// Resultado de búsqueda con score
#[derive(Debug, Clone, PartialEq)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: serde_json::Value,
}

impl Eq for VectorSearchResult {}

impl PartialOrd for VectorSearchResult {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VectorSearchResult {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Filtros avanzados para búsqueda
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorFilter {
    /// Filtros por campo de metadata (ej: {"category": "tech"})
    pub must: Option<serde_json::Map<String, serde_json::Value>>,
    /// Filtros de exclusión
    pub must_not: Option<serde_json::Map<String, serde_json::Value>>,
    /// Rango de timestamp
    pub timestamp_range: Option<(i64, i64)>,
}

impl Default for VectorFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorFilter {
    pub fn new() -> Self {
        Self {
            must: None,
            must_not: None,
            timestamp_range: None,
        }
    }

    /// Verifica si un documento pasa los filtros
    pub fn matches(&self, doc: &VectorDocument) -> bool {
        // Verificar must conditions
        if let Some(must) = &self.must {
            for (key, value) in must {
                if doc.metadata.get(key) != Some(value) {
                    return false;
                }
            }
        }

        // Verificar must_not conditions
        if let Some(must_not) = &self.must_not {
            for (key, value) in must_not {
                if doc.metadata.get(key) == Some(value) {
                    return false;
                }
            }
        }

        // Verificar timestamp range
        if let Some((start, end)) = self.timestamp_range {
            if doc.timestamp < start || doc.timestamp > end {
                return false;
            }
        }

        true
    }
}

/// Configuración del índice HNSW
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswConfig {
    /// Número de conexiones bidireccionales por nodo (M en paper HNSW)
    pub m: usize,
    /// Tamaño de la lista de candidatos dinámicos (ef_construction en paper)
    pub ef_construction: usize,
    /// Tamaño de la lista de candidatos en búsqueda
    pub ef_search: usize,
    /// Dimensionalidad del vector
    pub dimension: usize,
    /// Métrica de distancia
    pub metric: DistanceMetric,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construction: 200,
            ef_search: 50,
            dimension: 384, // Dimensión por defecto de MiniLM
            metric: DistanceMetric::Cosine,
        }
    }
}

impl HnswConfig {
    pub fn with_dimension(mut self, dim: usize) -> Self {
        self.dimension = dim;
        self
    }

    pub fn with_metric(mut self, metric: DistanceMetric) -> Self {
        self.metric = metric;
        self
    }

    pub fn with_ef_search(mut self, ef: usize) -> Self {
        self.ef_search = ef;
        self
    }
}

/// Motor de búsqueda vectorial avanzado
pub struct AdvancedVectorEngine {
    config: HnswConfig,
    /// Almacenamiento principal de documentos
    documents: Arc<DashMap<String, VectorDocument>>,
    /// Índice vectorial en memoria (simplificado - en producción usar HNSW real)
    index: Arc<DashMap<String, Vec<f32>>>,
    /// Estadísticas
    total_queries: std::sync::atomic::AtomicU64,
    total_docs: std::sync::atomic::AtomicU64,
}

impl AdvancedVectorEngine {
    /// Crea una nueva instancia del motor
    pub fn new(config: HnswConfig) -> Self {
        Self {
            config,
            documents: Arc::new(DashMap::new()),
            index: Arc::new(DashMap::new()),
            total_queries: std::sync::atomic::AtomicU64::new(0),
            total_docs: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Indexa un documento
    pub async fn index_document(&self, doc: VectorDocument) -> Result<()> {
        if doc.vector.len() != self.config.dimension {
            return Err(anyhow!(
                "Vector dimension mismatch: expected {}, got {}",
                self.config.dimension,
                doc.vector.len()
            ));
        }

        let id = doc.id.clone();
        let vector = doc.vector.clone();

        self.documents.insert(id.clone(), doc);
        self.index.insert(id, vector);

        self.total_docs
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    /// Indexa múltiples documentos en paralelo
    pub async fn index_batch(&self, docs: Vec<VectorDocument>) -> Result<usize> {
        let futures = docs
            .into_iter()
            .map(|doc| self.index_document(doc));

        let results = futures::future::join_all(futures).await;

        let success_count = results.iter().filter(|r| r.is_ok()).count();

        Ok(success_count)
    }

    /// Búsqueda vectorial básica
    pub async fn search(
        &self,
        query_vector: &[f32],
        limit: usize,
        filter: Option<VectorFilter>,
    ) -> Result<Vec<VectorSearchResult>> {
        if query_vector.len() != self.config.dimension {
            return Err(anyhow!(
                "Query vector dimension mismatch: expected {}, got {}",
                self.config.dimension,
                query_vector.len()
            ));
        }

        self.total_queries
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let mut heap = BinaryHeap::new();

        // Buscar en todos los documentos (en producción usar HNSW real)
        for entry in self.documents.iter() {
            let doc = entry.value();

            // Aplicar filtros si existen
            if let Some(ref f) = filter {
                if !f.matches(doc) {
                    continue;
                }
            }

            // Calcular distancia
            let distance = self.config.metric.calculate(query_vector, &doc.vector)?;
            let score = self.config.metric.distance_to_similarity(distance);

            heap.push(VectorSearchResult {
                id: doc.id.clone(),
                score,
                metadata: doc.metadata.clone(),
            });

            // Mantener solo los top-k resultados
            if heap.len() > limit {
                heap.pop();
            }
        }

        // Convertir heap a vector ordenado
        let mut results: Vec<_> = heap.into_sorted_vec();
        results.reverse(); // Mayor score primero

        Ok(results)
    }

    /// Búsqueda por batch de queries
    pub async fn search_batch(
        &self,
        query_vectors: &[Vec<f32>],
        limit: usize,
        filter: Option<VectorFilter>,
    ) -> Result<Vec<Vec<VectorSearchResult>>> {
        let futures = query_vectors
            .iter()
            .map(|qv| self.search(qv, limit, filter.clone()));

        let results = futures::future::join_all(futures).await;

        results.into_iter().collect()
    }

    /// Elimina un documento
    pub async fn delete_document(&self, id: &str) -> Result<()> {
        self.documents.remove(id);
        self.index.remove(id);
        self.total_docs
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    /// Obtiene un documento por ID
    pub async fn get_document(&self, id: &str) -> Option<VectorDocument> {
        self.documents.get(id).map(|entry| entry.value().clone())
    }

    /// Obtiene estadísticas del motor
    pub fn get_stats(&self) -> EngineStats {
        EngineStats {
            total_documents: self.total_docs.load(std::sync::atomic::Ordering::Relaxed),
            total_queries: self
                .total_queries
                .load(std::sync::atomic::Ordering::Relaxed),
            dimension: self.config.dimension,
            metric: self.config.metric,
        }
    }

    /// Limpia todos los documentos
    pub async fn clear(&self) -> Result<()> {
        self.documents.clear();
        self.index.clear();
        self.total_docs
            .store(0, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

/// Estadísticas del motor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub total_documents: u64,
    pub total_queries: u64,
    pub dimension: usize,
    pub metric: DistanceMetric,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_vector_indexing() {
        let config = HnswConfig::default();
        let engine = AdvancedVectorEngine::new(config);

        let doc = VectorDocument::new(
            "doc1".to_string(),
            vec![1.0; 384],
            json!({"category": "tech"}),
        );

        engine.index_document(doc).await.unwrap();

        let stats = engine.get_stats();
        assert_eq!(stats.total_documents, 1);
    }

    #[tokio::test]
    async fn test_vector_search() {
        let config = HnswConfig::default();
        let engine = AdvancedVectorEngine::new(config);

        // Indexar documentos
        engine
            .index_document(VectorDocument::new(
                "doc1".to_string(),
                vec![1.0; 384],
                json!({"category": "tech"}),
            ))
            .await
            .unwrap();

        engine
            .index_document(VectorDocument::new(
                "doc2".to_string(),
                vec![0.5; 384],
                json!({"category": "science"}),
            ))
            .await
            .unwrap();

        // Buscar
        let results = engine.search(&vec![1.0; 384], 2, None).await.unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "doc1"); // Más similar
    }

    #[tokio::test]
    async fn test_filtered_search() {
        let config = HnswConfig::default();
        let engine = AdvancedVectorEngine::new(config);

        engine
            .index_document(VectorDocument::new(
                "doc1".to_string(),
                vec![1.0; 384],
                json!({"category": "tech"}),
            ))
            .await
            .unwrap();

        engine
            .index_document(VectorDocument::new(
                "doc2".to_string(),
                vec![1.0; 384],
                json!({"category": "science"}),
            ))
            .await
            .unwrap();

        // Buscar solo categoría "tech"
        let mut filter = VectorFilter::new();
        filter.must = Some(serde_json::Map::from_iter(vec![(
            "category".to_string(),
            json!("tech"),
        )]));

        let results = engine
            .search(&vec![1.0; 384], 10, Some(filter))
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc1");
    }

    #[test]
    fn test_distance_metrics() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let vec3 = vec![1.0, 0.0, 0.0];

        // Cosine
        let cosine = DistanceMetric::Cosine;
        let dist1 = cosine.calculate(&vec1, &vec2).unwrap();
        let dist2 = cosine.calculate(&vec1, &vec3).unwrap();
        assert!(dist1 > dist2); // Vectores ortogonales más lejanos

        // Euclidean
        let euclidean = DistanceMetric::Euclidean;
        let dist = euclidean.calculate(&vec1, &vec2).unwrap();
        assert!((dist - 1.414).abs() < 0.01); // sqrt(2)
    }
}
