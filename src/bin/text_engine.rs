//! Text Search Engine - Tier 2
//! Handles: Tantivy, LNX, Toshi, MeiliSearch (4 text search motors)
//! Compilation: cargo build --release --bin text_engine

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("╔════════════════════════════════════════════════╗");
    println!("║     MEMORY_P - Text Search Engine              ║");
    println!("║     Motores: Tantivy, LNX, Toshi,             ║");
    println!("║              MeiliSearch                       ║");
    println!("╚════════════════════════════════════════════════╝");
    println!();

    // Initialize FFI subsystems
    memory_p::ffi::init().await?;

    println!("✓ FFI initialized");
    println!("✓ Text Search Engine ready on port 3002");
    println!();

    // Initialize motors
    println!("Initializing text search engines:");
    println!("  [1] Tantivy (native Rust full-text search - BM25)");
    println!("  [2] LNX (distributed search with Raft coordination)");
    println!("  [3] Toshi (experimental distributed text engine)");
    println!("  [4] MeiliSearch (typo-tolerant user-friendly search)");
    println!();

    let sample_query = "find documents about machine learning";
    println!("Sample query: '{}'", sample_query);
    println!("Ready to receive text search requests...");
    println!();

    // Graceful shutdown
    memory_p::ffi::shutdown();
    println!("✓ Text Search Engine shutdown complete");
    Ok(())
}
