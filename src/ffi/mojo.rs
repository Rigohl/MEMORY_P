//! ffi/mojo.rs - Mojo SIMD Kernels Integration
//! REAL FFI Implementation connecting to Mojo shared library

use super::error::{FfiError, Result};

#[cfg(feature = "ffi-mojo")]
#[link(name = "mojo_kernels", kind = "dylib")]
extern "C" {
    fn mojo_dot_product(a: *const f64, b: *const f64, n: usize) -> f64;
    fn mojo_cosine_similarity(a: *const f64, b: *const f64, n: usize) -> f64;
    fn mojo_cosine_similarity_batch(query: *const f64, corpus: *const f64, n_docs: usize, dim: usize, results: *mut f64);
}

pub fn init() -> Result<()> {
    #[cfg(feature = "ffi-mojo")]
    {
        tracing::info!("⚡ Inicializando Mojo SIMD kernels");
        Ok(())
    }
    #[cfg(not(feature = "ffi-mojo"))]
    {
        Err(FfiError::NotAvailable("Mojo".into()))
    }
}

pub fn shutdown() {}

pub fn dot_product(a: &[f64], b: &[f64]) -> Result<f64> {
    #[cfg(feature = "ffi-mojo")]
    {
        if a.len() != b.len() {
            return Err(FfiError::CallFailed("Dimension mismatch".into()));
        }
        unsafe { Ok(mojo_dot_product(a.as_ptr(), b.as_ptr(), a.len())) }
    }
    #[cfg(not(feature = "ffi-mojo"))]
    {
        Ok(a.iter().zip(b.iter()).map(|(x, y)| x * y).sum())
    }
}

pub fn cosine_similarity(a: &[f64], b: &[f64]) -> Result<f64> {
    #[cfg(feature = "ffi-mojo")]
    {
        if a.len() != b.len() {
            return Err(FfiError::CallFailed("Dimension mismatch".into()));
        }
        unsafe { Ok(mojo_cosine_similarity(a.as_ptr(), b.as_ptr(), a.len())) }
    }
    #[cfg(not(feature = "ffi-mojo"))]
    {
        let dot = dot_product(a, b)?;
        let norm_a = a.iter().map(|x| x*x).sum::<f64>().sqrt();
        let norm_b = b.iter().map(|x| x*x).sum::<f64>().sqrt();
        if norm_a < 1e-8 || norm_b < 1e-8 { return Ok(0.0); }
        Ok(dot / (norm_a * norm_b))
    }
}

pub fn cosine_similarity_batch(query: &[f64], corpus: &[Vec<f64>]) -> Result<Vec<f64>> {
    #[cfg(feature = "ffi-mojo")]
    {
        let n_docs = corpus.len();
        let dim = query.len();
        let mut results = vec![0.0; n_docs];
        // Corpus must be flattened for FFI
        let flattened_corpus: Vec<f64> = corpus.iter().flatten().cloned().collect();
        unsafe {
            mojo_cosine_similarity_batch(query.as_ptr(), flattened_corpus.as_ptr(), n_docs, dim, results.as_mut_ptr());
        }
        Ok(results)
    }
    #[cfg(not(feature = "ffi-mojo"))]
    {
        corpus.iter().map(|doc| cosine_similarity(query, doc)).collect()
    }
}
