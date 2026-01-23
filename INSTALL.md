# INSTALL.md - Installation Guide for MEMORY_P v2.0

Complete installation guide for all MEMORY_P components including multi-language FFI stack.

---

## 📋 Table of Contents

- [Quick Start (Rust Only)](#quick-start-rust-only)
- [Full Stack Installation](#full-stack-installation)
- [Language-Specific Setup](#language-specific-setup)
- [Docker Installation](#docker-installation)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)

---

## Quick Start (Rust Only)

Minimal installation - only Rust core without FFI extensions.

### Prerequisites

- **Rust 1.75+**: https://rustup.rs/
- **Git**: For cloning repository

### Installation

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 2. Clone repository
git clone https://github.com/Rigohl/MEMORY_P.git
cd MEMORY_P

# 3. Build (core only, no FFI)
cargo build --release

# 4. Run server
./target/release/memory_p

# Server starts on http://127.0.0.1:4040
```

**Time**: ~5-10 minutes  
**Result**: Rust MCP server functional without multi-language capabilities

---

## Full Stack Installation

Complete installation with all FFI languages for maximum capabilities.

### System Requirements

- **OS**: Linux (Ubuntu 22.04+, Debian 12+) or macOS 13+
- **RAM**: 8GB minimum, 16GB recommended
- **Disk**: 15GB free space
- **CPU**: x86-64 with AVX2 (for SIMD optimizations)

### Step 1: Install Core Dependencies

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y build-essential git curl wget \
    pkg-config libssl-dev python3 python3-pip

# macOS (requires Homebrew)
brew install git curl wget pkg-config openssl python3
```

### Step 2: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # Should be 1.75+
```

### Step 3: Install Zig (FFI Bridge)

```bash
# Download Zig 0.12+
wget https://ziglang.org/download/0.12.0/zig-linux-x86_64-0.12.0.tar.xz
tar -xf zig-linux-x86_64-0.12.0.tar.xz
sudo mv zig-linux-x86_64-0.12.0 /opt/zig
echo 'export PATH=/opt/zig:$PATH' >> ~/.bashrc
source ~/.bashrc

# Verify
zig version
```

### Step 4: Install Julia (Mathematical Core)

```bash
# Download Julia 1.10+
wget https://julialang-s3.julialang.org/bin/linux/x64/1.10/julia-1.10.0-linux-x86_64.tar.gz
tar -xf julia-1.10.0-linux-x86_64.tar.gz
sudo mv julia-1.10.0 /opt/julia
echo 'export PATH=/opt/julia/bin:$PATH' >> ~/.bashrc
source ~/.bashrc

# Verify
julia --version

# Install required packages
julia -e 'using Pkg; Pkg.add(["Optim", "LinearAlgebra", "Statistics"])'
# Optional but recommended:
# julia -e 'using Pkg; Pkg.add(["ChaosTools", "DifferentialEquations", "ModelingToolkit"])'
```

### Step 5: Install Python + JAX (ML Inference)

```bash
# Ensure Python 3.11+
python3 --version

# Install JAX (CPU version)
pip3 install --upgrade pip
pip3 install jax[cpu]==0.4.28 sentence-transformers==3.0.1

# For GPU support (CUDA 12):
# pip3 install jax[cuda12]==0.4.28

# Verify
python3 -c "import jax; print('JAX version:', jax.__version__)"
```

### Step 6: Install Mojo (SIMD Kernels) [Optional]

```bash
# Mojo requires registration at modular.com
# Follow instructions at: https://docs.modular.com/mojo/manual/get-started/

# After installation:
mojo --version
```

**Note**: Mojo is optional. MEMORY_P will use Rust fallbacks if unavailable.

### Step 7: Install Pony (Actor System) [Optional]

```bash
# Ubuntu/Debian
sudo apt install -y ponyc

# macOS
brew install ponyc

# Verify
ponyc --version
```

**Note**: Pony is optional. MEMORY_P will use Tokio async if unavailable.

### Step 8: Build MEMORY_P with FFI

```bash
cd MEMORY_P

# Build FFI libraries
cd FFI
make check-deps  # Verify what's available
make all-ffi     # Build all available FFI components
cd ..

# Build Rust with FFI features
cargo build --release --features ffi-all

# Or selectively:
# cargo build --release --features ffi-zig,ffi-julia,ffi-jax
```

### Step 9: Configure Environment

```bash
# Add FFI libraries to library path
export LD_LIBRARY_PATH=$PWD/FFI/lib:$LD_LIBRARY_PATH
echo "export LD_LIBRARY_PATH=$PWD/FFI/lib:\$LD_LIBRARY_PATH" >> ~/.bashrc

# Set Julia home (if needed)
export JULIA_HOME=$(julia -e 'print(Sys.BINDIR)')
```

### Step 10: Run Server

```bash
# Start MEMORY_P server
./target/release/memory_p

# Check logs - should show:
# 🧮 Inicializando Julia mathematical core
# 🤖 Inicializando JAX ML inference
# ⚡ Inicializando Mojo SIMD kernels
# 🎭 Inicializando Pony actor system
# 🌉 Zig FFI bridge listo
# 🚀 MCP Toolkit HTTP iniciando
```

---

## Language-Specific Setup

### Julia Advanced Setup

```bash
# Install additional packages for full capabilities
julia << 'EOF'
using Pkg
Pkg.add([
    "ChaosTools",           # Chaos analysis
    "DifferentialEquations", # ODE/PDE solving
    "ModelingToolkit",      # Symbolic math
    "Plots",                # Visualization
    "BenchmarkTools"        # Performance testing
])
EOF
```

### JAX GPU Setup

```bash
# For NVIDIA GPUs with CUDA 12
pip3 install jax[cuda12]==0.4.28 jaxlib[cuda12]==0.4.28

# Verify GPU
python3 << 'EOF'
import jax
print("Devices:", jax.devices())
print("GPU available:", jax.devices()[0].platform == 'gpu')
EOF
```

### Mojo Optimization

```bash
# Compile Mojo kernels with maximum optimization
cd FFI/src
mojo build kernels.mojo \
    -o ../lib/libmojo_kernels.so \
    --release \
    -D SIMD_WIDTH=8 \
    -march=native
```

---

## Docker Installation

For consistent environment across machines.

### Docker Compose (Recommended)

```yaml
# docker-compose.yml
version: '3.8'

services:
  memory_p:
    build: .
    ports:
      - "4040:4040"
    environment:
      - RUST_LOG=info
      - FFI_ENABLED=true
    volumes:
      - ./data:/app/data
    command: ./target/release/memory_p
```

```bash
# Build and run
docker-compose up -d

# View logs
docker-compose logs -f memory_p
```

### Dockerfile

```dockerfile
FROM ubuntu:22.04

# Install dependencies
RUN apt update && apt install -y \
    build-essential git curl wget \
    python3 python3-pip

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Zig
RUN wget https://ziglang.org/download/0.12.0/zig-linux-x86_64-0.12.0.tar.xz && \
    tar -xf zig-linux-x86_64-0.12.0.tar.xz && \
    mv zig-linux-x86_64-0.12.0 /opt/zig
ENV PATH="/opt/zig:${PATH}"

# Install Julia
RUN wget https://julialang-s3.julialang.org/bin/linux/x64/1.10/julia-1.10.0-linux-x86_64.tar.gz && \
    tar -xf julia-1.10.0-linux-x86_64.tar.gz && \
    mv julia-1.10.0 /opt/julia
ENV PATH="/opt/julia/bin:${PATH}"

# Install JAX
RUN pip3 install jax[cpu]==0.4.28 sentence-transformers==3.0.1

# Copy project
WORKDIR /app
COPY . .

# Build FFI
RUN cd FFI && make all-ffi

# Build MEMORY_P
RUN cargo build --release --features ffi-all

# Expose port
EXPOSE 4040

# Run
CMD ["./target/release/memory_p"]
```

---

## Verification

Test that everything is working:

```bash
# 1. Test server health
curl http://localhost:4040/status

# 2. Test MCP protocol
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list"
  }'

# 3. Test FFI components
cd FFI
make test

# 4. Run Rust tests
cd ..
cargo test --all-features
```

---

## Troubleshooting

### Common Issues

#### 1. "Cannot find libXXX.so"

**Solution**: Add FFI/lib to LD_LIBRARY_PATH

```bash
export LD_LIBRARY_PATH=/path/to/MEMORY_P/FFI/lib:$LD_LIBRARY_PATH
```

#### 2. "Julia initialization failed"

**Solution**: Set JULIA_HOME

```bash
export JULIA_HOME=$(julia -e 'print(Sys.BINDIR)')
```

#### 3. "JAX CUDA not available"

**Solution**: Install correct CUDA version

```bash
# Check CUDA version
nvcc --version

# Install matching JAX
pip3 install jax[cuda12]==0.4.28  # For CUDA 12
```

#### 4. Compilation errors

**Solution**: Update Rust

```bash
rustup update stable
cargo clean
cargo build --release
```

### Getting Help

- **GitHub Issues**: https://github.com/Rigohl/MEMORY_P/issues
- **Discussions**: https://github.com/Rigohl/MEMORY_P/discussions
- **Documentation**: https://github.com/Rigohl/MEMORY_P/tree/main/docs

---

**Version**: 2.0.0  
**Last Updated**: January 2026  
**Tested On**: Ubuntu 22.04, Ubuntu 24.04, macOS 13+
