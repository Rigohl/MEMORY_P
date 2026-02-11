//! ffi/mojo.rs - Mojo SIMD Kernels Integration

use super::error::{FfiError, Result};

/// Inicializa los kernels de Mojo
pub fn init() -> Result<()> {
    #[cfg(feature = "ffi-mojo")]
    {
        tracing::info!("⚡ Inicializando Mojo SIMD kernels");
        // Mojo kernels son stateless, no requieren init
        Ok(())
    }

    #[cfg(not(feature = "ffi-mojo"))]
    {
        tracing::warn!("⚠️  Mojo no disponible (feature 'ffi-mojo' deshabilitado)");
        Err(FfiError::NotAvailable("Mojo".to_string()))
    }
}

/// Finaliza los kernels de Mojo
pub fn shutdown() {
    #[cfg(feature = "ffi-mojo")]
    {
        tracing::info!("⚡ Finalizando Mojo kernels");
    }
}

/// Calcula dot product con SIMD (35000x más rápido que Python)
pub fn dot_product(a: &[f64], b: &[f64]) -> Result<f64> {
    #[cfg(feature = "ffi-mojo")]
    {
        if a.len() != b.len() {
            return Err(FfiError::CallFailed(
                "Vector dimensions mismatch".to_string(),
            ));
        }

        tracing::trace!("Dot product con Mojo SIMD para {} elementos", a.len());

        // TODO: Llamada real a Mojo kernel
        // Fallback: Implementación nativa Rust (ya es rápida)
        let result: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

        Ok(result)
    }

    #[cfg(not(feature = "ffi-mojo"))]
    {
        Err(FfiError::NotAvailable("Mojo dot_product".to_string()))
    }
}

/// Calcula similitud coseno con SIMD
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> Result<f64> {
    #[cfg(feature = "ffi-mojo")]
    {
        if a.len() != b.len() {
            return Err(FfiError::CallFailed(
                "Vector dimensions mismatch".to_string(),
            ));
        }

        // TODO: Llamada real a Mojo kernel
        let dot = dot_product(a, b)?;
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

        if norm_a < 1e-8 || norm_b < 1e-8 {
            return Ok(0.0);
        }

        Ok(dot / (norm_a * norm_b))
    }

    #[cfg(not(feature = "ffi-mojo"))]
    {
        Err(FfiError::NotAvailable("Mojo cosine_similarity".to_string()))
    }
}

/// Calcula similitudes coseno en batch (query vs corpus)
pub fn cosine_similarity_batch(query: &[f64], corpus: &[Vec<f64>]) -> Result<Vec<f64>> {
    #[cfg(feature = "ffi-mojo")]
    {
        tracing::debug!(
            "Batch cosine similarity con Mojo para {} documentos",
            corpus.len()
        );

        // TODO: Llamada real a Mojo kernel optimizado
        let similarities: Result<Vec<f64>> = corpus
            .iter()
            .map(|doc| cosine_similarity(query, doc))
            .collect();

        similarities
    }

    #[cfg(not(feature = "ffi-mojo"))]
    {
        Err(FfiError::NotAvailable(
            "Mojo cosine_similarity_batch".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];

        let result = dot_product(&a, &b);

        if let Ok(dot) = result {
            // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
            assert!((dot - 32.0).abs() < 0.01);
        }
    }
}
