//! ffi/bridge.rs - Zig FFI Bridge Integration

use super::error::{FfiError, Result};
use std::ffi::CString;

/// Inicializa el Zig FFI bridge
pub fn init() -> bool {
    #[cfg(feature = "ffi-zig")]
    {
        unsafe {
            ffi_init()
        }
    }

    #[cfg(not(feature = "ffi-zig"))]
    {
        tracing::warn!("⚠️  Zig FFI bridge no compilado (feature 'ffi-zig' deshabilitado)");
        false
    }
}

/// Finaliza el Zig FFI bridge
pub fn shutdown() {
    #[cfg(feature = "ffi-zig")]
    {
        unsafe {
            ffi_shutdown();
        }
    }
}

#[cfg(feature = "ffi-zig")]
#[link(name = "zig_bridge")]
extern "C" {
    fn ffi_init() -> bool;
    fn ffi_shutdown();
}
