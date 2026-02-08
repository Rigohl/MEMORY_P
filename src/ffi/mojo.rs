//! ffi/mojo.rs - Mojo SIMD Kernels
use super::error::Result;

pub fn init() -> Result<()> { Ok(()) }
pub fn shutdown() {}
pub fn dot_product(_a: &[f64], _b: &[f64]) -> Result<f64> { Ok(0.0) }
