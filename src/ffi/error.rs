//! src/ffi/error.rs - FFI Error Handling with Recovery Strategies
//!
//! Comprehensive error handling for all FFI layers:
//! - Zig shared memory
//! - Julia mathematics
//! - JAX/Python embeddings
//! - Mojo SIMD kernels
//! - Pony actors
//!
//! Each error includes recovery strategy for graceful degradation

use thiserror::Error;

pub type Result<T> = std::result::Result<T, FfiError>;

/// Recovery strategies for FFI failures (no panic)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Use pure Rust buffer implementation
    FallbackToRustBuffer,
    /// Use mathematical fallback (no integration)
    UseMathFallback,
    /// Use deterministic embedding (reproducible but not semantic)
    UseDeterministicEmbedding,
    /// Use Rust SIMD via std intrinsics
    UseRustSIMD,
    /// Use Tokio actor model instead of Pony
    UseTokioActors,
    /// Graceful shutdown
    Shutdown,
    /// Retry after delay
    RetryWithBackoff,
}

#[derive(Error, Debug)]
pub enum FfiError {
    // ============ ZIG FFI ERRORS ============
    #[error("Zig FFI: shared memory buffer failed - {reason}")]
    ZigMemoryError { reason: String },

    #[error("Zig FFI: buffer allocation failed ({capacity} bytes) - {reason}")]
    ZigAllocationFailed { capacity: usize, reason: String },

    #[error("Zig FFI: not initialized")]
    ZigNotInitialized,

    // ============ JULIA FFI ERRORS ============
    #[error("Julia FFI: mathematician unavailable - {reason}")]
    JuliaUnavailable { reason: String },

    #[error("Julia FFI: chaos analysis failed - {reason}")]
    JuliaChaosAnalysisFailed { reason: String },

    #[error("Julia FFI: optimization failed - {reason}")]
    JuliaOptimizationFailed { reason: String },

    // ============ JAX/PYTHON FFI ERRORS ============
    #[error("JAX FFI: Python runtime not found")]
    JaxPythonNotFound,

    #[error("JAX FFI: embedding generation failed - {reason}")]
    JaxEmbeddingFailed { reason: String },

    #[error("JAX FFI: model loading failed - {model}")]
    JaxModelLoad { model: String },

    // ============ MOJO FFI ERRORS ============
    #[error("Mojo FFI: compiler not found")]
    MojoCompilerNotFound,

    #[error("Mojo FFI: kernel execution failed - {kernel}")]
    MojoKernelFailed { kernel: String },

    // ============ PONY FFI ERRORS ============
    #[error("Pony FFI: actor initialization failed - {reason}")]
    PonyActorFailed { reason: String },

    #[error("Pony FFI: message passing failed - {reason}")]
    PonyMessageFailed { reason: String },

    // ============ GENERIC FFI ERRORS ============
    #[error("FFI initialization failed: {0}")]
    InitFailed(String),

    #[error("FFI call failed: {0}")]
    CallFailed(String),

    #[error("Dimension mismatch: {0}")]
    DimensionMismatch(String),

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("FFI not available - using fallback")]
    FallbackActive,
}

impl FfiError {
    /// Determines recovery strategy for each error type
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            // Zig errors → fallback to Rust buffer
            FfiError::ZigMemoryError { .. }
            | FfiError::ZigAllocationFailed { .. }
            | FfiError::ZigNotInitialized => RecoveryStrategy::FallbackToRustBuffer,

            // Julia errors → fallback to Rust math
            FfiError::JuliaUnavailable { .. }
            | FfiError::JuliaChaosAnalysisFailed { .. }
            | FfiError::JuliaOptimizationFailed { .. } => RecoveryStrategy::UseMathFallback,

            // JAX errors → deterministic embedding
            FfiError::JaxPythonNotFound
            | FfiError::JaxEmbeddingFailed { .. }
            | FfiError::JaxModelLoad { .. } => RecoveryStrategy::UseDeterministicEmbedding,

            // Mojo errors → Rust SIMD
            FfiError::MojoCompilerNotFound
            | FfiError::MojoKernelFailed { .. } => RecoveryStrategy::UseRustSIMD,

            // Pony errors → Tokio actors
            FfiError::PonyActorFailed { .. }
            | FfiError::PonyMessageFailed { .. } => RecoveryStrategy::UseTokioActors,

            // Generic errors
            FfiError::InitFailed(_) => RecoveryStrategy::Shutdown,
            FfiError::CallFailed(_) => RecoveryStrategy::RetryWithBackoff,
            FfiError::FallbackActive => RecoveryStrategy::UseMathFallback,
            _ => RecoveryStrategy::RetryWithBackoff,
        }
    }

    /// Returns whether this error is fatal (requires shutdown)
    pub fn is_fatal(&self) -> bool {
        matches!(self.recovery_strategy(), RecoveryStrategy::Shutdown)
    }

    /// Returns whether we can retry this error
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.recovery_strategy(),
            RecoveryStrategy::RetryWithBackoff | RecoveryStrategy::FallbackToRustBuffer
        )
    }

    /// Log error with recovery strategy
    pub fn log_with_recovery(&self) {
        let strategy = self.recovery_strategy();
        tracing::warn!(
            "FFI error: {} | Recovery: {:?}",
            self,
            strategy
        );
    }
}
