#!/bin/bash
# Install complete FFI toolchain for MEMORY_P in WSL2

set -e

echo "🔧 Installing MEMORY_P FFI Toolchain in WSL2..."
echo "=================================================="

# Update package manager
echo "📦 Updating apt..."
sudo apt-get update -qq
sudo apt-get upgrade -y -qq

# Install build essentials
echo "🔨 Installing build tools..."
sudo apt-get install -y -qq \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    git \
    curl \
    wget \
    unzip \
    ca-certificates

# Install Rust (required for memory_p)
if ! command -v cargo &> /dev/null; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y -q
    source $HOME/.cargo/env
else
    echo "✅ Rust already installed"
    rustc --version
fi

# Install Zig
if ! command -v zig &> /dev/null; then
    echo "⚡ Installing Zig..."
    ZIG_VERSION="0.12.0"
    cd /tmp
    wget -q "https://ziglang.org/download/${ZIG_VERSION}/zig-linux-x86_64-${ZIG_VERSION}.tar.xz"
    tar xf "zig-linux-x86_64-${ZIG_VERSION}.tar.xz"
    sudo mv "zig-linux-x86_64-${ZIG_VERSION}" /opt/zig
    sudo ln -sf /opt/zig/zig /usr/local/bin/zig
    rm "zig-linux-x86_64-${ZIG_VERSION}.tar.xz"
    export PATH="/opt/zig:$PATH"
else
    echo "✅ Zig already installed"
    zig version
fi

# Install Julia
if ! command -v julia &> /dev/null; then
    echo "📊 Installing Julia..."
    JULIA_VERSION="1.10.0"
    cd /tmp
    wget -q "https://julialang-s3.julialang.org/bin/linux/x64/${JULIA_VERSION:0:3}/julia-${JULIA_VERSION}-linux-x86_64.tar.gz"
    tar xzf "julia-${JULIA_VERSION}-linux-x86_64.tar.gz"
    sudo mv "julia-${JULIA_VERSION}" /opt/julia
    sudo ln -sf /opt/julia/bin/julia /usr/local/bin/julia
    rm "julia-${JULIA_VERSION}-linux-x86_64.tar.gz"
    export JULIA_DIR="/opt/julia"
else
    echo "✅ Julia already installed"
    julia --version
fi

# Install Pony compiler
if ! command -v ponyc &> /dev/null; then
    echo "🎭 Installing Pony compiler..."
    cd /tmp
    git clone https://github.com/ponylang/ponyc.git -q
    cd ponyc
    make install -j$(nproc) > /dev/null 2>&1
    export PATH="/tmp/ponyc/build/release:$PATH"
    cd ..
    rm -rf ponyc
else
    echo "✅ Pony already installed"
    ponyc --version
fi

# Install Python ML libraries (for JAX/Mojo)
echo "🐍 Installing Python ML dependencies..."
python3 -m pip install -q --upgrade pip setuptools wheel
python3 -m pip install -q jax jaxlib huggingface-hub transformers

# Set environment variables
echo "🔧 Setting environment variables..."
cat >> ~/.bashrc << 'EOF'
export PATH="/opt/zig:$PATH"
export JULIA_DIR="/opt/julia"
export PYTHON_HOME="/usr"
export RUST_BACKTRACE=1
EOF

echo "=================================================="
echo "✅ Toolchain installation complete!"
echo ""
echo "To use these tools, run: source ~/.bashrc"
echo "Or start a new terminal session."
