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
