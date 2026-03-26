// src/ffi/mod.rs - FFI Module Index
//
// REAL FFI SYSTEM FOR MEMORY_P v3.0
// - Native bindings when compiled libraries exist in FFI/lib
// - Deterministic Rust fallbacks when a runtime is not linkable on this platform

pub mod bridge;
pub mod error;
pub mod jax;
pub mod julia;
pub mod mojo;
pub mod pony;
pub mod zig;

#[allow(ambiguous_glob_reexports)]
pub use jax::*;
#[allow(ambiguous_glob_reexports)]
pub use julia::*;
#[allow(ambiguous_glob_reexports)]
pub use mojo::*;
#[allow(ambiguous_glob_reexports)]
pub use pony::*;
#[allow(ambiguous_glob_reexports)]
pub use zig::*;

pub async fn initialize_all() -> Result<(), error::FfiError> {
    #[cfg(has_zig_ffi)]
    let _ = zig::init();
    let _ = julia::init();
    #[cfg(has_mojo_ffi)]
    let _ = mojo::init();
    let _ = jax::init();
    #[cfg(has_pony_ffi)]
    let _ = pony::init();
    Ok(())
}

#[cfg(test)]
mod benchmarks;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

static FFI_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FfiStatus {
    pub zig: bool,
    pub julia: bool,
    pub jax: bool,
    pub mojo: bool,
    pub pony: bool,
}

impl FfiStatus {
    pub fn available_count(&self) -> usize {
        [self.zig, self.julia, self.jax, self.mojo, self.pony]
            .into_iter()
            .filter(|available| *available)
            .count()
    }

    pub fn has_any(&self) -> bool {
        self.available_count() > 0
    }
}

pub async fn init() -> crate::error::Result<()> {
    tracing::info!("Initializing FFI subsystems...");

    // Sequential initialization instead of join! macro for type clarity
    let mut init_status = Vec::new();

    #[cfg(has_zig_ffi)]
    match self::zig::init() {
        Ok(_) => init_status.push(Ok("Zig")),
        Err(e) => init_status.push(Err(format!("Zig: {}", e))),
    }

    match self::julia::init().await {
        Ok(_) => init_status.push(Ok("Julia")),
        Err(e) => init_status.push(Err(format!("Julia: {}", e))),
    }

    match self::jax::init() {
        Ok(_) => init_status.push(Ok("JAX")),
        Err(e) => init_status.push(Err(format!("JAX: {}", e))),
    }

    #[cfg(has_mojo_ffi)]
    match self::mojo::init() {
        Ok(_) => init_status.push(Ok("Mojo")),
        Err(e) => init_status.push(Err(format!("Mojo: {}", e))),
    }

    #[cfg(has_pony_ffi)]
    match self::pony::init() {
        Ok(_) => init_status.push(Ok("Pony")),
        Err(e) => init_status.push(Err(format!("Pony: {}", e))),
    }

    for result in init_status {
        match result {
            Ok(name) => tracing::info!("  {} FFI: ready", name),
            Err(msg) => tracing::warn!("  {}", msg),
        }
    }

    let status = detect_status();
    if !status.has_any() {
        FFI_INITIALIZED.store(false, Ordering::SeqCst);
        return Err(crate::error::MemoryPError::Ffi(
			self::error::FfiError::InitFailed(
				"No real FFI backend is active. Build at least one backend in FFI/lib before starting the runtime.".into(),
			),
		));
    }

    FFI_INITIALIZED.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn shutdown() {
    if FFI_INITIALIZED.swap(false, Ordering::SeqCst) {
        self::zig::shutdown();
        self::julia::shutdown();
        self::jax::shutdown();
        self::mojo::shutdown();
        self::pony::shutdown();
    }
}

pub fn is_initialized() -> bool {
    FFI_INITIALIZED.load(Ordering::SeqCst)
}

pub fn detect_status() -> FfiStatus {
    FfiStatus {
        zig: self::zig::is_available(),
        julia: self::julia::is_available(),
        jax: self::jax::is_available(),
        mojo: self::mojo::is_available(),
        pony: self::pony::is_available(),
    }
}

pub use bridge::{dispatch_batch, dispatch_fast, Language};
pub use jax::{EmbeddingConfig, EmbeddingGenerator, EmbeddingModel};
pub use mojo::{cosine_similarity, dot_product};
pub use pony::distributed_search;
pub use zig::ZigBridge;
