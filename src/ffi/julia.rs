//! ffi/julia.rs - Julia Mathematical Core Integration
//! REAL FFI Implementation connecting to Julia shared library

use super::error::{FfiError, Result};
use std::os::raw::{c_int, c_double};

// Julia FFI functions
#[cfg(feature = "ffi-julia")]
#[link(name = "julia_ffi", kind = "dylib")]
extern "C" {
    fn julia_init() -> c_int;
    fn julia_shutdown() -> c_int;
    fn julia_optimize_weights_ffi(
        data: *const c_double,
        len: c_int,
        result: *mut c_double,
    ) -> c_int;
    fn julia_chaos_analysis_ffi(
        data: *const c_double,
        len: c_int,
    ) -> c_double;
    fn julia_get_decision_ffi(
        entropy_val: f64,
        chaos_val: f64,
        stability_val: f64,
        result_buf: *mut u8,
        buf_len: c_int
    ) -> c_int;
}

/// Inicializa el runtime de Julia
pub fn init() -> Result<()> {
    #[cfg(feature = "ffi-julia")]
    {
        tracing::info!("🧮 Inicializando Julia mathematical core");
        unsafe {
            let ret = julia_init();
            if ret == 0 {
                tracing::info!("✅ Julia runtime initialized");
                Ok(())
            } else {
                Err(FfiError::JuliaException(
                    "Failed to initialize Julia runtime".to_string(),
                ))
            }
        }
    }
    #[cfg(not(feature = "ffi-julia"))]
    {
        tracing::warn!("⚠️ Julia no disponible (feature 'ffi-julia' deshabilitado)");
        Err(FfiError::NotAvailable("Julia".to_string()))
    }
}

/// Finaliza el runtime de Julia
pub fn shutdown() {
    #[cfg(feature = "ffi-julia")]
    unsafe { julia_shutdown(); }
}

/// Optimiza pesos usando Julia
pub fn optimize_weights(weights: &[f64]) -> Result<Vec<f64>> {
    #[cfg(feature = "ffi-julia")]
    {
        let mut result = vec![0.0; weights.len()];
        unsafe {
            let ret = julia_optimize_weights_ffi(
                weights.as_ptr(),
                weights.len() as c_int,
                result.as_mut_ptr(),
            );
            if ret == 0 {
                Ok(result)
            } else {
                Err(FfiError::JuliaException("Julia optimize_weights_ffi failed".into()))
            }
        }
    }
    #[cfg(not(feature = "ffi-julia"))]
    {
        let sum: f64 = weights.iter().sum();
        if sum > 0.0 {
            Ok(weights.iter().map(|w| w / sum).collect())
        } else {
            Err(FfiError::NotAvailable("Julia".into()))
        }
    }
}

/// Analiza caos usando Julia
pub fn chaos_analysis(data: &[f64]) -> Result<f64> {
    #[cfg(feature = "ffi-julia")]
    {
        unsafe {
            let lyapunov = julia_chaos_analysis_ffi(data.as_ptr(), data.len() as c_int);
            if lyapunov.is_nan() {
                Err(FfiError::JuliaException("Julia chaos_analysis_ffi failed".into()))
            } else {
                Ok(lyapunov)
            }
        }
    }
    #[cfg(not(feature = "ffi-julia"))]
    {
        Ok(0.1) // Constant fallback
    }
}

/// Obtiene decisión de estrategia de búsqueda
pub fn get_search_decision(entropy: f64, chaos: f64, stability: f64) -> Result<String> {
    #[cfg(feature = "ffi-julia")]
    {
        let mut buf = [0u8; 64];
        unsafe {
            let len = julia_get_decision_ffi(entropy, chaos, stability, buf.as_mut_ptr(), buf.len() as c_int);
            if len >= 0 {
                let s = std::str::from_utf8(&buf[..len as usize]).unwrap_or("UNKNOWN");
                Ok(s.to_string())
            } else {
                Err(FfiError::JuliaException("Julia get_decision failed".into()))
            }
        }
    }
    #[cfg(not(feature = "ffi-julia"))]
    {
        Ok("VECTOR".into())
    }
}
