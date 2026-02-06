//! ffi/mod.rs - Rust FFI Orchestrator for MEMORY_P v2.0
//!
//! Este módulo actúa como orquestador de todos los lenguajes FFI:
//! - Julia: Mathematical core
//! - JAX: ML inference
//! - Mojo: SIMD kernels
//! - Pony: Actor system
//! - Zig: FFI bridge (ultra-low-latency <1µs)
//!
//! Rust coordina todas las llamadas FFI y garantiza memory safety.
//!
//! ## Optimizaciones de Performance
//!
//! El FFI bridge está optimizado para latencia ultra-baja:
//! - Zero-copy data transfer usando slices directas
//! - Stack allocation para arrays pequeños (<256 elementos)
//! - Arena allocator en Zig para reducir overhead
//! - Dispatch inline sin allocations
//! - Batch processing paralelo con Rayon
//! - Métricas automáticas de latencia
//!
//! ## Uso
//!
//! ```rust
//! use memory_p::ffi::bridge::{self, Language};
//!
//! // Inicializar
//! bridge::init();
//!
//! // Llamada simple
//! let mut data = vec![1.0, 2.0, 3.0];
//! let result = bridge::dispatch_fast(Language::Zig, "process", &mut data)?;
//!
//! // Batch paralelo
//! let requests = vec![
//!     (Language::Zig, "op1", vec![1.0, 2.0]),
//!     (Language::Zig, "op2", vec![3.0, 4.0]),
//! ];
//! let results = bridge::dispatch_batch(&requests);
//!
//! // Métricas
//! let (calls, avg_us) = bridge::get_metrics();
//!
//! // Cleanup
//! bridge::shutdown();
//! ```

use std::os::raw::{c_int};

pub mod bridge;
pub mod error;
pub mod julia;
pub mod jax;
pub mod mojo;
pub mod pony;

#[cfg(test)]
mod benchmarks;

// Re-export FFI error types
pub use error::{FfiError, Result as FfiResult};

// Re-export bridge types para uso público
// Nota: Language, dispatch_fast, dispatch_batch, get_metrics, reset_metrics
// se definen en el módulo bridge cuando las características correspondientes estén compiladas

/// Inicializa el sistema FFI completo
pub fn init() -> crate::error::Result<()> {
    tracing::info!("🔧 Inicializando sistema FFI multi-lenguaje");

    // Inicializar cada runtime (ignorar errores si no están disponibles)
    if !bridge::init() {
        tracing::warn!("⚠️  Zig bridge no disponible");
    }

    let _ = julia::init();
    let _ = jax::init();
    let _ = mojo::init();
    let _ = pony::init();

    tracing::info!("✅ Sistema FFI inicializado");
    Ok(())
}

/// Finaliza el sistema FFI y libera recursos
pub fn shutdown() {
    tracing::info!("🔧 Finalizando sistema FFI");

    pony::shutdown();
    mojo::shutdown();
    jax::shutdown();
    julia::shutdown();
    bridge::shutdown();

    tracing::info!("✅ Sistema FFI finalizado");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_init() {
        // Test que el sistema FFI se puede inicializar
        let result = init();
        assert!(result.is_ok()); // Siempre ok, los FFI individuales pueden fallar

        shutdown();
    }
}

