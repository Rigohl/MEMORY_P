#!/bin/bash
# Install FFI toolchains for MEMORY_P in WSL2 Ubuntu24D
# Strategy: User-level installs where possible, minimal sudo for build deps only

set -e
set -o pipefail

ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64) ZIG_ARCH="x86_64-linux" ;;
    aarch64|arm64) ZIG_ARCH="aarch64-linux" ;;
    *)
        echo "❌ Unsupported architecture for automated Zig install: $ARCH"
        exit 1
        ;;
esac

JULIA_STABLE_VERSION="1.12.5"
PYTHON_STABLE_SERIES="3.13"

get_latest_zig_version() {
    python3 - <<'PY'
import json, urllib.request
data = json.load(urllib.request.urlopen("https://ziglang.org/download/index.json"))
for key, value in data.items():
    if key == "master":
        continue
    if isinstance(value, dict) and value.get("x86_64-linux"):
        print(value["version"])
        break
PY
}

echo "🔧 Installing MEMORY_P FFI Toolchains (Ubuntu24D)..."
echo "====================================================="

have_sudo_nopass() {
    command -v sudo >/dev/null 2>&1 && sudo -n true >/dev/null 2>&1
}

report_missing_deps() {
    local missing=0
    for cmd in gcc cmake pkg-config git curl wget llvm-config clang python3; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            echo "⚠️  Missing dependency: $cmd"
            missing=1
        fi
    done
    return $missing
}

# Check/install build deps without blocking on sudo password
if report_missing_deps; then
    echo "✅ Build tools already present"
else
    if have_sudo_nopass; then
        echo "📦 Installing missing build dependencies with passwordless sudo..."
        sudo apt-get update -qq && \
        sudo apt-get install -y -qq \
            build-essential cmake pkg-config libssl-dev \
            git curl wget unzip ca-certificates \
            llvm clang lld libllvm-dev \
            python3-dev python3-pip python3-venv \
            libpcre2-dev zlib1g-dev libncurses-dev
        echo "✅ Build deps installed"
    else
        echo "⚠️  No sudo passwordless access; continuing with existing user-level toolchain only"
    fi
fi

# Source cargo env if exists
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

mkdir -p "$HOME/.local/bin"
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

# 0. Install pixi if missing (required for Modular/Max/Mojo)
if ! command -v pixi &> /dev/null; then
    echo ""
    echo "📦 Installing pixi..."
    curl -fsSL https://pixi.sh/install.sh | bash
    export PATH="$HOME/.pixi/bin:$PATH"
    if ! command -v pixi &> /dev/null; then
        echo "❌ pixi installation failed"
        exit 1
    fi
    echo "✅ pixi installed: $(pixi --version)"
else
    echo "✅ pixi already installed: $(pixi --version)"
fi

# 1. Install Zig
if ! command -v zig &> /dev/null; then
    echo ""
    echo "⚡ Installing Zig..."
    ZIG_VERSION="$(get_latest_zig_version)"
    cd /tmp
    wget -q "https://ziglang.org/download/${ZIG_VERSION}/zig-${ZIG_ARCH}-${ZIG_VERSION}.tar.xz"
    tar xf "zig-${ZIG_ARCH}-${ZIG_VERSION}.tar.xz"
    mkdir -p "$HOME/.local/bin"
    cp "zig-${ZIG_ARCH}-${ZIG_VERSION}/zig" "$HOME/.local/bin/"
    rm -rf "zig-${ZIG_ARCH}-${ZIG_VERSION}"
    rm "zig-${ZIG_ARCH}-${ZIG_VERSION}.tar.xz"
    export PATH="$HOME/.local/bin:$PATH"
    echo "✅ Zig installed: $(zig version)"
else
    echo "🔄 Zig already installed ($(zig version)) - checking latest stable..."
    ZIG_VERSION="$(get_latest_zig_version)"
    if [ "$(zig version)" != "$ZIG_VERSION" ]; then
        echo "⬆️ Updating Zig to $ZIG_VERSION"
        cd /tmp
        wget -q "https://ziglang.org/download/${ZIG_VERSION}/zig-${ZIG_ARCH}-${ZIG_VERSION}.tar.xz"
        tar xf "zig-${ZIG_ARCH}-${ZIG_VERSION}.tar.xz"
        mkdir -p "$HOME/.local/bin"
        cp "zig-${ZIG_ARCH}-${ZIG_VERSION}/zig" "$HOME/.local/bin/"
        rm -rf "zig-${ZIG_ARCH}-${ZIG_VERSION}"
        rm "zig-${ZIG_ARCH}-${ZIG_VERSION}.tar.xz"
        export PATH="$HOME/.local/bin:$PATH"
    fi
    echo "✅ Zig ready: $(zig version)"
fi

echo ""
echo "📊 Installing Julia ${JULIA_STABLE_VERSION} (official stable binary, no sudo)..."
cd /tmp
JULIA_TARBALL="julia-${JULIA_STABLE_VERSION}-linux-x86_64.tar.gz"
JULIA_URL="https://julialang-s3.julialang.org/bin/linux/x64/1.12/${JULIA_TARBALL}"
rm -rf "$HOME/.local/julia-${JULIA_STABLE_VERSION}" "$HOME/.local/julia"
wget -q "$JULIA_URL"
tar xzf "$JULIA_TARBALL"
mv "julia-${JULIA_STABLE_VERSION}" "$HOME/.local/julia-${JULIA_STABLE_VERSION}"
ln -sfn "$HOME/.local/julia-${JULIA_STABLE_VERSION}" "$HOME/.local/julia"
ln -sf "$HOME/.local/julia/bin/julia" "$HOME/.local/bin/julia"
rm -f "$JULIA_TARBALL"
export PATH="$HOME/.local/bin:$PATH"
echo "✅ Julia ready: $(julia --version)"

# 3. Install Pony (build from source, no sudo for compilation)
if ! command -v ponyc &> /dev/null; then
    echo ""
    echo "🎭 Building Pony compiler..."
    cd /tmp
    rm -rf ponyc_build
    if git clone --depth=1 https://github.com/ponylang/ponyc.git ponyc_build -q 2>/dev/null; then
        cd ponyc_build
        if make config=release -j$(nproc); then
            mkdir -p "$HOME/.local/bin"
            cp build/release/ponyc "$HOME/.local/bin/"
            export PATH="$HOME/.local/bin:$PATH"
            echo "✅ Pony installed: $(ponyc --version)"
        else
            echo "⚠️  Pony build failed - leaving existing environment unchanged"
        fi
        cd ..
        rm -rf ponyc_build
    else
        echo "⚠️  Could not clone Pony source - skipping Pony update"
    fi
else
    echo "✅ Pony already installed: $(ponyc --version)"
fi

# 4. Setup modern Python + GPU JAX environment (no sudo)
echo ""
echo "🐍 Setting up modern Python ${PYTHON_STABLE_SERIES} + JAX GPU environment..."
PY_VENV_DIR="$HOME/.local/share/memory_p_py"
if ! command -v uv >/dev/null 2>&1; then
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.local/bin:$PATH"
fi
uv python install "$PYTHON_STABLE_SERIES"
rm -rf "$PY_VENV_DIR"
uv venv --python "$PYTHON_STABLE_SERIES" "$PY_VENV_DIR"
if [ -x "$PY_VENV_DIR/bin/python" ]; then
    "$PY_VENV_DIR/bin/python" -m pip install -q --upgrade pip setuptools wheel 2>/dev/null || true
    "$PY_VENV_DIR/bin/python" -m pip install -q --upgrade "jax[cuda13]" sentence-transformers 2>/dev/null || \
        "$PY_VENV_DIR/bin/python" -m pip install -q --upgrade "jax[cuda12]" sentence-transformers 2>/dev/null || \
        echo "⚠️  GPU JAX installation had issues inside virtualenv"
    export VIRTUAL_ENV="$PY_VENV_DIR"
    export PATH="$PY_VENV_DIR/bin:$PATH"
else
    echo "⚠️  Could not create Python virtualenv; Python ML stack not updated"
fi

# 5. Update PATH in bashrc for persistence
echo ""
echo "🔧 Configuring environment persistence..."
BASHRC_APPEND='
# ===== MEMORY_P FFI TOOLCHAINS (added by install script) =====
export PATH="$HOME/.local/bin:$PATH"
export PATH="$HOME/.cargo/bin:$PATH"
export VIRTUAL_ENV="$HOME/.local/share/memory_p_py"
[ -d "$VIRTUAL_ENV/bin" ] && export PATH="$VIRTUAL_ENV/bin:$PATH"
[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
# ============================================================
'

if ! grep -q "MEMORY_P FFI TOOLCHAINS" "$HOME/.bashrc" 2>/dev/null; then
    echo "$BASHRC_APPEND" >> "$HOME/.bashrc"
    echo "✅ Environment variables added to ~/.bashrc"
else
    echo "✅ Environment already configured"
fi

# Summary
echo ""
echo "====================================================="
echo "✅ FFI Toolchain Installation Summary:"
echo ""
echo "Installed/verified:"
zig version 2>/dev/null && echo "  ✅ Zig" || echo "  ❌ Zig unavailable"
julia --version 2>/dev/null && echo "  ✅ Julia" || echo "  ❌ Julia unavailable"
ponyc --version 2>/dev/null && echo "  ✅ Pony" || echo "  ⚠️  Pony unavailable"
if [ -x "$PY_VENV_DIR/bin/python" ]; then
    "$PY_VENV_DIR/bin/python" -c "import jax; print('  ✅ JAX')" 2>/dev/null || echo "  ⚠️  JAX unavailable"
else
    echo "  ⚠️  JAX unavailable"
fi

if command -v cargo &> /dev/null; then
    echo "  ✅ Cargo/Rust"
else
    echo "  ⚠️  Rust needs installation - run install_rust_wsl.sh first"
fi

echo ""
echo "📝 Next steps:"
echo "  1. If Rust not installed: bash scripts/install_rust_wsl.sh"
echo "  2. Reload shell: exec bash"
echo "  3. Build from WSL using ext4 target dir: CARGO_TARGET_DIR=\$HOME/.cache/memory_p-target cargo build --release"
echo ""
