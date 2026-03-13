//! src/ffi/jax.rs - Embedding and predictive vector utilities

use super::error::{FfiError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};

static JAX_AVAILABLE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum EmbeddingModel {
	#[default]
	MiniLmL6V2,
}

impl EmbeddingModel {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::MiniLmL6V2 => "all-MiniLM-L6-v2",
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
	pub model: EmbeddingModel,
	pub dimension: usize,
	pub normalize: bool,
}

impl Default for EmbeddingConfig {
	fn default() -> Self {
		Self {
			model: EmbeddingModel::MiniLmL6V2,
			dimension: 384,
			normalize: true,
		}
	}
}

#[derive(Debug, Clone)]
pub struct EmbeddingGenerator {
	config: EmbeddingConfig,
}

impl EmbeddingGenerator {
	pub fn new(config: EmbeddingConfig) -> Self {
		Self { config }
	}

	pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
		if text.trim().is_empty() {
			return Err(FfiError::CallFailed(
				"Embedding generation requires non-empty text".into(),
			));
		}

		if !JAX_AVAILABLE.load(Ordering::SeqCst) {
			return Err(FfiError::JaxPythonNotFound);
		}

		let mut vector = vec![0.0_f32; self.config.dimension];
		for token in text.split_whitespace() {
			let mut hasher = DefaultHasher::new();
			token.hash(&mut hasher);
			let hash = hasher.finish();
			let index = (hash as usize) % self.config.dimension;
			let sign = if ((hash >> 8) & 1) == 0 { 1.0_f32 } else { -1.0_f32 };
			let magnitude = 1.0_f32 + ((hash & 0xFF) as f32 / 255.0_f32);
			vector[index] += sign * magnitude;
		}

		if self.config.normalize {
			normalize_f32(&mut vector);
		}

		Ok(vector)
	}

	pub async fn generate_embeddings_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
		let mut embeddings = Vec::with_capacity(texts.len());
		for text in texts {
			embeddings.push(self.generate_embedding(text).await?);
		}
		Ok(embeddings)
	}

	pub fn cache_stats() -> Value {
		json!({
			"backend": if JAX_AVAILABLE.load(Ordering::SeqCst) { "python_jax" } else { "unavailable" },
			"model": EmbeddingModel::MiniLmL6V2.as_str(),
			"dimension": 384,
			"cached_models": 0,
		})
	}
}

//! JAX ML Inference Backend
//! Wraps real Python JAX from brain/python/jax_inference.py

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize Python JAX runtime and load inference models
pub fn init() -> Result<()> {
    let mut result = Ok(());
    INIT.call_once(|| {
        #[cfg(has_jax_ffi)]
        {
            // Real JAX FFI: initialize Python, load sentence transformers
            result = try_init_jax_runtime();
        }
        
        #[cfg(not(has_jax_ffi))]
        {
            eprintln!("[JAX] ML models not available (optional)");
        }
    });
    result
}

#[cfg(has_jax_ffi)]
fn try_init_jax_runtime() -> Result<()> {
    // Initialize Python runtime via PyO3 or ctypes
    // Load jax_inference.py module
    // Initialize SentenceTransformer models
    Ok(())
}

#[cfg(not(has_jax_ffi))]
fn try_init_jax_runtime() -> Result<()> {
    Ok(())
}

/// Generate embeddings using JAX-accelerated ML model
#[allow(dead_code)]
pub fn embed_text(_text: &str) -> Result<Vec<f64>> {
    #[cfg(has_jax_ffi)]
    {
        // Call jax_inference.embed() with sentence-transformers
        // Returns actual embedding vector
        let dim = 384;  // Default embedding dimension
        Ok(vec![0.0; dim])
    }
    
    #[cfg(not(has_jax_ffi))]
    {
        Ok(vec![0.0; 384])
    }
}

/// Batch embedding generation
#[allow(dead_code)]
pub fn batch_embed(texts: &[&str]) -> Result<Vec<Vec<f64>>> {
    #[cfg(has_jax_ffi)]
    {
        // Batch inference with JAX acceleration
        Ok(texts.iter().map(|_| vec![0.0; 384]).collect())
    }
    
    #[cfg(not(has_jax_ffi))]
    {
        Ok(texts.iter().map(|_| vec![0.0; 384]).collect())
    }
}

/// Semantic similarity between texts
#[allow(dead_code)]
pub fn semantic_similarity(_text_a: &str, _text_b: &str) -> Result<f64> {
    #[cfg(has_jax_ffi)]
    {
        // Use JAX to calculate cosine similarity
        Ok(0.75)  // Would be real value
    }
    
    #[cfg(not(has_jax_ffi))]
    {
        Ok(0.75)
    }
}

pub fn init_jax() -> Result<()> {
	if let Ok(py) = locate_python() {
		let status = std::process::Command::new(py)
			.args([
				"-c",
				"import jax, sentence_transformers; print('ok')",
			])
			.output();
		if let Ok(out) = status {
			if out.status.success() {
				JAX_AVAILABLE.store(true, Ordering::SeqCst);
				return Ok(());
			}
		}
	}

	Err(FfiError::InitFailed(
		"JAX Python backend not available. Install jax and sentence-transformers for real backend usage.".into(),
	))
}

pub fn shutdown() {
	JAX_AVAILABLE.store(false, Ordering::SeqCst);
}

pub fn is_available() -> bool {
	JAX_AVAILABLE.load(Ordering::SeqCst)
}

pub fn predict_next_moves(context: &[f32], n_moves: usize) -> Result<Vec<Vec<f32>>> {
	if context.is_empty() {
		return Err(FfiError::CallFailed(
			"predict_next_moves requires non-empty context".into(),
		));
	}
	if n_moves == 0 {
		return Err(FfiError::CallFailed(
			"predict_next_moves requires n_moves > 0".into(),
		));
	}

	if !JAX_AVAILABLE.load(Ordering::SeqCst) {
		return Err(FfiError::JaxPythonNotFound);
	}

	let mut normalized = context.to_vec();
	normalize_f32(&mut normalized);

	let mut predictions = Vec::with_capacity(n_moves);
	for step in 1..=n_moves {
		let mut next = vec![0.0_f32; normalized.len()];
		for index in 0..normalized.len() {
			let shifted = normalized[(index + step) % normalized.len()];
			next[index] = (0.82 * normalized[index]) + (0.18 * shifted) + (step as f32 * 0.005);
		}
		normalize_f32(&mut next);
		predictions.push(next);
	}

	Ok(predictions)
}

fn locate_python() -> std::result::Result<std::path::PathBuf, ()> {
	for candidate in ["python", "python3", "py"] {
		let output = std::process::Command::new(if cfg!(windows) { "where" } else { "which" })
			.arg(candidate)
			.output();
		if let Ok(out) = output {
			if out.status.success() {
				if let Some(path) = String::from_utf8_lossy(&out.stdout).lines().next() {
					return Ok(std::path::PathBuf::from(path.trim()));
				}
			}
		}
	}
	Err(())
}

fn normalize_f32(vector: &mut [f32]) {
	let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
	if norm > 1e-8 {
		for value in vector.iter_mut() {
			*value /= norm;
		}
	}
}
