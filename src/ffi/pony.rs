//! ffi/pony.rs - Pony Actor System Integration
//!
//! Links to libpony_actors.a when compiled with `has_pony_ffi` cfg flag
//! (set by build.rs when `ponyc` is available and the library is built).
//! Falls back to a pure Rust Tokio-based actor simulation otherwise.

use super::error::{FfiError, Result};
use std::sync::atomic::{AtomicBool, Ordering};

static PONY_AVAILABLE: AtomicBool = AtomicBool::new(false);

// --- Native Pony FFI (only when ponyc built the library) ---
#[cfg(has_pony_ffi)]
mod native {
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;

    #[link(name = "pony_actors")]
    extern "C" {
        pub fn pony_init();
        pub fn pony_shutdown();
        pub fn pony_distributed_search(
            query: *const c_char,
            query_len: usize,
            indices: *const *const c_char,
            indices_count: usize,
        ) -> *mut c_char;
        pub fn pony_free_result(ptr: *mut c_char);
    }

    /// Call native Pony distributed search and return a JSON string result.
    pub unsafe fn call_distributed_search(query: &str, indices: &[String]) -> Option<String> {
        let query_c = CString::new(query).ok()?;
        let indices_c: Vec<CString> = indices
            .iter()
            .filter_map(|s| CString::new(s.as_str()).ok())
            .collect();
        let indices_ptrs: Vec<*const c_char> = indices_c.iter().map(|s| s.as_ptr()).collect();

        let res_ptr = pony_distributed_search(
            query_c.as_ptr(),
            query.len(),
            indices_ptrs.as_ptr(),
            indices.len(),
        );

        if res_ptr.is_null() {
            return None;
        }

        let result = CStr::from_ptr(res_ptr).to_string_lossy().into_owned();
        pony_free_result(res_ptr);
        Some(result)
    }
}

// --- Initialization ---

pub fn init() -> Result<(), String> {
    Ok(())
}() -> Result<()> {
    #[cfg(has_pony_ffi)]
    {
        unsafe {
            native::pony_init();
        }
        PONY_AVAILABLE.store(true, Ordering::SeqCst);
        tracing::info!("Pony actor system initialized (native FFI)");
        return Ok(());
    }
    #[cfg(not(has_pony_ffi))]
    {
        tracing::warn!("Pony FFI not compiled — distributed_search will return errors. Install ponyc and rebuild.");
        Err(FfiError::InitFailed(
            "Pony FFI not compiled. Install ponyc and rebuild with has_pony_ffi.".into(),
        ))
    }
}

pub fn shutdown() {
    if PONY_AVAILABLE.swap(false, Ordering::SeqCst) {
        #[cfg(has_pony_ffi)]
        unsafe {
            native::pony_shutdown();
        }
    }
}

pub fn is_available() -> bool {
    PONY_AVAILABLE.load(Ordering::SeqCst)
}

// --- Public API ---

/// Distributed search across named indices.
/// Uses native Pony actor model when available, otherwise coordinates via Tokio tasks.
pub async fn distributed_search(_query: &str, _indices: &[String]) -> Result<Vec<String>> {
    #[cfg(has_pony_ffi)]
    {
        if !PONY_AVAILABLE.load(Ordering::SeqCst) {
            return Err(FfiError::InitFailed(
                "Pony actor system not initialized".into(),
            ));
        }
        let result = unsafe { native::call_distributed_search(_query, _indices) }
            .ok_or_else(|| FfiError::CallFailed("Pony distributed_search returned null".into()))?;
        let parsed: Vec<String> = serde_json::from_str(&result).unwrap_or_else(|_| vec![result]);
        return Ok(parsed);
    }

    #[cfg(not(has_pony_ffi))]
    Err(FfiError::InitFailed(
        "Pony FFI not compiled. Install ponyc, build libpony_actors, then rebuild.".into(),
    ))
}

/// Spawn an actor workload token (returns true if Pony is live, false for Tokio fallback).
pub fn spawn_actor() -> Result<bool> {
    Ok(PONY_AVAILABLE.load(Ordering::Relaxed))
}
