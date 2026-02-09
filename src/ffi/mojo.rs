//! ffi/mojo.rs - Mojo SIMD Kernels
use super::error::Result;
use std::os::raw::{c_int, c_double};

extern "C" {
    fn mojo_init_kernels() -> c_int;
    fn mojo_dot_product_simd(a: *const c_double, b: *const c_double, len: usize) -> c_double;
}

pub fn init() -> Result<()> {
    unsafe {
        mojo_init_kernels();
    }
    Ok(())
}

pub fn shutdown() {}

pub fn dot_product(a: &[f64], b: &[f64]) -> Result<f64> {
    if a.len() != b.len() {
        return Err(crate::error::MemoryPError::Other("Dimensiones no coinciden para dot product".into()));
    }
    unsafe {
        let res = mojo_dot_product_simd(a.as_ptr(), b.as_ptr(), a.len());
        Ok(res)
    }
}
