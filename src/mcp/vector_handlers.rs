//! MCP Handlers para búsqueda vectorial avanzada
//!
//! Implementa los handlers MCP para:
//! - map_search: Búsqueda vectorial con filtros avanzados
//! - index_documents: Indexación de documentos con embeddings
//! - similar_docs: Encontrar documentos similares
//! - embedding_stats: Estadísticas del sistema de embeddings

use crate::ffi::jax::{EmbeddingConfig, EmbeddingGenerator, EmbeddingModel};
use crate::motores::vector_search::{
    AdvancedVectorEngine, DistanceMetric, HnswConfig, VectorDocument, VectorFilter,
};
use axum::Json;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static! {
    /// Instancia global del motor vectorial
    static ref VECTOR_ENGINE: Arc<RwLock<Option<AdvancedVectorEngine>>> = 
        Arc::new(RwLock::new(None));
    
    /// Instancia global del generador de embeddings
    static ref EMBEDDING_GEN: Arc<RwLock<Option<EmbeddingGenerator>>> =
        Arc::new(RwLock::new(None));
}

/// Inicializa el motor vectorial global (llamar al inicio)
pub async fn init_vector_engine(config: HnswConfig) {
    let engine = AdvancedVectorEngine::new(config);
    let mut lock = VECTOR_ENGINE.write().await;
    *lock = Some(engine);
    tracing::info!("✅ Motor vectorial inicializado");
}

/// Inicializa el generador de embeddings global
pub async fn init_embedding_generator(config: EmbeddingConfig) {
    let generator = EmbeddingGenerator::new(config);
    let mut lock = EMBEDDING_GEN.write().await;
    *lock = Some(generator);
    tracing::info!("✅ Generador de embeddings inicializado");
}

/// Request para búsqueda vectorial avanzada
#[derive(Debug, Deserialize, Serialize)]
pub struct MapSearchRequest {
    /// Query de texto (se convertirá a embedding)
    pub query: String,
    /// Número máximo de resultados
    pub limit: usize,
    /// Filtros por metadata (opcional)
    pub filters: Option<VectorFilter>,
    /// Modelo de embeddings a usar (opcional)
    pub model: Option<String>,
    /// Métrica de distancia (opcional)
    pub metric: Option<String>,
}

/// Response de búsqueda vectorial
#[derive(Debug, Serialize, Deserialize)]
pub struct MapSearchResponse {
    pub results: Vec<SearchResultItem>,
    pub total: usize,
    pub query_time_ms: u64,
    pub model_used: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub id: String,
    pub score: f32,
    pub metadata: Value,
}

/// Handler para búsqueda vectorial avanzada (tool: map_search)
pub async fn map_search_handler(
    Json(req): Json<MapSearchRequest>,
) -> Result<Json<MapSearchResponse>, (axum::http::StatusCode, String)> {
    let start = std::time::Instant::now();

    // Validar límite
    if req.limit == 0 || req.limit > 1000 {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "limit debe estar entre 1 y 1000".to_string(),
        ));
    }

    // Obtener o inicializar generador de embeddings
    let gen_lock = EMBEDDING_GEN.read().await;
    let generator = if let Some(ref gen) = *gen_lock {
        gen
    } else {
        drop(gen_lock);
        init_embedding_generator(EmbeddingConfig::default()).await;
        let gen_lock = EMBEDDING_GEN.read().await;
        gen_lock.as_ref().unwrap()
    };

    // Generar embedding de la query
    let query_vector = generator
        .generate_embedding(&req.query)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error generando embedding: {}", e),
            )
        })?;

    drop(gen_lock);

    // Obtener o inicializar motor vectorial
    let engine_lock = VECTOR_ENGINE.read().await;
    let engine = if let Some(ref eng) = *engine_lock {
        eng
    } else {
        drop(engine_lock);
        init_vector_engine(HnswConfig::default()).await;
        let engine_lock = VECTOR_ENGINE.read().await;
        engine_lock.as_ref().unwrap()
    };

    // Realizar búsqueda
    let results = engine
        .search(&query_vector, req.limit, req.filters)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error en búsqueda: {}", e),
            )
        })?;

    drop(engine_lock);

    let query_time_ms = start.elapsed().as_millis() as u64;

    Ok(Json(MapSearchResponse {
        total: results.len(),
        results: results
            .into_iter()
            .map(|r| SearchResultItem {
                id: r.id,
                score: r.score,
                metadata: r.metadata,
            })
            .collect(),
        query_time_ms,
        model_used: "MiniLM-L6-v2".to_string(),
    }))
}

/// Request para indexar documentos
#[derive(Debug, Deserialize, Serialize)]
pub struct IndexDocumentsRequest {
    pub documents: Vec<DocumentToIndex>,
    pub model: Option<String>,
    pub batch_size: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DocumentToIndex {
    pub id: String,
    pub text: String,
    pub metadata: Value,
}

/// Response de indexación
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexDocumentsResponse {
    pub indexed_count: usize,
    pub failed_count: usize,
    pub index_time_ms: u64,
}

/// Handler para indexar documentos (tool: index_documents)
pub async fn index_documents_handler(
    Json(req): Json<IndexDocumentsRequest>,
) -> Result<Json<IndexDocumentsResponse>, (axum::http::StatusCode, String)> {
    let start = std::time::Instant::now();

    if req.documents.is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "No se proporcionaron documentos".to_string(),
        ));
    }

    // Obtener o inicializar generador
    let gen_lock = EMBEDDING_GEN.read().await;
    let generator = if let Some(ref gen) = *gen_lock {
        gen
    } else {
        drop(gen_lock);
        init_embedding_generator(EmbeddingConfig::default()).await;
        let gen_lock = EMBEDDING_GEN.read().await;
        gen_lock.as_ref().unwrap()
    };

    // Generar embeddings en batch
    let texts: Vec<String> = req.documents.iter().map(|d| d.text.clone()).collect();
    let embeddings = generator
        .generate_embeddings_batch(&texts)
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error generando embeddings: {}", e),
            )
        })?;

    drop(gen_lock);

    // Crear documentos vectoriales
    let vector_docs: Vec<VectorDocument> = req
        .documents
        .into_iter()
        .zip(embeddings.into_iter())
        .map(|(doc, embedding)| VectorDocument::new(doc.id, embedding, doc.metadata))
        .collect();

    // Obtener o inicializar motor
    let engine_lock = VECTOR_ENGINE.read().await;
    let engine = if let Some(ref eng) = *engine_lock {
        eng
    } else {
        drop(engine_lock);
        init_vector_engine(HnswConfig::default()).await;
        let engine_lock = VECTOR_ENGINE.read().await;
        engine_lock.as_ref().unwrap()
    };

    // Indexar batch
    let total_docs = vector_docs.len();
    let indexed_count = engine.index_batch(vector_docs).await.map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error indexando: {}", e),
        )
    })?;

    drop(engine_lock);

    let index_time_ms = start.elapsed().as_millis() as u64;

    Ok(Json(IndexDocumentsResponse {
        indexed_count,
        failed_count: total_docs - indexed_count,
        index_time_ms,
    }))
}

/// Request para encontrar documentos similares
#[derive(Debug, Deserialize, Serialize)]
pub struct SimilarDocsRequest {
    /// ID del documento de referencia
    pub document_id: String,
    /// Número de resultados similares
    pub limit: usize,
    /// Filtros adicionales
    pub filters: Option<VectorFilter>,
}

/// Handler para encontrar documentos similares (tool: similar_docs)
pub async fn similar_docs_handler(
    Json(req): Json<SimilarDocsRequest>,
) -> Result<Json<MapSearchResponse>, (axum::http::StatusCode, String)> {
    let start = std::time::Instant::now();

    // Obtener motor
    let engine_lock = VECTOR_ENGINE.read().await;
    let engine = engine_lock.as_ref().ok_or((
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        "Motor vectorial no inicializado".to_string(),
    ))?;

    // Obtener documento de referencia
    let reference_doc = engine
        .get_document(&req.document_id)
        .await
        .ok_or((
            axum::http::StatusCode::NOT_FOUND,
            format!("Documento '{}' no encontrado", req.document_id),
        ))?;

    // Buscar similares usando el vector del documento
    let results = engine
        .search(&reference_doc.vector, req.limit + 1, req.filters) // +1 para excluir el mismo
        .await
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error en búsqueda: {}", e),
            )
        })?;

    drop(engine_lock);

    // Filtrar el documento de referencia de los resultados
    let filtered_results: Vec<_> = results
        .into_iter()
        .filter(|r| r.id != req.document_id)
        .take(req.limit)
        .collect();

    let query_time_ms = start.elapsed().as_millis() as u64;

    Ok(Json(MapSearchResponse {
        total: filtered_results.len(),
        results: filtered_results
            .into_iter()
            .map(|r| SearchResultItem {
                id: r.id,
                score: r.score,
                metadata: r.metadata,
            })
            .collect(),
        query_time_ms,
        model_used: "MiniLM-L6-v2".to_string(),
    }))
}

/// Handler para obtener estadísticas del sistema
pub async fn vector_stats_handler() -> Json<Value> {
    let engine_lock = VECTOR_ENGINE.read().await;
    
    let stats = if let Some(ref engine) = *engine_lock {
        let engine_stats = engine.get_stats();
        json!({
            "initialized": true,
            "total_documents": engine_stats.total_documents,
            "total_queries": engine_stats.total_queries,
            "dimension": engine_stats.dimension,
            "metric": format!("{:?}", engine_stats.metric),
            "cache_stats": crate::ffi::jax::EmbeddingGenerator::cache_stats(),
        })
    } else {
        json!({
            "initialized": false,
            "message": "Motor vectorial no inicializado"
        })
    };

    drop(engine_lock);
    Json(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_index_and_search() {
        // Inicializar motores
        init_vector_engine(HnswConfig::default()).await;
        init_embedding_generator(EmbeddingConfig::default()).await;

        // Indexar documentos de prueba
        let req = IndexDocumentsRequest {
            documents: vec![
                DocumentToIndex {
                    id: "doc1".to_string(),
                    text: "Rust programming language".to_string(),
                    metadata: json!({"category": "tech"}),
                },
                DocumentToIndex {
                    id: "doc2".to_string(),
                    text: "Python machine learning".to_string(),
                    metadata: json!({"category": "tech"}),
                },
            ],
            model: None,
            batch_size: None,
        };

        let response = index_documents_handler(Json(req)).await.unwrap();
        assert_eq!(response.0.indexed_count, 2);

        // Buscar
        let search_req = MapSearchRequest {
            query: "Rust language".to_string(),
            limit: 5,
            filters: None,
            model: None,
            metric: None,
        };

        let search_response = map_search_handler(Json(search_req)).await.unwrap();
        assert!(search_response.0.results.len() > 0);
    }
}
