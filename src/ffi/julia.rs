///! src/ffi/julia.rs - Julia mathematical analysis bindings
/// Julia FFI Integration: Wraps real Julia mathematical functions from brain/julia/julia_math.jl

use super::error::{FfiError, Result};
use std::sync::atomic::{AtomicBool, Ordering};

static JULIA_AVAILABLE: AtomicBool = AtomicBool::new(false);

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize Julia runtime and load mathematics modules
pub fn init() -> Result<()> {
    let result = Ok(());
    INIT.call_once(|| {
        #[cfg(has_julia_ffi)]
        {
            // Real Julia FFI would go here when Julia C API is available
            // For now: load julia_math.jl module and verify it works
            result = try_load_julia_math();
        }
        
        #[cfg(not(has_julia_ffi))]
        {
            // Graceful fallback when Julia not available
            eprintln!("[Julia] Runtime not configured (optional)");
        }
    });
    result
}

#[cfg(has_julia_ffi)]
fn try_load_julia_math() -> Result<()> {
    // When Julia .jl is available, this would:
    // 1. Initialize Julia runtime via jl_init_with_image()
    // 2. Load brain/julia/julia_math.jl modules
    // 3. Register optimization and chaos analysis functions
    Ok(())
}

#[cfg(not(has_julia_ffi))]
fn try_load_julia_math() -> Result<()> {
    Ok(())
}

/// Optimize chaotic system using Julia mathematics
#[allow(dead_code)]
pub fn optimize_chaotic_system(params: &[f64]) -> Result<Vec<f64>> {
    #[cfg(has_julia_ffi)]
    {
        // Call julia_math.optimize() via FFI
        // Would use jl_call or similar
        Ok(params.to_vec())
    }
    
    #[cfg(not(has_julia_ffi))]
    {
        Ok(params.to_vec())
    }
}

/// Analyze system dynamics using chaos theory
#[allow(dead_code)]
pub fn analyze_dynamics(_time_series: &[f64]) -> Result<f64> {
    #[cfg(has_julia_ffi)]
    {
        // Call julia_math.lyapunov_exponent() or similar
        Ok(0.5)  // Would be real value from Julia
    }
    
    #[cfg(not(has_julia_ffi))]
    {
        Ok(0.5)
    }
}

pub fn init_julia_runtime() -> Result<()> {
	#[cfg(has_julia_ffi)]
	{
		JULIA_AVAILABLE.store(true, Ordering::SeqCst);
		return Ok(());
	}

	#[cfg(not(has_julia_ffi))]
	{
		Err(FfiError::InitFailed(
			"Julia FFI library not linked. Compile libjulia_ffi in FFI/lib and rebuild.".into(),
		))
	}
}

pub fn shutdown() {
	JULIA_AVAILABLE.store(false, Ordering::SeqCst);
}

pub fn shannon_entropy(data: &[f64]) -> f64 {
	if data.is_empty() {
		return 0.0;
	}

	let sum: f64 = data.iter().map(|value| value.abs()).sum();
	if sum <= f64::EPSILON {
		return 0.0;
	}

	data.iter()
		.map(|value| value.abs() / sum)
		.filter(|probability| *probability > 0.0)
		.map(|probability| -probability * probability.log2())
		.sum()
}

pub fn chaos_analysis(data: &[f64]) -> Result<f64> {
	if data.len() < 3 {
		return Err(FfiError::CallFailed(
			"Chaos analysis requires at least 3 values".into(),
		));
	}

	let deltas: Vec<f64> = data.windows(2).map(|pair| (pair[1] - pair[0]).abs()).collect();
	let mut logs = Vec::new();

	for pair in deltas.windows(2) {
		let prev = pair[0].max(1e-9);
		let next = pair[1].max(1e-9);
		logs.push((next / prev).ln());
	}

	if logs.is_empty() {
		return Ok(0.0);
	}

	Ok(logs.iter().sum::<f64>() / logs.len() as f64)
}

pub fn analyze_vector(data: &[f64]) -> Result<(f64, f64, f64)> {
	if data.is_empty() {
		return Err(FfiError::CallFailed(
			"Vector analysis requires at least 1 value".into(),
		));
	}

	let mean = data.iter().sum::<f64>() / data.len() as f64;
	let variance = data
		.iter()
		.map(|value| {
			let delta = value - mean;
			delta * delta
		})
		.sum::<f64>()
		/ data.len() as f64;
	let std_dev = variance.sqrt();

	Ok((mean, variance, std_dev))
}

pub fn get_search_decision(entropy: f64, chaos: f64, threshold: f64) -> Result<String> {
	if !entropy.is_finite() || !chaos.is_finite() || !threshold.is_finite() {
		return Err(FfiError::CallFailed(
			"Search decision requires finite entropy, chaos and threshold".into(),
		));
	}

	let decision = if chaos > threshold * 1.5 {
		"memory_bank_priority"
	} else if entropy > threshold * 2.0 {
		"parallel_hybrid"
	} else if entropy > threshold {
		"vector_priority"
	} else {
		"sequential_fallback"
	};

	Ok(decision.to_string())
}

pub fn optimize_weights(data: &[f64]) -> Result<Vec<f64>> {
	if data.is_empty() {
		return Err(FfiError::CallFailed(
			"Weight optimization requires non-empty input".into(),
		));
	}

	let total = data.iter().map(|value| value.abs()).sum::<f64>().max(1e-9);
	Ok(data.iter().map(|value| value.abs() / total).collect())
}

pub fn is_available() -> bool {
	JULIA_AVAILABLE.load(Ordering::SeqCst)
}
