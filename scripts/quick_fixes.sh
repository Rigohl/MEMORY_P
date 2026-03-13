#!/bin/bash
# MEMORY_P Quick Fixes + FFI/BRAIN/SRC Validation
# Quick fix script for common MEMORY_P issues
# Based on CI/CD analysis results

set -e

echo "🔧 Quick Fix Script for MEMORY_P Issues"
echo "======================================="

# Fix 1: Ensure Cargo.toml has proper dependencies
echo -e "\n📦 Checking Cargo.toml dependencies..."
if [[ -f "Cargo.toml" ]]; then
    # Check for common missing dependencies
    if ! grep -q "tokio" Cargo.toml; then
        echo "⚠️  Adding tokio dependency..."
        # This would need manual editing of Cargo.toml
        echo "Please add: tokio = { version = \"1.0\", features = [\"full\"] }"
    fi
    
    if ! grep -q "serde" Cargo.toml; then
        echo "⚠️  Adding serde dependencies..."
        echo "Please add: serde = { version = \"1.0\", features = [\"derive\"] }"
        echo "Please add: serde_json = \"1.0\""
    fi
else
    echo "❌ Cargo.toml not found!"
fi

# Fix 2: Check for missing source files
echo -e "\n📁 Checking source file structure..."
if [[ ! -d "src" ]]; then
    echo "❌ src/ directory missing!"
    mkdir -p src
    echo "✅ Created src/ directory"
fi

if [[ ! -f "src/main.rs" ]] && [[ ! -f "src/lib.rs" ]]; then
    echo "⚠️  No main.rs or lib.rs found - creating basic lib.rs"
    cat > src/lib.rs << 'EOF'
//! # MEMORY_P - Multi-Language Memory System
//!
//! A high-performance, multi-language memory and search system
//! with FFI support for Zig, Julia, Mojo, JAX, and Pony.

pub mod core;
pub mod ffi;
pub mod motores;
pub mod mcp;
pub mod shared_memory;

use std::error::Error;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
EOF
    echo "✅ Created basic lib.rs"
fi

# Fix 3: Check FFI directory structure
echo -e "\n🔗 Checking FFI directory structure..."
if [[ ! -d "FFI" ]]; then
    echo "⚠️  FFI/ directory missing - creating basic structure"
    mkdir -p FFI/src
    mkdir -p FFI/include
    
    # Create basic FFI bridge file
    cat > FFI/src/mod.rs << 'EOF'
//! FFI Bridge Module
//!
//! Provides foreign function interfaces to:
//! - Zig: Shared memory management
//! - Julia: Mathematical computations
//! - Mojo: SIMD kernels
//! - JAX: ML inference
//! - Pony: Actor-based concurrency

pub mod zig_bridge;
pub mod julia_math;
pub mod mojo_kernels;
pub mod jax_inference;
pub mod pony_actors;

use crate::Result;

pub fn initialize_ffi() -> Result<()> {
    // Initialize all FFI bridges
    println!("🔗 Initializing FFI bridges...");
    Ok(())
}
EOF
    echo "✅ Created basic FFI structure"
fi

# Fix 4: Check brain directory
echo -e "\n🧠 Checking brain directory..."
if [[ ! -d "brain" ]]; then
    echo "⚠️  brain/ directory missing - creating basic structure"
    mkdir -p brain/core
    mkdir -p brain/math
    mkdir -p brain/chaos
    
    # Create basic brain README
    cat > brain/README.md << 'EOF'
# MEMORY_P Brain

This directory contains the core intelligence and mathematical processing components.

## Components

- `core/`: Core algorithms and data structures
- `math/`: Mathematical computations and optimizations
- `chaos/`: Chaos theory and dynamic analysis

## Key Features

- Multi-language mathematical processing
- Chaos theory analysis
- Performance optimization algorithms
- Intelligent routing and decision making
EOF
    echo "✅ Created basic brain structure"
fi

# Fix 5: Create basic .gitignore if missing
echo -e "\n📝 Checking .gitignore..."
if [[ ! -f ".gitignore" ]]; then
    echo "⚠️  .gitignore missing - creating basic one"
    cat > .gitignore << 'EOF'
# Rust
target/
Cargo.lock

# IDEs
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log

# Temporary files
*.tmp
*.bak

# FFI builds
FFI/target/
*.so
*.dylib
*.dll

# Python
__pycache__/
*.pyc
*.pyo

# Julia
*.ji
/Manifest.toml

# Node.js (if any)
node_modules/
EOF
    echo "✅ Created .gitignore"
fi

echo -e "\n🎯 Quick fixes applied!"
echo "======================"
echo "Review the changes and run 'cargo check' to verify."
echo "Then run the full analysis again: bash scripts/full_analysis.sh"