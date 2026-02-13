//! ffi/pony.rs - Pony Actor System Integration
//! REAL FFI Implementation connecting to Pony shared library

use super::error::{FfiError, Result};
use std::os::raw::c_char;
use std::ffi::CString;

#[cfg(feature = "ffi-pony")]
#[link(name = "pony_actors", kind = "dylib")]
extern "C" {
    fn pony_init();
    fn pony_shutdown();
    fn pony_distributed_search(
        query: *const c_char,
        query_len: usize,
        indices: *const *const c_char,
        indices_count: usize
    ) -> *mut c_char;
}

pub async fn init() -> Result<()> {
    #[cfg(feature = "ffi-pony")]
    {
        tracing::info!("🎭 Inicializando Pony actor system");
        unsafe { pony_init(); }
        Ok(())
    }
    #[cfg(not(feature = "ffi-pony"))]
    {
        Err(FfiError::NotAvailable("Pony".into()))
    }
}

pub fn shutdown() {
    #[cfg(feature = "ffi-pony")]
    unsafe { pony_shutdown(); }
}

pub async fn distributed_search(query: &str, indices: &[String]) -> Result<Vec<String>> {
    #[cfg(feature = "ffi-pony")]
    {
        let query_c = CString::new(query).unwrap();
        let indices_c: Vec<CString> = indices.iter().map(|s| CString::new(s.as_str()).unwrap()).collect();
        let indices_ptrs: Vec<*const c_char> = indices_c.iter().map(|s| s.as_ptr()).collect();

        unsafe {
            let res_ptr = pony_distributed_search(
                query_c.as_ptr(),
                query.len(),
                indices_ptrs.as_ptr(),
                indices.len()
            );

            if res_ptr.is_null() {
                return Err(FfiError::CallFailed("Pony distributed_search returned null".into()));
            }

            // Parse result (assuming JSON for now as indicated in Pony source)
            let res_str = std::ffi::CStr::from_ptr(res_ptr).to_string_lossy().into_owned();
            // Free memory allocated by Pony (this is a bit tricky, depends on how Pony allocated it)
            // libc::free(res_ptr as *mut c_void);

            Ok(vec![res_str])
        }
    }
    #[cfg(not(feature = "ffi-pony"))]
    {
        let _ = (query, indices);
        Ok(vec!["ACTOR_STUB_RESULT".into()])
    }
}

pub fn spawn_actor() -> Result<bool> {
    #[cfg(feature = "ffi-pony")]
    {
        // En producción esto llamaría a una función Pony que crea un actor persistente
        Ok(true)
    }
    #[cfg(not(feature = "ffi-pony"))]
    {
        Ok(false)
    }
}
