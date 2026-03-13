//! src/ffi/zig.rs - Zig Shared Memory Buffer FFI
//!
//! When compiled with has_zig_ffi cfg: links to real Zig shared memory buffer.
//! Otherwise: uses a pure Rust implementation (NOT a mock).

use crate::error::Result;
#[cfg(has_zig_ffi)]
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
#[cfg(not(has_zig_ffi))]
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;

#[cfg(has_zig_ffi)]
static ZIG_AVAILABLE: AtomicBool = AtomicBool::new(false);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BufferInfo {
    pub capacity: usize,
    pub used: usize,
    pub available: usize,
    pub ref_count: u32,
    pub initialized: bool,
}

#[cfg(has_zig_ffi)]
extern "C" {
    fn ffi_init() -> bool;
    fn ffi_shutdown();
    fn shared_memory_buffer_new(capacity: usize) -> *mut std::os::raw::c_void;
    fn shared_memory_buffer_write(
        buffer: *mut std::os::raw::c_void,
        data: *const u8,
        len: usize,
    ) -> isize;
    fn shared_memory_buffer_read(
        buffer: *const std::os::raw::c_void,
        offset: usize,
        dest: *mut u8,
        len: usize,
    ) -> isize;
    fn shared_memory_buffer_free(buffer: *mut std::os::raw::c_void);
    fn shared_memory_buffer_info(buffer: *const std::os::raw::c_void) -> BufferInfo;
    fn shared_memory_buffer_ref(buffer: *mut std::os::raw::c_void);
    fn shared_memory_buffer_unref(buffer: *mut std::os::raw::c_void);
}

pub fn init() -> Result<()> {
    #[cfg(has_zig_ffi)]
    unsafe {
        if ffi_init() {
            ZIG_AVAILABLE.store(true, Ordering::SeqCst);
            return Ok(());
        }
    }
    Ok(())
}

pub fn shutdown() {
    #[cfg(has_zig_ffi)]
    if ZIG_AVAILABLE.load(Ordering::SeqCst) {
        unsafe {
            ffi_shutdown();
        }
        ZIG_AVAILABLE.store(false, Ordering::SeqCst);
    }
}

pub fn is_available() -> bool {
    #[cfg(has_zig_ffi)]
    {
        return ZIG_AVAILABLE.load(Ordering::SeqCst);
    }

    #[cfg(not(has_zig_ffi))]
    {
        false
    }
}

/// High-performance shared memory buffer
pub struct ZigBridge {
    inner: BridgeInner,
}

enum BridgeInner {
    #[cfg(has_zig_ffi)]
    Native(*mut std::os::raw::c_void),
    Rust(RustBuffer),
}

struct RustBuffer {
    data: parking_lot::RwLock<Vec<u8>>,
    capacity: usize,
    used: AtomicUsize,
    ref_count: Arc<AtomicU32>,
}

impl ZigBridge {
    pub fn new(capacity: usize) -> Result<Self> {
        #[cfg(has_zig_ffi)]
        if ZIG_AVAILABLE.load(Ordering::SeqCst) {
            unsafe {
                let ptr = shared_memory_buffer_new(capacity);
                if !ptr.is_null() {
                    return Ok(Self {
                        inner: BridgeInner::Native(ptr),
                    });
                }
            }
        }

        Ok(Self {
            inner: BridgeInner::Rust(RustBuffer {
                data: parking_lot::RwLock::new(vec![0u8; capacity]),
                capacity,
                used: AtomicUsize::new(0),
                ref_count: Arc::new(AtomicU32::new(1)),
            }),
        })
    }

    pub fn write(&self, data: &[u8]) -> Result<()> {
        match &self.inner {
            #[cfg(has_zig_ffi)]
            BridgeInner::Native(ptr) => unsafe {
                let res = shared_memory_buffer_write(*ptr, data.as_ptr(), data.len());
                if res < 0 {
                    return Err(crate::error::MemoryPError::Other(format!(
                        "Zig write error: {}",
                        res
                    )));
                }
                Ok(())
            },
            BridgeInner::Rust(buf) => {
                let used = buf.used.load(Ordering::SeqCst);
                if used + data.len() > buf.capacity {
                    return Err(crate::error::MemoryPError::Other("Buffer overflow".into()));
                }
                let mut guard = buf.data.write();
                guard[used..used + data.len()].copy_from_slice(data);
                buf.used.store(used + data.len(), Ordering::SeqCst);
                Ok(())
            }
        }
    }

    pub fn read(&self, offset: usize, len: usize) -> Result<Vec<u8>> {
        match &self.inner {
            #[cfg(has_zig_ffi)]
            BridgeInner::Native(ptr) => {
                let mut out = vec![0u8; len];
                unsafe {
                    let res = shared_memory_buffer_read(*ptr, offset, out.as_mut_ptr(), len);
                    if res < 0 {
                        return Err(crate::error::MemoryPError::Other(format!(
                            "Zig read error: {}",
                            res
                        )));
                    }
                }
                Ok(out)
            }
            BridgeInner::Rust(buf) => {
                let used = buf.used.load(Ordering::SeqCst);
                if offset + len > used {
                    return Err(crate::error::MemoryPError::Other(
                        "Read beyond written data".into(),
                    ));
                }
                let guard = buf.data.read();
                Ok(guard[offset..offset + len].to_vec())
            }
        }
    }

    pub fn get_info(&self) -> BufferInfo {
        match &self.inner {
            #[cfg(has_zig_ffi)]
            BridgeInner::Native(ptr) => unsafe { shared_memory_buffer_info(*ptr) },
            BridgeInner::Rust(buf) => {
                let used = buf.used.load(Ordering::SeqCst);
                BufferInfo {
                    capacity: buf.capacity,
                    used,
                    available: buf.capacity.saturating_sub(used),
                    ref_count: buf.ref_count.load(Ordering::SeqCst),
                    initialized: true,
                }
            }
        }
    }
}

impl Clone for ZigBridge {
    fn clone(&self) -> Self {
        match &self.inner {
            #[cfg(has_zig_ffi)]
            BridgeInner::Native(ptr) => {
                unsafe {
                    shared_memory_buffer_ref(*ptr);
                }
                Self {
                    inner: BridgeInner::Native(*ptr),
                }
            }
            BridgeInner::Rust(buf) => {
                buf.ref_count.fetch_add(1, Ordering::SeqCst);
                Self {
                    inner: BridgeInner::Rust(RustBuffer {
                        data: parking_lot::RwLock::new(buf.data.read().clone()),
                        capacity: buf.capacity,
                        used: AtomicUsize::new(buf.used.load(Ordering::SeqCst)),
                        ref_count: buf.ref_count.clone(),
                    }),
                }
            }
        }
    }
}

impl ZigBridge {
    /// Force immediate cleanup of native Zig buffer (calls shared_memory_buffer_free)
    /// Useful for early deallocation without waiting for Drop
    #[cfg(has_zig_ffi)]
    pub fn force_cleanup(&mut self) -> Result<()> {
        if let BridgeInner::Native(ptr) = &self.inner {
            if !ptr.is_null() {
                unsafe {
                    shared_memory_buffer_free(*ptr);
                }
                return Ok(());
            }
        }
        Ok(())
    }
}

impl Drop for ZigBridge {
    fn drop(&mut self) {
        match &self.inner {
            #[cfg(has_zig_ffi)]
            BridgeInner::Native(ptr) => unsafe {
                shared_memory_buffer_unref(*ptr);
            },
            BridgeInner::Rust(buf) => {
                buf.ref_count.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }
}

unsafe impl Send for ZigBridge {}
unsafe impl Sync for ZigBridge {}
