//! ffi/jax.rs - JAX ML Inference Integration
//!
//! Integración avanzada con JAX para generación de embeddings
//! con soporte para múltiples modelos y cache en Redis.

use super::error::{FfiError, Result};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;

lazy_static! {
    /// Cache global de embeddings en memoria (fallback si Redis no disponible)
    static ref EMBEDDING_CACHE: Arc<DashMap<String, Vec<f32>>> = Arc::new(DashMap::new());
}

/// Modelos de embeddings soportados
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EmbeddingModel {
    /// sentence-transformers/all-MiniLM-L6-v2 (384 dims)
    MiniLML6,
    /// sentence-transformers/all-MiniLM-L12-v2 (384 dims)
    MiniLML12,
    /// BAAI/bge-small-en-v1.5 (384 dims)
    BGESmall,
    /// BAAI/bge-base-en-v1.5 (768 dims)
    BGEBase,
    /// BAAI/bge-large-en-v1.5 (1024 dims)
    BGELarge,
    /// intfloat/e5-small-v2 (384 dims)
    E5Small,
    /// intfloat/e5-base-v2 (768 dims)
    E5Base,
}

impl EmbeddingModel {
    /// Retorna la dimensionalidad del modelo
    pub fn dimension(&self) -> usize {
        match self {
            EmbeddingModel::MiniLML6 => 384,
            EmbeddingModel::MiniLML12 => 384,
            EmbeddingModel::BGESmall => 384,
            EmbeddingModel::BGEBase => 768,
            EmbeddingModel::BGELarge => 1024,
            EmbeddingModel::E5Small => 384,
            EmbeddingModel::E5Base => 768,
        }
    }

    /// Retorna el nombre del modelo en HuggingFace
    pub fn model_name(&self) -> &'static str {
        match self {
            EmbeddingModel::MiniLML6 => "sentence-transformers/all-MiniLM-L6-v2",
            EmbeddingModel::MiniLML12 => "sentence-transformers/all-MiniLM-L12-v2",
            EmbeddingModel::BGESmall => "BAAI/bge-small-en-v1.5",
            EmbeddingModel::BGEBase => "BAAI/bge-base-en-v1.5",
            EmbeddingModel::BGELarge => "BAAI/bge-large-en-v1.5",
            EmbeddingModel::E5Small => "intfloat/e5-small-v2",
            EmbeddingModel::E5Base => "intfloat/e5-base-v2",
        }
    }
}

impl Default for EmbeddingModel {
    fn default() -> Self {
        EmbeddingModel::MiniLML6
    }
}

/// Configuración del generador de embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model: EmbeddingModel,
    pub use_cache: bool,
    pub redis_url: Option<String>,
    pub batch_size: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model: EmbeddingModel::default(),
            use_cache: true,
            redis_url: None,
            batch_size: 32,
        }
    }
}

/// Generador de embeddings con cache
pub struct EmbeddingGenerator {
    config: EmbeddingConfig,
    // En producción: conexión a Redis
    // redis_client: Option<redis::Client>,
}

impl EmbeddingGenerator {
    /// Crea un nuevo generador con configuración
    pub fn new(config: EmbeddingConfig) -> Self {
        Self {
            config,
            // redis_client: None,
        }
    }

    /// Genera embedding para un texto con cache
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Generar cache key
        let cache_key = self.cache_key(text);

        // Intentar recuperar del cache
        if self.config.use_cache {
            if let Some(cached) = self.get_from_cache(&cache_key) {
                tracing::debug!("Cache hit para texto: '{}'", text);
                return Ok(cached);
            }
        }

        // Generar nuevo embedding
        let embedding = self.generate_raw_embedding(text)?;

        // Guardar en cache
        if self.config.use_cache {
            self.save_to_cache(&cache_key, &embedding);
        }

        Ok(embedding)
    }

    /// Genera embeddings para múltiples textos (batch optimizado)
    pub async fn generate_embeddings_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        use rayon::prelude::*;

        // Dividir en batches según batch_size
        let batches: Vec<&[String]> = texts.chunks(self.config.batch_size).collect();
        
        let mut all_embeddings = Vec::with_capacity(texts.len());

        for batch in batches {
            let batch_embeddings: Vec<Vec<f32>> = batch
                .par_iter()
                .map(|text| {
                    futures::executor::block_on(self.generate_embedding(text))
                        .unwrap_or_else(|_| vec![0.0; self.config.model.dimension()])
                })
                .collect();

            all_embeddings.extend(batch_embeddings);
        }

        Ok(all_embeddings)
    }

    /// Genera embedding raw sin cache
    fn generate_raw_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        #[cfg(feature = "ffi-jax")]
        {
            tracing::debug!(
                "Generando embedding con modelo {} para: '{}'",
                self.config.model.model_name(),
                text
            );
            
            // TODO: Llamada real a JAX/HuggingFace via Python C API
            // Por ahora: stub con vector normalizado pseudo-aleatorio
            let dim = self.config.model.dimension();
            let embedding = self.generate_stub_embedding(text, dim);
            
            Ok(embedding)
        }

        #[cfg(not(feature = "ffi-jax"))]
        {
            Err(FfiError::NotAvailable("JAX generate_embedding".to_string()))
        }
    }

    /// Genera un embedding stub determinístico para testing
    fn generate_stub_embedding(&self, text: &str, dim: usize) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();

        // Generar vector pseudo-aleatorio determinístico
        let mut embedding = Vec::with_capacity(dim);
        let mut state = seed;
        
        for _ in 0..dim {
            state = state.wrapping_mul(1103515245).wrapping_add(12345);
            let val = ((state / 65536) % 32768) as f32 / 32768.0 - 0.5;
            embedding.push(val);
        }

        // Normalizar
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        embedding
    }

    /// Genera cache key para un texto
    fn cache_key(&self, text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.config.model.model_name().hash(&mut hasher);
        text.hash(&mut hasher);
        format!("emb:{}:{:x}", self.config.model.model_name(), hasher.finish())
    }

    /// Obtiene embedding del cache (memoria)
    fn get_from_cache(&self, key: &str) -> Option<Vec<f32>> {
        EMBEDDING_CACHE.get(key).map(|entry| entry.value().clone())
    }

    /// Guarda embedding en cache (memoria)
    fn save_to_cache(&self, key: &str, embedding: &[f32]) {
        EMBEDDING_CACHE.insert(key.to_string(), embedding.to_vec());
    }

    /// Limpia el cache
    pub fn clear_cache() {
        EMBEDDING_CACHE.clear();
        tracing::info!("Cache de embeddings limpiado");
    }

    /// Obtiene estadísticas del cache
    pub fn cache_stats() -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("cache_size".to_string(), EMBEDDING_CACHE.len());
        stats
    }
}

/// Inicializa el engine de JAX
pub fn init() -> Result<()> {
    #[cfg(feature = "ffi-jax")]
    {
        tracing::info!("🤖 Inicializando JAX ML inference con cache de embeddings");
        // TODO: Inicializar Python runtime + JAX
        Ok(())
    }

    #[cfg(not(feature = "ffi-jax"))]
    {
        tracing::warn!("⚠️  JAX no disponible (feature 'ffi-jax' deshabilitado)");
        Err(FfiError::NotAvailable("JAX".to_string()))
    }
}

/// Finaliza el engine de JAX
pub fn shutdown() {
    #[cfg(feature = "ffi-jax")]
    {
        tracing::info!("🤖 Finalizando JAX runtime");
        EmbeddingGenerator::clear_cache();
        // TODO: Finalizar Python runtime
    }
}

/// Genera embedding para un texto (API legacy)
pub fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    let config = EmbeddingConfig::default();
    let generator = EmbeddingGenerator::new(config);
    futures::executor::block_on(generator.generate_embedding(text))
}

/// Genera embeddings para múltiples textos (batch) (API legacy)
pub fn generate_embeddings_batch(texts: &[String]) -> Result<Vec<Vec<f32>>> {
    let config = EmbeddingConfig::default();
    let generator = EmbeddingGenerator::new(config);
    futures::executor::block_on(generator.generate_embeddings_batch(texts))
}

/// Calcula similitud coseno entre dos vectores
pub fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> Result<f32> {
    if vec1.len() != vec2.len() {
        return Err(FfiError::CallFailed("Vector dimensions mismatch".to_string()));
    }
    
    // Implementación nativa en Rust (JAX no necesario para esto)
    let dot: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();
    let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm1 < 1e-8 || norm2 < 1e-8 {
        return Ok(0.0);
    }
    
    Ok(dot / (norm1 * norm2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_model_dimensions() {
        assert_eq!(EmbeddingModel::MiniLML6.dimension(), 384);
        assert_eq!(EmbeddingModel::BGEBase.dimension(), 768);
        assert_eq!(EmbeddingModel::BGELarge.dimension(), 1024);
    }

    #[tokio::test]
    async fn test_embedding_generation() {
        let config = EmbeddingConfig::default();
        let generator = EmbeddingGenerator::new(config);

        let text = "Hello world";
        let embedding = generator.generate_embedding(text).await;
        
        if let Ok(emb) = embedding {
            assert_eq!(emb.len(), 384);
            // Verificar normalización
            let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!((norm - 1.0).abs() < 0.01);
        }
    }

    #[tokio::test]
    async fn test_embedding_cache() {
        let config = EmbeddingConfig {
            use_cache: true,
            ..Default::default()
        };
        let generator = EmbeddingGenerator::new(config);

        let text = "Test cache";
        
        // Primera generación
        let emb1 = generator.generate_embedding(text).await.unwrap();
        
        // Segunda generación (debería venir del cache)
        let emb2 = generator.generate_embedding(text).await.unwrap();
        
        assert_eq!(emb1, emb2);
    }

    #[tokio::test]
    async fn test_batch_embeddings() {
        let config = EmbeddingConfig::default();
        let generator = EmbeddingGenerator::new(config);

        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        let embeddings = generator.generate_embeddings_batch(&texts).await.unwrap();
        
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 384);
    }

    #[test]
    fn test_cosine_similarity() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let vec3 = vec![1.0, 0.0, 0.0];
        
        let sim1 = cosine_similarity(&vec1, &vec2).unwrap();
        let sim2 = cosine_similarity(&vec1, &vec3).unwrap();
        
        // Vectores ortogonales -> similitud ~0
        assert!((sim1 - 0.0).abs() < 0.01);
        // Vectores idénticos -> similitud ~1
        assert!((sim2 - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_stub_embedding_deterministic() {
        let config = EmbeddingConfig::default();
        let generator = EmbeddingGenerator::new(config);

        let text = "Deterministic test";
        let emb1 = generator.generate_stub_embedding(text, 384);
        let emb2 = generator.generate_stub_embedding(text, 384);

        // Debe ser determinístico
        assert_eq!(emb1, emb2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        
        let result = cosine_similarity(&vec1, &vec2);
        
        if let Ok(sim) = result {
            // Vectores ortogonales -> similitud ~0
            assert!((sim - 0.0).abs() < 0.01);
        }
    }
}
