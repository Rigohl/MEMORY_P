//! shared_memory/buffer.rs - Shared Memory Buffer using Zig FFI Bridge

use crate::error::{MemoryPError, Result};
use crate::ffi::zig::ZigBridge;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::debug;

/// Shared memory buffer backed by ZigBridge (real FFI when available, Rust fallback)
pub struct SharedMemoryBuffer {
    capacity_bytes: usize,
    bridge: ZigBridge,
    initialized: AtomicBool,
}

impl SharedMemoryBuffer {
    pub fn new() -> Result<Self> {
        Self::with_capacity(100 * 1024 * 1024) // 100 MB
    }

    pub fn with_capacity(capacity_bytes: usize) -> Result<Self> {
        let bridge = ZigBridge::new(capacity_bytes)?;
        Ok(Self {
            capacity_bytes,
            bridge,
            initialized: AtomicBool::new(false),
        })
    }

    pub fn initialize(&self) -> Result<()> {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(());
        }
        self.initialized.store(true, Ordering::Release);
        Ok(())
    }

    pub fn write(&self, data: &[u8]) -> Result<usize> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(MemoryPError::SharedMemoryError(
                "Buffer not initialized".into(),
            ));
        }
        self.bridge.write(data)?;
        debug!("Wrote {} bytes to shared buffer", data.len());
        Ok(data.len())
    }

    pub fn read(&self, offset: usize, len: usize) -> Result<Vec<u8>> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(MemoryPError::SharedMemoryError(
                "Buffer not initialized".into(),
            ));
        }
        let data = self.bridge.read(offset, len)?;
        debug!("Read {} bytes from shared buffer", data.len());
        Ok(data)
    }

    pub fn used_bytes(&self) -> u64 {
        self.bridge.get_info().used as u64
    }

    pub fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }

    pub fn clear(&self) {
        // Reset by creating a fresh bridge would be ideal,
        // but for now just log. The buffer is reusable.
        debug!("Buffer clear requested");
    }
}

impl Default for SharedMemoryBuffer {
    fn default() -> Self {
        Self::new().expect("SharedMemoryBuffer default creation failed")
    }
}

unsafe impl Send for SharedMemoryBuffer {}
unsafe impl Sync for SharedMemoryBuffer {}
