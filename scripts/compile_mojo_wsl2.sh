#!/bin/bash
# Mojo Kernel Compilation for MEMORY_P - WSL2 Script
# Run this in WSL2 Ubuntu-24.04 to compile Mojo kernels

set -e

REPO_ROOT="/mnt/d/REPOSITORIOS/memory_p"
BRAIN_DIR="$REPO_ROOT/brain/mojo"
OUTPUT_DIR="$REPO_ROOT/FFI/lib"
PIXI_BIN="$HOME/.pixi/bin/pixi"
MOJO_ENV_DIR="$HOME/.cache/memory_p_mojo_env"
MOJO_ENV_PREFIX="$MOJO_ENV_DIR/.pixi/envs/default"

echo "🔨 MEMORY_P Mojo Kernel Compilation Script"
echo "=========================================="
echo "Workspace: $REPO_ROOT"
echo ""

# Check if pixi exists and provision a clean max/mojo env if needed
echo "1️⃣  Checking Mojo runtime..."
if [ ! -x "$PIXI_BIN" ]; then
    echo "❌ pixi not found at $PIXI_BIN"
    echo "   Run scripts/install_ffi_toolchains_wsl.sh first"
    exit 1
fi

mkdir -p "$MOJO_ENV_DIR"

cat > "$MOJO_ENV_DIR/pixi.toml" <<'EOF'
[workspace]
channels = ["conda-forge", "https://conda.modular.com/max"]
name = "mojo_env"
platforms = ["linux-64"]
version = "0.1.0"

[dependencies]
max = ">=26.1.0,<27"
EOF

cd "$MOJO_ENV_DIR"
rm -f "$MOJO_ENV_DIR/pixi.lock"

if ! "$PIXI_BIN" install; then
    echo "⚠️  pixi install failed in existing runtime dir, recreating clean environment..."
    cd "$HOME"
    rm -rf "$MOJO_ENV_DIR"
    mkdir -p "$MOJO_ENV_DIR"
    cat > "$MOJO_ENV_DIR/pixi.toml" <<'EOF'
[workspace]
channels = ["conda-forge", "https://conda.modular.com/max"]
name = "mojo_env"
platforms = ["linux-64"]
version = "0.1.0"

[dependencies]
max = ">=26.1.0,<27"
EOF
    cd "$MOJO_ENV_DIR"
    "$PIXI_BIN" install
fi

export CONDA_PREFIX="$MOJO_ENV_PREFIX"
source "$MOJO_ENV_PREFIX/etc/conda/activate.d/10-activate-max.sh"
MOJO_BIN="$MOJO_ENV_PREFIX/bin/mojo"

if [ ! -x "$MOJO_BIN" ]; then
    echo "❌ mojo binary not found after pixi install"
    exit 1
fi

MOJO_VERSION=$($MOJO_BIN --version 2>&1 || echo "unknown")
echo "✅ Mojo found: $MOJO_VERSION"
echo ""

# Check if source files exist
echo "2️⃣  Checking source files..."
if [ ! -f "$BRAIN_DIR/kernels.mojo" ]; then
    echo "❌ kernels.mojo not found at $BRAIN_DIR/kernels.mojo"
    exit 1
fi
echo "✅ kernels.mojo found"

if [ ! -d "$OUTPUT_DIR" ]; then
    mkdir -p "$OUTPUT_DIR"
    echo "✅ Created output directory: $OUTPUT_DIR"
fi

echo ""
echo "3️⃣  Compiling Mojo kernels..."
cd "$BRAIN_DIR"

if "$MOJO_BIN" build kernels.mojo --emit shared-lib -o "$OUTPUT_DIR/libmojo_kernels.so" 2>&1; then
    echo "✅ Compilation successful!"
    ls -lh "$OUTPUT_DIR/libmojo_kernels.so"
    echo ""
    echo "4️⃣  Next steps:"
    echo "  - Add has_mojo_ffi feature to Cargo.toml (if not present)"
    echo "  - Run: cargo build --lib --release --features has_mojo_ffi"
    echo ""
else
    echo "❌ Mojo compilation failed"
    exit 1
fi
