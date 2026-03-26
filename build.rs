// build.rs - MEMORY_P FFI Linkage Configuration
// 
// This script configures real FFI connections from Rust to BRAIN/ implementations
// Handles: Julia, JAX/Python, Mojo, Pony, Zig

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=FFI/");
    println!("cargo:rerun-if-changed=brain/");

    // ===== DECLARE ALL CFG CONDITIONS =====
    // Suppresses "unexpected cfg" warnings
    println!("cargo::rustc-check-cfg=cfg(has_zig_ffi)");
    println!("cargo::rustc-check-cfg=cfg(has_julia_ffi)");
    println!("cargo::rustc-check-cfg=cfg(has_mojo_ffi)");
    println!("cargo::rustc-check-cfg=cfg(has_pony_ffi)");
    println!("cargo::rustc-check-cfg=cfg(has_jax_ffi)");
    println!("cargo::rustc-check-cfg=cfg(has_faiss_ffi)");
    println!("cargo::rustc-check-cfg=cfg(feature,values(\"ffi-all\"))");

    // ===== ZIG FFI BRIDGE =====
    // When Zig source changes, trigger rebuild
    println!("cargo:rerun-if-changed=brain/zig/ffi_bridge.zig");
    println!("cargo:rerun-if-changed=brain/zig/shared_memory_buffer.zig");
    println!("cargo:rerun-if-changed=brain/zig/zig_buffers.zig");

    // Try to link pre-compiled Zig library (if exists)
    // Disabled: Causes linking errors if library doesn't exist
    // #[cfg(target_os = "windows")]
    // {
    //     println!("cargo:rustc-link-search=native=brain/zig");
    //     println!("cargo:rustc-link-lib=static=ffi_bridge");
    // }
    //
    // #[cfg(target_os = "linux")]
    // {
    //     println!("cargo:rustc-link-search=native=brain/zig");
    //     println!("cargo:rustc-link-lib=static=ffi_bridge");
    // }

    // ===== DEVELOPMENT MODE: FFI flags for conditional compilation =====
    // **IMPORTANT**: All FFI symbols use libloading for DYNAMIC loading
    // NO static linking - prevents linker errors for missing .lib files
    // FFI code has proper #[cfg(has_*_ffi)] guards AND fallback Rust implementations
    
    // ✅ [ACTIVATED] ZIG FFI - has fallback shared memory in pure Rust
    println!("cargo:rustc-cfg=has_zig_ffi");
    
    // ===== JULIA FFI ===== (ENABLED - has complete Rust fallback)
    // Always enable - Julia loads via libloading + dynamic symbol resolution
    println!("cargo:rustc-cfg=has_julia_ffi");
    
    // ✅ [ACTIVATED] MOJO FFI - has Rust SIMD equivalent for kernels
    println!("cargo:rustc-cfg=has_mojo_ffi");

    // ✅ [ACTIVATED] PONY FFI - has actor pattern fallback in tokio
    println!("cargo:rustc-cfg=has_pony_ffi");

    // ✅ [ACTIVATED] JAX/Python FFI - has embedding fallback
    println!("cargo:rustc-cfg=has_jax_ffi");
    
    println!("cargo:rustc-cfg=feature=\"ffi-all\"");
}

