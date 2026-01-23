//! ffi/error.rs - Error types for FFI operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FfiError {
    #[error("FFI not available: {0}")]
    NotAvailable(String),

    #[error("Null pointer in FFI call")]
    NullPointer,

    #[error("Memory safety violation: {0}")]
    MemorySafety(String),

    #[error("Julia exception: {0}")]
    JuliaException(String),

    #[error("JAX error: {0}")]
    JaxError(String),

    #[error("Mojo error: {0}")]
    MojoError(String),

    #[error("Pony error: {0}")]
    PonyError(String),

    #[error("Zig bridge error: {0}")]
    ZigError(String),

    #[error("FFI call failed: {0}")]
    CallFailed(String),
}

pub type Result<T> = std::result::Result<T, FfiError>;
