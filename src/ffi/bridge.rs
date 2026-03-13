//! src/ffi/bridge.rs - Core Zig bridge wrappers

use super::error::{FfiError, Result};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

static BRIDGE_AVAILABLE: AtomicBool = AtomicBool::new(false);
static TOTAL_CALLS: AtomicU64 = AtomicU64::new(0);
static TOTAL_LATENCY_NS: AtomicU64 = AtomicU64::new(0);

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Language {
    Julia = 0,
    Jax = 1,
    Mojo = 2,
    Pony = 3,
    Zig = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiVec {
    pub data: *mut f64,
    pub len: usize,
    pub cap: usize,
}

impl FfiVec {
    pub fn from_mut_slice(data: &mut [f64]) -> Self {
        Self {
            data: data.as_mut_ptr(),
            len: data.len(),
            cap: data.len(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiResult {
    pub success: bool,
    pub data: FfiVec,
    pub error_msg: *const c_char,
}

#[cfg(has_zig_ffi)]
#[link(name = "ffi_bridge")]
extern "C" {
    fn ffi_dispatch(lang: Language, operation: *const c_char, input: FfiVec) -> FfiResult;
    fn ffi_free_result(result: *mut FfiResult);
}

pub fn init() -> bool {
    #[cfg(has_zig_ffi)]
    {
        BRIDGE_AVAILABLE.store(true, Ordering::SeqCst);
        return true;
    }

    #[cfg(not(has_zig_ffi))]
    {
        BRIDGE_AVAILABLE.store(false, Ordering::SeqCst);
        false
    }
}

pub fn shutdown() {
    BRIDGE_AVAILABLE.store(false, Ordering::SeqCst);
}

pub fn reset_metrics() {
    TOTAL_CALLS.store(0, Ordering::SeqCst);
    TOTAL_LATENCY_NS.store(0, Ordering::SeqCst);
}

pub fn get_metrics() -> (u64, f64) {
    let total_calls = TOTAL_CALLS.load(Ordering::SeqCst);
    let total_latency_ns = TOTAL_LATENCY_NS.load(Ordering::SeqCst);
    let avg_us = if total_calls == 0 {
        0.0
    } else {
        (total_latency_ns as f64 / total_calls as f64) / 1000.0
    };
    (total_calls, avg_us)
}

pub fn dispatch_fast(lang: Language, operation: &str, data: &mut [f64]) -> Result<bool> {
    let started = Instant::now();

    if data.is_empty() {
        return Ok(true);
    }

    #[cfg(has_zig_ffi)]
    {
        if !BRIDGE_AVAILABLE.load(Ordering::SeqCst) {
            return Err(FfiError::InitFailed("Zig bridge is not initialized".into()));
        }

        let op = CString::new(operation)
            .map_err(|_| FfiError::CallFailed("Operation contains an interior NUL byte".into()))?;
        let input = FfiVec::from_mut_slice(data);
        let mut result = unsafe { ffi_dispatch(lang, op.as_ptr(), input) };

        if !result.success {
            let message = if result.error_msg.is_null() {
                "Native Zig dispatch returned failure".to_string()
            } else {
                unsafe { CStr::from_ptr(result.error_msg) }
                    .to_string_lossy()
                    .into_owned()
            };
            unsafe { ffi_free_result(&mut result) };
            return Err(FfiError::CallFailed(message));
        }

        if !result.data.data.is_null() && result.data.len > 0 {
            let output = unsafe { std::slice::from_raw_parts(result.data.data, result.data.len) };
            let copy_len = output.len().min(data.len());
            data[..copy_len].copy_from_slice(&output[..copy_len]);
        }
        unsafe { ffi_free_result(&mut result) };
        record_metrics(started.elapsed().as_nanos() as u64);
        return Ok(true);
    }

    #[cfg(not(has_zig_ffi))]
    {
        let _ = lang;
        let _ = operation;
        for value in data.iter_mut() {
            *value *= 2.0;
        }
        record_metrics(started.elapsed().as_nanos() as u64);
        Ok(true)
    }
}

pub fn dispatch_batch(requests: &[(Language, &str, Vec<f64>)]) -> Vec<Result<bool>> {
    requests
        .iter()
        .map(|(language, operation, payload)| {
            let mut owned = payload.clone();
            dispatch_fast(*language, operation, &mut owned)
        })
        .collect()
}

fn record_metrics(latency_ns: u64) {
    TOTAL_CALLS.fetch_add(1, Ordering::SeqCst);
    TOTAL_LATENCY_NS.fetch_add(latency_ns, Ordering::SeqCst);
}
