//! ffi/julia.rs - Julia Mathematical Core
use super::error::Result;
use std::os::raw::{c_int, c_double};

extern "C" {
    fn julia_init() -> c_int;
    fn julia_shutdown() -> c_int;
    fn julia_chaos_analysis_ffi(data: *const c_double, len: c_int) -> c_double;
}

pub fn init() -> Result<()> {
    unsafe {
        if julia_init() != 0 {
            return Err(crate::error::MemoryPError::Other("Falló inicialización de Julia".into()));
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
            return Err(crate::error::MemoryPError::Other("Error en análisis de caos de Julia".into()));
        }
        Ok(res)
    }
}

/// Obtiene una decisión de estrategia de búsqueda basada en matemáticas de caos y entropía
pub fn get_search_decision(entropy: f64, chaos: f64, stability: f64) -> Result<String> {
    #[cfg(feature = "ffi-julia")]
    {
        let mut buf = [0u8; 64];
        unsafe {
            // Llamada simulada a julia_get_decision_ffi
            // En producción: ffi_bridge.call("julia_get_decision_ffi", ...)

            // Lógica de decisión proactiva (espejo de lo que hace Julia)
            if entropy > 2.5 {
                Ok("HYBRID_FUSION".to_string())
            } else if chaos > 0.4 {
                Ok("VECTOR_QDRANT".to_string())
            } else if stability > 0.8 {
                Ok("TEXT_TANTIVY".to_string())
            } else {
                Ok("HYBRID_BALANCED".to_string())
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
