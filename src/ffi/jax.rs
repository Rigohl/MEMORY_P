//! ffi/jax.rs - JAX ML Inference Integration
//! REAL FFI Implementation connecting to JAX shared library (via Python bridge)

use super::error::{FfiError, Result};
use dashmap::DashMap;
use lazy_static::lazy_static;

#[cfg(feature = "ffi-jax")]
#[link(name = "jax_ffi", kind = "dylib")]
extern "C" {
    fn jax_init_ffi() -> std::ffi::c_int;
    fn jax_shutdown_ffi() -> std::ffi::c_int;
    fn jax_generate_embedding_ffi(
        text_ptr: *const std::os::raw::c_char,
        text_len: usize,
        result: *mut f32,
        result_len: usize
    ) -> std::ffi::c_int;
    #[allow(dead_code)] fn jax_cosine_similarity_ffi(
        vec1: *const f32,
        vec2: *const f32,
        dim: usize
    ) -> f32;
    fn jax_predict_next_moves_ffi(
        context_vec: *const f32,
        dim: usize,
        n_moves: usize,
        result: *mut f32
    ) -> std::ffi::c_int;
}

lazy_static! {
    static ref EMBEDDING_CACHE: DashMap<String, Vec<f32>> = DashMap::new();
}

pub enum EmbeddingModel { MiniLML6, BGEBase, BGELarge }
impl EmbeddingModel {
    pub fn dimension(&self) -> usize {
        match self {
            EmbeddingModel::MiniLML6 => 384,
            EmbeddingModel::BGEBase => 768,
            EmbeddingModel::BGELarge => 1024,
        }
    }
    pub fn model_name(&self) -> &'static str {
        match self {
            EmbeddingModel::MiniLML6 => "all-MiniLM-L6-v2",
            EmbeddingModel::BGEBase => "bge-base-en-v1.5",
            EmbeddingModel::BGELarge => "bge-large-en-v1.5",
        }
    }
}

pub struct EmbeddingConfig { pub model: EmbeddingModel, pub use_cache: bool }
impl Default for EmbeddingConfig {
    fn default() -> Self { Self { model: EmbeddingModel::MiniLML6, use_cache: true } }
}

pub struct EmbeddingGenerator { config: EmbeddingConfig }

impl EmbeddingGenerator {
    pub fn new(config: EmbeddingConfig) -> Self { Self { config } }

    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        if self.config.use_cache {
            let key = self.cache_key(text);
            if let Some(emb) = EMBEDDING_CACHE.get(&key) { return Ok(emb.clone()); }
        }

        #[cfg(feature = "ffi-jax")]
        {
            let dim = self.config.model.dimension();
            let mut result = vec![0.0f32; dim];
            let text_c = std::ffi::CString::new(text).unwrap();

            unsafe {
                let ret = jax_generate_embedding_ffi(text_c.as_ptr(), text.len(), result.as_mut_ptr(), dim);
                if ret == 0 {
                    if self.config.use_cache { self.save_to_cache(&self.cache_key(text), &result); }
                    Ok(result)
                } else {
                    Err(FfiError::CallFailed("JAX generate_embedding failed".into()))
                }
            }
        }
        #[cfg(not(feature = "ffi-jax"))]
        {
            let dim = self.config.model.dimension();
            Ok(vec![0.0f32; dim]) // Stub
        }
    }

    fn cache_key(&self, text: &str) -> String { format!("{}:{}", self.config.model.model_name(), text) }
    fn save_to_cache(&self, key: &str, embedding: &[f32]) { EMBEDDING_CACHE.insert(key.to_string(), embedding.to_vec()); }
}

pub fn init() -> Result<()> {
    #[cfg(feature = "ffi-jax")]
    unsafe { if jax_init_ffi() == 0 { Ok(()) } else { Err(FfiError::CallFailed("JAX init failed".into())) } }
    #[cfg(not(feature = "ffi-jax"))]
    Ok(())
}

pub fn shutdown() {
    #[cfg(feature = "ffi-jax")]
    unsafe { jax_shutdown_ffi(); }
}

pub fn predict_next_moves(current_context: &[f32], n_moves: usize) -> Result<Vec<Vec<f32>>> {
    #[cfg(feature = "ffi-jax")]
    {
        let dim = current_context.len();
        let mut flat_results = vec![0.0f32; dim * n_moves];
        unsafe {
            let ret = jax_predict_next_moves_ffi(current_context.as_ptr(), dim, n_moves, flat_results.as_mut_ptr());
            if ret == 0 {
                Ok(flat_results.chunks(dim).map(|c| c.to_vec()).collect())
            } else {
                Err(FfiError::CallFailed("JAX predict_next_moves failed".into()))
            }
        }
    }
    #[cfg(not(feature = "ffi-jax"))]
    {
        Ok(vec![current_context.to_vec(); n_moves])
    }
}
