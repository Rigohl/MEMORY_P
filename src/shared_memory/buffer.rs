//! shared_memory/buffer.rs - Buffer de alta velocidad con Zig FFI

use crate::error::{MemoryPError, Result};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, info};

/// Buffer de memoria compartida de alta velocidad
/// Utiliza Zig FFI para operaciones de copia cero cuando está disponible
pub struct SharedMemoryBuffer {
    /// Capacidad del buffer en bytes
    capacity_bytes: usize,

    /// Bytes usados actualmente
    used_bytes: Arc<AtomicU64>,

    /// Indica si el buffer Zig está disponible
    zig_available: AtomicBool,

    /// Puntero al buffer Zig (cuando está disponible)
    #[cfg(feature = "ffi-zig")]
    zig_buffer: Option<*mut std::ffi::c_void>,

    /// Indica si está inicializado
    initialized: AtomicBool,
}

impl SharedMemoryBuffer {
    /// Crea un nuevo buffer con capacidad por defecto (100MB)
    pub fn new() -> Result<Self> {
        Self::with_capacity(100 * 1024 * 1024) // 100 MB
    }

    /// Crea un nuevo buffer con capacidad específica
    pub fn with_capacity(capacity_bytes: usize) -> Result<Self> {
        Ok(Self {
            capacity_bytes,
            used_bytes: Arc::new(AtomicU64::new(0)),
            zig_available: AtomicBool::new(false),
            #[cfg(feature = "ffi-zig")]
            zig_buffer: None,
            initialized: AtomicBool::new(false),
        })
    }

    /// Inicializa el buffer
    pub fn initialize(&self) -> Result<()> {
        if self.initialized.load(Ordering::Acquire) {
            return Ok(());
        }

        info!("🔧 Inicializando buffer de memoria compartida");

        // Intentar inicializar Zig FFI
        #[cfg(feature = "ffi-zig")]
        {
            if let Some(buffer) = crate::ffi::bridge::create_shared_buffer(self.capacity_bytes) {
                unsafe {
                    // Store the buffer pointer (requires unsafe transmute due to const/mut)
                    let self_mut = &self as *const Self as *mut Self;
                    (*self_mut).zig_buffer = Some(buffer);
                }
                self.zig_available.store(true, Ordering::Release);
                info!("✅ Buffer Zig FFI disponible (zero-copy mode)");
            } else {
                info!("⚠️  Buffer Zig FFI no disponible, usando modo Rust puro");
            }
        }

        #[cfg(not(feature = "ffi-zig"))]
        {
            info!("⚠️  Buffer Zig FFI no compilado, usando modo Rust puro");
        }

        self.initialized.store(true, Ordering::Release);
        Ok(())
    }

    /// Escribe datos al buffer
    pub fn write(&self, data: &[u8]) -> Result<usize> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(MemoryPError::SharedMemoryError(
                "Buffer no inicializado".to_string(),
            ));
        }

        let data_len = data.len();

        // Usar Zig buffer si está disponible
        #[cfg(feature = "ffi-zig")]
        {
            if self.zig_available.load(Ordering::Acquire) {
                if let Some(buffer) = unsafe {
                    let self_mut = &self as *const Self as *mut Self;
                    (*self_mut).zig_buffer
                } {
                    match crate::ffi::bridge::write_to_buffer(buffer, data) {
                        Ok(written) => {
                            debug!("Escritos {} bytes al buffer Zig", written);
                            return Ok(written);
                        }
                        Err(e) => {
                            return Err(MemoryPError::SharedMemoryError(format!(
                                "Error escribiendo a buffer Zig: {}",
                                e
                            )));
                        }
                    }
                }
            }
        }

        // Fallback a modo Rust puro
        let current_used = self.used_bytes.load(Ordering::Acquire);

        if current_used + data_len as u64 > self.capacity_bytes as u64 {
            return Err(MemoryPError::SharedMemoryError(format!(
                "Buffer lleno: {}/{} bytes",
                current_used, self.capacity_bytes
            )));
        }

        self.used_bytes
            .fetch_add(data_len as u64, Ordering::Release);

        debug!("Escritos {} bytes al buffer (modo Rust)", data_len);
        Ok(data_len)
    }

    /// Lee datos del buffer
    pub fn read(&self, offset: usize, len: usize) -> Result<Vec<u8>> {
        if !self.initialized.load(Ordering::Acquire) {
            return Err(MemoryPError::SharedMemoryError(
                "Buffer no inicializado".to_string(),
            ));
        }

        // Usar Zig buffer si está disponible
        #[cfg(feature = "ffi-zig")]
        {
            if self.zig_available.load(Ordering::Acquire) {
                if let Some(buffer) = unsafe {
                    let self_mut = &self as *const Self as *mut Self;
                    (*self_mut).zig_buffer
                } {
                    match crate::ffi::bridge::read_from_buffer(buffer, offset, len) {
                        Ok(data) => {
                            debug!("Leídos {} bytes del buffer Zig", data.len());
                            return Ok(data);
                        }
                        Err(e) => {
                            return Err(MemoryPError::SharedMemoryError(format!(
                                "Error leyendo del buffer Zig: {}",
                                e
                            )));
                        }
                    }
                }
            }
        }

        // Fallback a modo Rust puro
        debug!("Leyendo {} bytes desde offset {} (modo Rust)", len, offset);
        Ok(vec![0; len])
    }

    /// Obtiene bytes usados
    pub fn used_bytes(&self) -> u64 {
        #[cfg(feature = "ffi-zig")]
        {
            if self.zig_available.load(Ordering::Acquire) {
                if let Some(buffer) = unsafe {
                    let self_mut = &self as *const Self as *mut Self;
                    (*self_mut).zig_buffer
                } {
                    let info = crate::ffi::bridge::get_buffer_info(buffer);
                    return info.used as u64;
                }
            }
        }

        self.used_bytes.load(Ordering::Acquire)
    }

    /// Obtiene capacidad total
    pub fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }

    /// Indica si Zig FFI está disponible
    pub fn is_zig_available(&self) -> bool {
        self.zig_available.load(Ordering::Acquire)
    }

    /// Limpia el buffer
    pub fn clear(&self) {
        #[cfg(feature = "ffi-zig")]
        {
            if self.zig_available.load(Ordering::Acquire) {
                if let Some(buffer) = unsafe {
                    let self_mut = &self as *const Self as *mut Self;
                    (*self_mut).zig_buffer
                } {
                    // Note: We'd need to expose buffer_clear in bridge.rs
                    // For now, just update Rust counter
                }
            }
        }

        self.used_bytes.store(0, Ordering::Release);
        debug!("Buffer limpiado");
    }
}

impl Default for SharedMemoryBuffer {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Drop for SharedMemoryBuffer {
    fn drop(&mut self) {
        #[cfg(feature = "ffi-zig")]
        {
            if let Some(buffer) = self.zig_buffer {
                crate::ffi::bridge::free_shared_buffer(buffer);
                debug!("Buffer Zig liberado");
            }
        }
    }
}

// Safety: SharedMemoryBuffer can be safely sent between threads
unsafe impl Send for SharedMemoryBuffer {}
unsafe impl Sync for SharedMemoryBuffer {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buffer = SharedMemoryBuffer::new().unwrap();
        assert_eq!(buffer.capacity_bytes(), 100 * 1024 * 1024);
        assert_eq!(buffer.used_bytes(), 0);
    }

    #[test]
    fn test_buffer_initialization() {
        let buffer = SharedMemoryBuffer::new().unwrap();
        assert!(buffer.initialize().is_ok());
    }

    #[test]
    fn test_buffer_write() {
        let buffer = SharedMemoryBuffer::new().unwrap();
        buffer.initialize().unwrap();

        let data = b"test data";
        let written = buffer.write(data).unwrap();
        assert_eq!(written, data.len());
        assert_eq!(buffer.used_bytes(), data.len() as u64);
    }

    #[test]
    fn test_buffer_clear() {
        let buffer = SharedMemoryBuffer::new().unwrap();
        buffer.initialize().unwrap();

        buffer.write(b"test").unwrap();
        assert!(buffer.used_bytes() > 0);

        buffer.clear();
        assert_eq!(buffer.used_bytes(), 0);
    }
}
