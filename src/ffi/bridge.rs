//! ffi/bridge.rs - Unified Multi-Language FFI Bridge
//! REAL FFI Implementation connecting to Zig bridge

use super::error::{Result, FfiError};
use std::ffi::CString;
use std::os::raw::{c_void, c_char};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language { Zig = 0, Julia = 1, Jax = 2, Mojo = 3, Pony = 4 }

#[repr(C)]
pub struct FfiVec {
    pub data: *mut f64,
    pub len: usize,
    pub cap: usize,
}

#[repr(C)]
pub struct FfiResult {
    pub success: bool,
    pub data: FfiVec,
    pub error_msg: *const c_char,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BufferInfo {
    pub capacity: usize,
    pub used: usize,
    pub available: usize,
    pub ref_count: u32,
    pub initialized: bool,
}

#[cfg(feature = "ffi-zig")]
#[link(name = "zig_bridge", kind = "dylib")]
extern "C" {
    fn ffi_init() -> bool;
    fn ffi_shutdown();
    fn ffi_dispatch(lang: Language, operation: *const c_char, input: FfiVec) -> FfiResult;
    fn ffi_free_result(result: *mut FfiResult);

    // Buffer functions from shared_memory_buffer.zig
    fn shared_memory_buffer_new(capacity: usize) -> *mut c_void;
    fn shared_memory_buffer_write(buffer: *mut c_void, data: *const u8, len: usize) -> isize;
    fn shared_memory_buffer_read(buffer: *const c_void, offset: usize, dest: *mut u8, len: usize) -> isize;
    fn shared_memory_buffer_free(buffer: *mut c_void);
    fn shared_memory_buffer_info(buffer: *const c_void) -> BufferInfo;
}

pub fn init() -> bool {
    #[cfg(feature = "ffi-zig")]
    unsafe { ffi_init() }
    #[cfg(not(feature = "ffi-zig"))]
    true
}

pub fn shutdown() {
    #[cfg(feature = "ffi-zig")]
    unsafe { ffi_shutdown() }
}

pub fn dispatch_fast(lang: Language, op: &str, data: &mut [f64]) -> Result<bool> {
    #[cfg(feature = "ffi-zig")]
    {
        let op_c = CString::new(op).unwrap();
        let input = FfiVec {
            data: data.as_mut_ptr(),
            len: data.len(),
            cap: data.len(),
        };

        unsafe {
            let mut res = ffi_dispatch(lang, op_c.as_ptr(), input);
            let success = res.success;

            if success && !res.data.data.is_null() && res.data.len == data.len() {
                std::ptr::copy_nonoverlapping(res.data.data, data.as_mut_ptr(), data.len());
            }

            ffi_free_result(&mut res);
            Ok(success)
        }
    }
    #[cfg(not(feature = "ffi-zig"))]
    {
        let _ = (lang, op, data);
        Ok(true)
    }
}

// Wrapper functions for shared_memory/buffer.rs
pub fn create_shared_buffer(capacity: usize) -> Option<*mut c_void> {
    #[cfg(feature = "ffi-zig")]
    unsafe {
        let ptr = shared_memory_buffer_new(capacity);
        if ptr.is_null() { None } else { Some(ptr) }
    }
    #[cfg(not(feature = "ffi-zig"))]
    { let _ = capacity; None }
}

pub fn write_to_buffer(buffer: *mut c_void, data: &[u8]) -> Result<usize> {
    #[cfg(feature = "ffi-zig")]
    unsafe {
        let res = shared_memory_buffer_write(buffer, data.as_ptr(), data.len());
        if res < 0 {
            Err(FfiError::CallFailed(format!("Zig write error: {}", res)))
        } else {
            Ok(res as usize)
        }
    }
    #[cfg(not(feature = "ffi-zig"))]
    { let _ = (buffer, data); Ok(0) }
}

pub fn read_from_buffer(buffer: *mut c_void, offset: usize, len: usize) -> Result<Vec<u8>> {
    #[cfg(feature = "ffi-zig")]
    unsafe {
        let mut dest = vec![0u8; len];
        let res = shared_memory_buffer_read(buffer, offset, dest.as_mut_ptr(), len);
        if res < 0 {
            Err(FfiError::CallFailed(format!("Zig read error: {}", res)))
        } else {
            Ok(dest)
        }
    }
    #[cfg(not(feature = "ffi-zig"))]
    { let _ = (buffer, offset, len); Ok(vec![]) }
}

pub fn get_buffer_info(buffer: *mut c_void) -> BufferInfo {
    #[cfg(feature = "ffi-zig")]
    unsafe { shared_memory_buffer_info(buffer) }
    #[cfg(not(feature = "ffi-zig"))]
    {
        let _ = buffer;
        BufferInfo { capacity: 0, used: 0, available: 0, ref_count: 0, initialized: false }
    }
}

pub fn free_shared_buffer(buffer: *mut c_void) {
    #[cfg(feature = "ffi-zig")]
    unsafe { shared_memory_buffer_free(buffer); }
    #[cfg(not(feature = "ffi-zig"))]
    { let _ = buffer; }
}

pub fn get_metrics() -> (u64, f64) { (0, 0.0) }
pub fn reset_metrics() {}
