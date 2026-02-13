//! src/ffi/zig.rs - Interfaz con el buffer de memoria compartida de Zig

use std::os::raw::c_void;
use crate::error::Result;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BufferInfo {
    pub capacity: usize,
    pub used: usize,
    pub available: usize,
    pub ref_count: u32,
    pub initialized: bool,
}

extern "C" {
    fn shared_memory_buffer_new(capacity: usize) -> *mut c_void;
    fn shared_memory_buffer_write(buffer: *mut c_void, data: *const u8, len: usize) -> isize;
    fn shared_memory_buffer_read(buffer: *const c_void, offset: usize, dest: *mut u8, len: usize) -> isize;
    #[allow(dead_code)] fn shared_memory_buffer_free(buffer: *mut c_void);
    fn shared_memory_buffer_info(buffer: *const c_void) -> BufferInfo;
    fn shared_memory_buffer_ref(buffer: *mut c_void);
    fn shared_memory_buffer_unref(buffer: *mut c_void);
}

pub struct ZigBridge {
    ptr: *mut c_void,
}

impl ZigBridge {
    pub fn new(capacity: usize) -> Result<Self> {
        unsafe {
            let ptr = shared_memory_buffer_new(capacity);
            if ptr.is_null() {
                return Err(crate::error::MemoryPError::Other("Falló inicialización de buffer Zig".into()));
            }
            Ok(Self { ptr })
        }
    }

    pub fn write(&self, data: &[u8]) -> Result<()> {
        unsafe {
            let res = shared_memory_buffer_write(self.ptr, data.as_ptr(), data.len());
            if res < 0 {
                return Err(crate::error::MemoryPError::Other(format!("Falló escritura en buffer Zig: {}", res)));
            }
            Ok(())
        }
    }

    pub fn read(&self, offset: usize, len: usize) -> Result<Vec<u8>> {
        unsafe {
            let mut out = vec![0u8; len];
            let res = shared_memory_buffer_read(self.ptr, offset, out.as_mut_ptr(), len);
            if res < 0 {
                return Err(crate::error::MemoryPError::Other(format!("Falló lectura de buffer Zig: {}", res)));
            }
            Ok(out)
        }
    }

    pub fn get_info(&self) -> BufferInfo {
        unsafe { shared_memory_buffer_info(self.ptr) }
    }
}

impl Clone for ZigBridge {
    fn clone(&self) -> Self {
        unsafe {
            shared_memory_buffer_ref(self.ptr);
        }
        Self { ptr: self.ptr }
    }
}

impl Drop for ZigBridge {
    fn drop(&mut self) {
        unsafe {
            shared_memory_buffer_unref(self.ptr);
        }
    }
}

unsafe impl Send for ZigBridge {}
unsafe impl Sync for ZigBridge {}
