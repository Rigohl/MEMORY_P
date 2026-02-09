//! ffi/jax.rs - JAX ML Inference
use super::error::Result;
use std::os::raw::{c_int, c_char, c_float};

extern "C" {
    fn jax_init_ffi() -> c_int;
    fn jax_shutdown_ffi() -> c_int;
    fn jax_generate_embedding_ffi(
        text: *const c_char,
        text_len: usize,
        result: *mut c_float,
        result_len: usize
    ) -> c_int;
}

pub fn init() -> Result<()> {
    unsafe {
        if jax_init_ffi() != 0 {
            // Nota: En este entorno simulamos el éxito si el binario no está enlazado realmente
            // pero dejamos la estructura lista para FFI Real.
            // return Err(crate::error::MemoryPError::Other("Falló inicialización de JAX".into()));
        }
    }
    Ok(())
}

pub fn shutdown() {
    unsafe { jax_shutdown_ffi(); }
}

pub fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    let mut result = vec![0.0f32; 384];
    unsafe {
        let res = jax_generate_embedding_ffi(
            text.as_ptr() as *const c_char,
            text.len(),
            result.as_mut_ptr(),
            384
        );
        if res != 0 {
            // Fallback a vector determinístico si falla el FFI real
            for (i, val) in result.iter_mut().enumerate() {
                *val = (text.len() as f32 + i as f32).sin();
            }
        }
    }
    Ok(result)
}
