//! src/ffi/zig.rs - Interfaz con el bridge de Zig

use std::os::raw::{c_int, c_void};
use crate::error::Result;

extern "C" {
    // Definiciones que coinciden con brain/zig/ffi_bridge.zig
    fn init_shared_buffer(size: usize) -> *mut c_void;
    fn write_to_buffer(buffer: *mut c_void, data: *const u8, len: usize) -> c_int;
    fn read_from_buffer(buffer: *mut c_void, out: *mut u8, len: usize) -> c_int;
    fn free_shared_buffer(buffer: *mut c_void);
}

pub struct ZigBridge {
    buffer: *mut c_void,
}

impl ZigBridge {
    pub fn new(size: usize) -> Result<Self> {
        unsafe {
            let buffer = init_shared_buffer(size);
            if buffer.is_null() {
                return Err(crate::error::MemoryPError::Other("Falló inicialización de buffer Zig".into()));
            }
            Ok(Self { buffer })
        }
    }

    pub fn write(&self, data: &[u8]) -> Result<()> {
        unsafe {
            let res = write_to_buffer(self.buffer, data.as_ptr(), data.len());
            if res != 0 {
                return Err(crate::error::MemoryPError::Other("Falló escritura en buffer Zig".into()));
            }
            Ok(())
        }
    }

    pub fn read(&self, len: usize) -> Result<Vec<u8>> {
        unsafe {
            let mut out = vec![0u8; len];
            let res = read_from_buffer(self.buffer, out.as_mut_ptr(), len);
            if res != 0 {
                return Err(crate::error::MemoryPError::Other("Falló lectura de buffer Zig".into()));
            }
            Ok(out)
        }
    }
}

impl Drop for ZigBridge {
    fn drop(&mut self) {
        unsafe {
            free_shared_buffer(self.buffer);
        }
    }
}

// Implementación segura para Threads
unsafe impl Send for ZigBridge {}
unsafe impl Sync for ZigBridge {}
