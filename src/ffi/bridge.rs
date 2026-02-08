//! ffi/bridge.rs - Zig FFI Bridge Integration
use super::error::{FfiError, Result};
use smallvec::SmallVec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language { Zig, Julia, Python, Mojo, Pony }

#[repr(C)]
#[derive(Debug)]
pub struct BufferInfo {
    pub capacity: usize,
    pub used: usize,
    pub available: usize,
    pub ref_count: u32,
    pub initialized: bool,
}

pub fn init() -> bool { true }
pub fn shutdown() {}
pub fn get_metrics() -> (u64, f64) { (0, 0.0) }
pub fn reset_metrics() {}
pub fn dispatch_fast(_lang: Language, _op: &str, data: &mut [f64]) -> Result<bool> {
    let mut _v: SmallVec<[f64; 16]> = SmallVec::from_slice(data);
    Ok(true)
}
