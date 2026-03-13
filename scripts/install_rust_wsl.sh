#!/bin/bash
# Install Rust in WSL2 Ubuntu24D as regular user (no sudo requirements for user installation)
# Target: ~/.cargo/.bin and ~/.rustup

set -e

echo "🦀 Installing Rust in WSL2 Ubuntu24D..."
echo "========================================"

# Check if rustup already installed
if command -v rustup &> /dev/null; then
    echo "🔄 Rust already installed - updating stable toolchain..."
    source "$HOME/.cargo/env" 2>/dev/null || true
    rustup self update || true
    rustup update stable
    rustup default stable
else
    # Install rustup (no sudo needed)
    echo "📥 Downloading and installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
fi

# Source cargo env
echo "🔧 Setting up Rust environment..."
source "$HOME/.cargo/env"

# Verify installation
echo "📋 Verifying Rust installation..."
rustup show active-toolchain
rustc --version
cargo --version

echo ""
echo "========================================"
echo "✅ Rust installation complete!"
echo ""
echo "📝 Add to ~/.bashrc for persistence:"
echo '   source "$HOME/.cargo/env"'
echo ""
