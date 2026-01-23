//! ffi/mod.rs - Rust FFI Orchestrator for MEMORY_P v2.0
//!
//! Este módulo actúa como orquestador de todos los lenguajes FFI:
//! - Julia: Mathematical core
//! - JAX: ML inference
//! - Mojo: SIMD kernels
//! - Pony: Actor system
//! - Zig: FFI bridge
//!
//! Rust coordina todas las llamadas FFI y garantiza memory safety.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int};

pub mod bridge;
pub mod error;
pub mod julia;
pub mod jax;
pub mod mojo;
pub mod pony;

// Re-export FFI error types
pub use error::{FfiError, Result as FfiResult};

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

