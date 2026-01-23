#!/bin/bash
# build.sh - Real FFI compilation script for MEMORY_P v2.0

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🔧 MEMORY_P FFI Real Build Script${NC}"
echo ""

# Create lib directory if it doesn't exist
mkdir -p lib

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 1. Build Zig FFI Bridge (Real)
if command_exists zig; then
    echo -e "${GREEN}🌉 Building Zig FFI bridge...${NC}"
    zig build-lib src/ffi_bridge.zig \
        -dynamic \
        -O ReleaseFast \
        -femit-bin=lib/libzig_bridge.so
    echo -e "${GREEN}✅ Zig bridge compiled: lib/libzig_bridge.so${NC}"
else
    echo -e "${YELLOW}⚠️  Zig not found, skipping${NC}"
fi

# 2. Build Julia FFI (Real with PackageCompiler)
if command_exists julia; then
    echo -e "${GREEN}🧮 Building Julia FFI with PackageCompiler...${NC}"
    
    # Install required packages
    julia -e '
    using Pkg
    
    # Add packages if not present
    packages = ["Optim", "LinearAlgebra", "Statistics"]
    for pkg in packages
        try
            eval(Meta.parse("using $pkg"))
            println("✓ $pkg already installed")
        catch
            println("Installing $pkg...")
            Pkg.add(pkg)
        end
    end
    
    # Try to add PackageCompiler
    try
        using PackageCompiler
        println("✓ PackageCompiler ready")
    catch
        println("Installing PackageCompiler...")
        Pkg.add("PackageCompiler")
    end
    '
    
    # Create precompile script
    cat > /tmp/julia_precompile.jl << 'EOJULIA'
include("src/julia_math.jl")
using .MemoryPMath

# Precompile critical functions
MemoryPMath.optimize_weights([0.33, 0.33, 0.34])
MemoryPMath.chaos_analysis(rand(100))
EOJULIA
    
    # Build shared library
    julia -e '
    using PackageCompiler
    
    create_library(
        "src/julia_math.jl",
        "lib/libjulia_ffi";
        lib_name="julia_ffi",
        precompile_execution_file="/tmp/julia_precompile.jl",
        incremental=false,
        filter_stdlibs=true
    )
    '
    
    echo -e "${GREEN}✅ Julia FFI compiled: lib/libjulia_ffi.*${NC}"
else
    echo -e "${YELLOW}⚠️  Julia not found, skipping${NC}"
fi

# 3. Setup JAX with CUDA (Python)
if command_exists python3; then
    echo -e "${GREEN}🤖 Setting up JAX with CUDA...${NC}"
    
    # Check if we're in conda environment
    if [ -n "$CONDA_DEFAULT_ENV" ]; then
        echo "Using conda environment: $CONDA_DEFAULT_ENV"
    fi
    
    # Install/upgrade JAX with CUDA
    pip install --upgrade "jax[cuda12]==0.4.28" "jaxlib[cuda12]==0.4.28" sentence-transformers
    
    # Test JAX GPU
    python3 << 'EOPYTHON'
import sys
try:
    import jax
    print(f"✓ JAX version: {jax.__version__}")
    print(f"✓ Devices: {jax.devices()}")
    
    # Test GPU availability
    if any(d.platform == 'gpu' for d in jax.devices()):
        print("✓ GPU available!")
    else:
        print("⚠️  No GPU found, using CPU")
    
    # Test sentence transformers
    from sentence_transformers import SentenceTransformer
    print("✓ sentence-transformers ready")
    
    sys.exit(0)
except Exception as e:
    print(f"✗ Error: {e}")
    sys.exit(1)
EOPYTHON
    
    if [ $? -eq 0 ]; then
        # Copy JAX inference to lib for easy access
        cp src/jax_inference.py lib/
        echo -e "${GREEN}✅ JAX with CUDA configured${NC}"
    else
        echo -e "${RED}✗ JAX setup failed${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Python not found, skipping${NC}"
fi

# 4. Build Mojo SIMD kernels (if available)
if command_exists mojo; then
    echo -e "${GREEN}⚡ Building Mojo SIMD kernels...${NC}"
    mojo build src/kernels.mojo \
        -o lib/libmojo_kernels.so \
        --release
    echo -e "${GREEN}✅ Mojo kernels compiled${NC}"
else
    echo -e "${YELLOW}⚠️  Mojo not found, skipping${NC}"
fi

# 5. Build Pony actor system (if available)
if command_exists ponyc; then
    echo -e "${GREEN}🎭 Building Pony actor system...${NC}"
    ponyc src/search_actor.pony \
        -o build \
        --pic \
        --library
    
    if [ -f build/libsearch_actor.so ]; then
        mv build/libsearch_actor.so lib/libpony_actors.so
        echo -e "${GREEN}✅ Pony actors compiled${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Pony not found, skipping${NC}"
fi

echo ""
echo -e "${GREEN}📊 Build Summary:${NC}"
ls -lh lib/ 2>/dev/null || echo "No libraries built"

echo ""
echo -e "${GREEN}✅ FFI build complete!${NC}"
echo ""
echo "To use FFI in Rust, build with:"
echo "  cargo build --release --features ffi-all"
