//! ffi/bridge.rs - Zig FFI Bridge Integration

use super::error::{FfiError, Result};

/// Estructura de información del buffer desde Zig
#[repr(C)]
#[derive(Debug)]
pub struct BufferInfo {
    pub capacity: usize,
    pub used: usize,
    pub available: usize,
    pub ref_count: u32,
    pub initialized: bool,
}

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

/// Crea un nuevo buffer de memoria compartida
#[cfg(feature = "ffi-zig")]
pub fn create_shared_buffer(capacity: usize) -> Option<*mut std::ffi::c_void> {
    unsafe {
        let ptr = shared_memory_buffer_new(capacity);
        if ptr.is_null() {
            None
        } else {
            Some(ptr)
        }
    }
}

#[cfg(not(feature = "ffi-zig"))]
pub fn create_shared_buffer(_capacity: usize) -> Option<*mut std::ffi::c_void> {
    None
}

/// Escribe datos al buffer compartido
#[cfg(feature = "ffi-zig")]
pub fn write_to_buffer(buffer: *mut std::ffi::c_void, data: &[u8]) -> Result<usize> {
    unsafe {
        let written = shared_memory_buffer_write(buffer, data.as_ptr(), data.len());
        if written < 0 {
            Err(FfiError::ZigError(format!("Error escribiendo al buffer: {}", written)))
        } else {
            Ok(written as usize)
        }
    }
}

#[cfg(not(feature = "ffi-zig"))]
pub fn write_to_buffer(_buffer: *mut std::ffi::c_void, _data: &[u8]) -> Result<usize> {
    Err(FfiError::ZigError("Zig FFI no disponible".to_string()))
}

/// Lee datos del buffer compartido
#[cfg(feature = "ffi-zig")]
pub fn read_from_buffer(buffer: *const std::ffi::c_void, offset: usize, len: usize) -> Result<Vec<u8>> {
    unsafe {
        let mut buf = vec![0u8; len];
        let read = shared_memory_buffer_read(buffer, offset, buf.as_mut_ptr(), len);
        if read < 0 {
            Err(FfiError::ZigError(format!("Error leyendo del buffer: {}", read)))
        } else {
            buf.truncate(read as usize);
            Ok(buf)
        }
    }
}

#[cfg(not(feature = "ffi-zig"))]
pub fn read_from_buffer(_buffer: *const std::ffi::c_void, _offset: usize, _len: usize) -> Result<Vec<u8>> {
    Err(FfiError::ZigError("Zig FFI no disponible".to_string()))
}

/// Obtiene información del buffer
#[cfg(feature = "ffi-zig")]
pub fn get_buffer_info(buffer: *const std::ffi::c_void) -> BufferInfo {
    unsafe {
        shared_memory_buffer_info(buffer)
    }
}

#[cfg(not(feature = "ffi-zig"))]
pub fn get_buffer_info(_buffer: *const std::ffi::c_void) -> BufferInfo {
    BufferInfo {
        capacity: 0,
        used: 0,
        available: 0,
        ref_count: 0,
        initialized: false,
    }
}

/// Libera el buffer compartido
#[cfg(feature = "ffi-zig")]
pub fn free_shared_buffer(buffer: *mut std::ffi::c_void) {
    unsafe {
        shared_memory_buffer_free(buffer);
    }
}

#[cfg(not(feature = "ffi-zig"))]
pub fn free_shared_buffer(_buffer: *mut std::ffi::c_void) {
    // No-op
}

#[cfg(feature = "ffi-zig")]
#[link(name = "zig_bridge")]
extern "C" {
    fn ffi_init() -> bool;
    fn ffi_shutdown();
    
    // Shared memory buffer functions
    fn shared_memory_buffer_new(capacity: usize) -> *mut std::ffi::c_void;
    fn shared_memory_buffer_write(buffer: *mut std::ffi::c_void, data: *const u8, len: usize) -> isize;
    fn shared_memory_buffer_read(buffer: *const std::ffi::c_void, offset: usize, dest: *mut u8, len: usize) -> isize;
    fn shared_memory_buffer_info(buffer: *const std::ffi::c_void) -> BufferInfo;
    fn shared_memory_buffer_clear(buffer: *mut std::ffi::c_void);
    fn shared_memory_buffer_free(buffer: *mut std::ffi::c_void);
}
