/// REAL Julia FFI Implementation - Phase 10
/// 
/// Connects Rust to brain/julia/julia_math.jl @ccallable functions
/// This replaces the stub implementations with actual FFI calls

use super::error::{FfiError, Result};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicBool, Ordering};

static JULIA_AVAILABLE: AtomicBool = AtomicBool::new(false);

// ============================================================================
// REAL JULIA FFI C BINDINGS - Calls brain/julia/julia_math.jl
// ============================================================================

#[cfg(has_julia_ffi)]
extern "C" {
    /// Real Julia @ccallable function from brain/julia/julia_math.jl
    /// Signature: julia_optimize_weights_ffi(data::Ptr{Float64}, len::Cint, result::Ptr{Float64}) -> Cint
    /// Located: brain/julia/julia_math.jl lines 218-242
    /// Returns: 0 on success, -1 on error
    fn julia_optimize_weights_ffi(
        data: *const f64,
        len: i32,
        result: *mut f64,
    ) -> i32;

    /// Real Julia @ccallable function from brain/julia/julia_math.jl
    /// Signature: julia_chaos_analysis_ffi(data::Ptr{Float64}, len::Cint) -> Float64
    /// Located: brain/julia/julia_math.jl lines 244-256
    /// Returns: Lyapunov exponent or NaN on error
    fn julia_chaos_analysis_ffi(
        data: *const f64,
        len: i32,
    ) -> f64;

    /// Real Julia @ccallable function from brain/julia/julia_math.jl
    /// Signature: julia_get_decision_ffi(entropy, chaos, stability, result_buf, buf_len) -> Cint
    /// Located: brain/julia/julia_math.jl lines 397-421
    /// Returns: Length of decision string written to buffer
    fn julia_get_decision_ffi(
        entropy: f64,
        chaos: f64,
        stability: f64,
        result_buf: *mut u8,
        buf_len: i32,
    ) -> i32;

    /// Julia init function
    fn julia_init() -> i32;

    /// Julia shutdown function
    fn julia_shutdown() -> i32;
}

/// Initialize Julia Runtime
/// Calls julia_init() from brain/julia/julia_math.jl
pub fn init() -> Result<()> {
    #[cfg(has_julia_ffi)]
    unsafe {
        match julia_init() {
            0 => {
                JULIA_AVAILABLE.store(true, Ordering::SeqCst);
                tracing::info!("[Julia FFI] ✅ Julia runtime initialized");
                Ok(())
            }
            code => {
                tracing::error!("[Julia FFI] ❌ julia_init failed with code: {}", code);
                Err(FfiError::InitFailed(format!("julia_init returned: {}", code)).into())
            }
        }
    }

    #[cfg(not(has_julia_ffi))]
    {
        tracing::warn!("[Julia FFI] Not compiled with has_julia_ffi feature");
        Ok(())
    }
}

/// Shutdown Julia Runtime
pub fn shutdown() -> Result<()> {
    #[cfg(has_julia_ffi)]
    {
        unsafe {
            if julia_shutdown() == 0 {
                tracing::info!("[Julia FFI] ✅ Julia runtime shut down");
                JULIA_AVAILABLE.store(false, Ordering::SeqCst);
                Ok(())
            } else {
                Err(FfiError::CallFailed("julia_shutdown failed".into()).into())
            }
        }
    }

    #[cfg(not(has_julia_ffi))]
    {
        Ok(())
    }
}

/// REAL: Optimize chaotic system weights via Julia FFI
/// Calls brain/julia/julia_math.jl::julia_optimize_weights_ffi()
/// 
/// This function:
/// 1. Takes initial weights [0.33, 0.33, 0.34]
/// 2. Passes to Julia via FFI
/// 3. Julia runs Nelder-Mead optimization
/// 4. Returns optimized weights [0.41, 0.29, 0.30] (example)
pub fn optimize_chaotic_system(params: &[f64]) -> Result<Vec<f64>> {
    #[cfg(has_julia_ffi)]
    {
        if !JULIA_AVAILABLE.load(Ordering::SeqCst) {
            return Err(FfiError::NotInitialized("Julia FFI not initialized".into()).into());
        }

        if params.is_empty() {
            return Ok(Vec::new());
        }

        tracing::debug!(
            "[Julia FFI] Calling julia_optimize_weights_ffi with {} params",
            params.len()
        );

        let mut result = vec![0.0f64; params.len()];

        unsafe {
            let ret = julia_optimize_weights_ffi(
                params.as_ptr(),
                params.len() as i32,
                result.as_mut_ptr(),
            );

            if ret == 0 {
                tracing::info!(
                    "[Julia FFI] ✅ Optimization succeeded:\n  Input:  {:?}\n  Output: {:?}",
                    &params[0..params.len().min(3)],
                    &result[0..result.len().min(3)]
                );
                Ok(result)
            } else {
                tracing::error!(
                    "[Julia FFI] ❌ julia_optimize_weights_ffi returned error code: {}",
                    ret
                );
                Err(FfiError::CallFailed(format!("Optimization failed with code: {}", ret))
                    .into())
            }
        }
    }

    #[cfg(not(has_julia_ffi))]
    {
        tracing::warn!("[Julia Fallback] Using pure Rust fallback (Julia FFI not compiled)");
        // Return input unchanged as fallback
        Ok(params.to_vec())
    }
}

/// REAL: Analyze system dynamics via Julia Chaos Analysis FFI
/// Calls brain/julia/julia_math.jl::julia_chaos_analysis_ffi()
/// Returns Lyapunov exponent (>0 indicates chaotic behavior)
pub fn analyze_dynamics(time_series: &[f64]) -> Result<f64> {
    #[cfg(has_julia_ffi)]
    {
        if !JULIA_AVAILABLE.load(Ordering::SeqCst) {
            return Err(FfiError::NotInitialized("Julia FFI not initialized".into()).into());
        }

        if time_series.is_empty() {
            return Ok(0.0);
        }

        tracing::debug!(
            "[Julia FFI] Calling julia_chaos_analysis_ffi with {} points",
            time_series.len()
        );

        unsafe {
            let lyapunov = julia_chaos_analysis_ffi(time_series.as_ptr(), time_series.len() as i32);

            if !lyapunov.is_nan() {
                tracing::info!(
                    "[Julia FFI] ✅ Chaos analysis succeeded:\n  Lyapunov exponent: {:.6}\n  Is chaotic: {}",
                    lyapunov,
                    lyapunov > 0.0
                );
                Ok(lyapunov)
            } else {
                tracing::error!("[Julia FFI] ❌ julia_chaos_analysis_ffi returned NaN");
                Err(FfiError::CallFailed("Chaos analysis returned NaN".into()).into())
            }
        }
    }

    #[cfg(not(has_julia_ffi))]
    {
        tracing::warn!("[Julia Fallback] Using pure Rust fallback (Julia FFI not compiled)");
        if time_series.is_empty() {
            return Ok(0.0);
        }
        let mean = time_series.iter().sum::<f64>() / time_series.len() as f64;
        let var: f64 = time_series
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / time_series.len() as f64;
        Ok((var.log2() / time_series.len() as f64).clamp(-1.0, 1.0))
    }
}

/// REAL: Get search strategy decision via Julia Decision FFI
/// Calls brain/julia/julia_math.jl::julia_get_decision_ffi()
/// Returns: "HYBRID_FUSION", "VECTOR_QDRANT", "TEXT_TANTIVY", or "HYBRID_BALANCED"
pub fn get_search_decision(entropy: f64, chaos: f64, stability: f64) -> Result<String> {
    #[cfg(has_julia_ffi)]
    {
        if !JULIA_AVAILABLE.load(Ordering::SeqCst) {
            return Err(FfiError::NotInitialized("Julia FFI not initialized".into()).into());
        }

        let mut result_buf = vec![0u8; 512];

        tracing::debug!(
            "[Julia FFI] Calling julia_get_decision_ffi(entropy={:.3}, chaos={:.3}, stability={:.3})",
            entropy,
            chaos,
            stability
        );

        unsafe {
            let len = julia_get_decision_ffi(
                entropy,
                chaos,
                stability,
                result_buf.as_mut_ptr(),
                512,
            );

            if len > 0 && len < 512 {
                let decision_bytes = &result_buf[..len as usize];
                let decision_str = String::from_utf8_lossy(decision_bytes).to_string();

                tracing::info!(
                    "[Julia FFI] ✅ Decision made:\n  Strategy: {}\n  Metrics: entropy={:.3}, chaos={:.3}, stability={:.3}",
                    decision_str,
                    entropy,
                    chaos,
                    stability
                );

                Ok(decision_str)
            } else {
                tracing::error!(
                    "[Julia FFI] ❌ julia_get_decision_ffi returned invalid length: {}",
                    len
                );
                Err(FfiError::CallFailed("Invalid decision buffer length".into()).into())
            }
        }
    }

    #[cfg(not(has_julia_ffi))]
    {
        tracing::warn!("[Julia Fallback] Using pure Rust fallback (Julia FFI not compiled)");
        // Same logic as Julia
        if entropy > 2.5 {
            Ok("HYBRID_FUSION".to_string())
        } else if chaos > 0.4 {
            Ok("VECTOR_QDRANT".to_string())
        } else if stability > 0.8 {
            Ok("TEXT_TANTIVY".to_string())
        } else {
            Ok("HYBRID_BALANCED".to_string())
        }
    }
}

/// Get status of Julia FFI
pub fn is_available() -> bool {
    JULIA_AVAILABLE.load(Ordering::SeqCst)
}
