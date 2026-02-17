//! ffi/julia.rs - Julia Mathematical Core
use super::error::{Result, FfiError};
use std::os::raw::{c_int, c_double, c_char};
use std::ffi::CStr;

#[link(name = "julia_ffi")]
extern "C" {
    fn julia_init() -> c_int;
    fn julia_shutdown() -> c_int;
    fn julia_chaos_analysis_ffi(data: *const c_double, len: c_int) -> c_double;
    fn julia_get_decision_ffi(entropy: c_double, chaos: c_double, stability: c_double, buffer: *mut c_char, buffer_len: usize) -> c_int;
}

pub fn init() -> Result<()> {
    unsafe {
        if julia_init() != 0 {
            return Err(FfiError::JuliaException("Falló inicialización de Julia".into()));
        }
    }
    Ok(())
}

pub fn shutdown() {
    unsafe { julia_shutdown(); }
}

pub fn chaos_analysis(data: &[f64]) -> Result<f64> {
    unsafe {
        let res = julia_chaos_analysis_ffi(data.as_ptr(), data.len() as c_int);
        if res.is_nan() {
            return Err(FfiError::JuliaException("Error en análisis de caos de Julia".into()));
        }
        Ok(res)
    }
}

/// Obtiene una decisión de estrategia de búsqueda basada en matemáticas de caos y entropía
pub fn get_search_decision(entropy: f64, chaos: f64, stability: f64) -> crate::error::Result<String> {
    // Note: Use crate::error::Result here because this is a high-level API function,
    // unlike the others which are low-level wrappers returning FfiError.
    // However, I need to map FfiError to MemoryPError if I use FFI calls.

    #[cfg(feature = "ffi-julia")]
    {
        let mut buf = [0u8; 64];
        unsafe {
            // Real FFI call
            let ret = julia_get_decision_ffi(
                entropy,
                chaos,
                stability,
                buf.as_mut_ptr() as *mut c_char,
                buf.len()
            );

            if ret == 0 {
                let c_str = CStr::from_ptr(buf.as_ptr() as *const c_char);
                Ok(c_str.to_string_lossy().into_owned())
            } else {
                Err(crate::error::MemoryPError::Ffi(FfiError::JuliaException("Julia FFI decision failed".into())))
            }
        }
    }

    #[cfg(not(feature = "ffi-julia"))]
    {
        // Fallback simple
        if entropy > 2.0 {
            Ok("HYBRID".to_string())
        } else {
            Ok("VECTOR".to_string())
        }
    }
}
