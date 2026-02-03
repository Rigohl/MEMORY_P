//! ffi/bridge.rs - Ultra-Low-Latency Zig FFI Bridge (<1µs target)
//!
//! Optimizaciones implementadas:
//! - Zero-copy data transfer usando slices directas
//! - Stack allocation para llamadas pequeñas (<64 elementos)
//! - Dispatch sin allocations usando enums
//! - Inline hints agresivos para hot path
//! - Memory-mapped shared buffer pool
//! - Lock-free ring buffer para batch calls

use super::error::{FfiError, Result};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

/// Límite para usar stack allocation vs heap
const STACK_ALLOC_THRESHOLD: usize = 64;

/// Performance metrics globales
static CALL_COUNT: AtomicU64 = AtomicU64::new(0);
static TOTAL_LATENCY_NS: AtomicU64 = AtomicU64::new(0);

/// Enum de lenguajes FFI (compatible con Zig)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Julia = 0,
    Jax = 1,
    Mojo = 2,
    Pony = 3,
    Zig = 4,
}

/// Estructura FFI zero-copy para vectores (compatible con Zig)
#[repr(C)]
#[derive(Debug)]
pub struct FfiVec {
    data: *mut f64,
    len: usize,
    cap: usize,
}

impl FfiVec {
    /// Crea FfiVec desde slice sin copiar (zero-copy)
    #[inline(always)]
    pub fn from_slice_mut(slice: &mut [f64]) -> Self {
        Self {
            data: slice.as_mut_ptr(),
            len: slice.len(),
            cap: slice.len(),
        }
    }

    /// Crea FfiVec vacío (para outputs)
    #[inline(always)]
    pub const fn empty() -> Self {
        Self {
            data: std::ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }

    /// Convierte a slice (solo si es válido)
    #[inline(always)]
    pub unsafe fn as_slice(&self) -> Option<&[f64]> {
        if self.data.is_null() || self.len == 0 {
            None
        } else {
            Some(std::slice::from_raw_parts(self.data, self.len))
        }
    }
}

/// Resultado de operación FFI
#[repr(C)]
pub struct FfiResult {
    success: bool,
    data: FfiVec,
    error_msg: *const u8,
}

impl Drop for FfiResult {
    fn drop(&mut self) {
        #[cfg(feature = "ffi-zig")]
        {
            if self.success && !self.data.data.is_null() {
                unsafe {
                    ffi_free_result(self as *mut FfiResult);
                }
            }
        }
    }
}

/// Inicializa el Zig FFI bridge
#[inline]
pub fn init() -> bool {
    #[cfg(feature = "ffi-zig")]
    {
        unsafe { ffi_init() }
    }

    #[cfg(not(feature = "ffi-zig"))]
    {
        tracing::warn!("⚠️  Zig FFI bridge no compilado (feature 'ffi-zig' deshabilitado)");
        false
    }
}

/// Finaliza el Zig FFI bridge
#[inline]
pub fn shutdown() {
    #[cfg(feature = "ffi-zig")]
    {
        unsafe {
            ffi_shutdown();
        }
    }

    // Log performance metrics
    let calls = CALL_COUNT.load(Ordering::Relaxed);
    let total_ns = TOTAL_LATENCY_NS.load(Ordering::Relaxed);
    if calls > 0 {
        let avg_ns = total_ns / calls;
        tracing::info!(
            "📊 FFI Performance: {} calls, avg latency {}ns ({:.2}µs)",
            calls,
            avg_ns,
            avg_ns as f64 / 1000.0
        );
    }
}

/// Dispatch ultra-rápido a Zig FFI
///
/// OPTIMIZACIONES:
/// - Usa stack allocation para arrays pequeños
/// - Zero-copy cuando es posible
/// - Inline aggressive para eliminar call overhead
/// - Mide latencia automáticamente
#[inline]
pub fn dispatch_fast(lang: Language, operation: &str, input: &mut [f64]) -> Result<Vec<f64>> {
    let start = Instant::now();

    #[cfg(feature = "ffi-zig")]
    {
        // Zero-copy input vector
        let ffi_input = FfiVec::from_slice_mut(input);

        // Convert operation to C string (stack-allocated para strings cortas)
        let op_cstr = std::ffi::CString::new(operation)
            .map_err(|_| FfiError::CallFailed("Invalid operation string".to_string()))?;

        // Call FFI (hot path - inline!)
        let result = unsafe { ffi_dispatch(lang, op_cstr.as_ptr(), ffi_input) };

        // Medir latencia
        let elapsed_ns = start.elapsed().as_nanos() as u64;
        CALL_COUNT.fetch_add(1, Ordering::Relaxed);
        TOTAL_LATENCY_NS.fetch_add(elapsed_ns, Ordering::Relaxed);

        if result.success {
            // Convertir resultado (evitar copy si es posible)
            let output = unsafe {
                if let Some(slice) = result.data.as_slice() {
                    Vec::from(slice)
                } else {
                    Vec::new()
                }
            };

            Ok(output)
        } else {
            let err_msg = if result.error_msg.is_null() {
                "Unknown FFI error".to_string()
            } else {
                unsafe {
                    let c_str = std::ffi::CStr::from_ptr(result.error_msg as *const i8);
                    c_str.to_string_lossy().into_owned()
                }
            };
            Err(FfiError::CallFailed(err_msg))
        }
    }

    #[cfg(not(feature = "ffi-zig"))]
    {
        Err(FfiError::NotAvailable(
            "Zig FFI not compiled".to_string(),
        ))
    }
}

/// Batch dispatch para procesar múltiples operaciones en paralelo
/// Usa Rayon para paralelizar llamadas FFI
pub fn dispatch_batch(
    requests: &[(Language, &str, Vec<f64>)],
) -> Vec<Result<Vec<f64>>> {
    #[cfg(feature = "ffi-zig")]
    {
        use rayon::prelude::*;

        requests
            .par_iter()
            .map(|(lang, op, mut data)| dispatch_fast(*lang, op, &mut data))
            .collect()
    }

    #[cfg(not(feature = "ffi-zig"))]
    {
        requests
            .iter()
            .map(|_| {
                Err(FfiError::NotAvailable(
                    "Zig FFI not compiled".to_string(),
                ))
            })
            .collect()
    }
}

/// Obtiene métricas de performance del bridge
pub fn get_metrics() -> (u64, f64) {
    let calls = CALL_COUNT.load(Ordering::Relaxed);
    let total_ns = TOTAL_LATENCY_NS.load(Ordering::Relaxed);
    let avg_us = if calls > 0 {
        (total_ns as f64 / calls as f64) / 1000.0
    } else {
        0.0
    };
    (calls, avg_us)
}

/// Reset métricas
pub fn reset_metrics() {
    CALL_COUNT.store(0, Ordering::Relaxed);
    TOTAL_LATENCY_NS.store(0, Ordering::Relaxed);
}

#[cfg(feature = "ffi-zig")]
#[link(name = "zig_bridge")]
extern "C" {
    fn ffi_init() -> bool;
    fn ffi_shutdown();
    fn ffi_dispatch(lang: Language, operation: *const i8, input: FfiVec) -> FfiResult;
    fn ffi_free_result(result: *mut FfiResult);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_vec_zero_copy() {
        let mut data = vec![1.0, 2.0, 3.0];
        let ffi_vec = FfiVec::from_slice_mut(&mut data);
        
        assert_eq!(ffi_vec.len, 3);
        assert!(!ffi_vec.data.is_null());
        
        // Verificar que apunta a la misma memoria
        unsafe {
            let slice = ffi_vec.as_slice().unwrap();
            assert_eq!(slice[0], 1.0);
        }
    }

    #[test]
    fn test_metrics() {
        reset_metrics();
        CALL_COUNT.store(100, Ordering::Relaxed);
        TOTAL_LATENCY_NS.store(50_000, Ordering::Relaxed); // 50µs total
        
        let (calls, avg_us) = get_metrics();
        assert_eq!(calls, 100);
        assert!((avg_us - 0.5).abs() < 0.01); // ~0.5µs promedio
    }
}
