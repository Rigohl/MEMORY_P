#!/bin/bash
# Simplified DevOps Analysis for MEMORY_P
# Uses existing GitHub Actions workflow locally

set -e

echo "🚀 MEMORY_P DevOps Analysis (Simplified)"
echo "======================================="

# Step 1: Setup act (if not already done)
echo -e "\n📦 Step 1: Ensuring act is available..."
if ! command -v act &> /dev/null; then
    echo "Installing act..."
    if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        powershell -Command "Invoke-WebRequest -Uri 'https://github.com/nektos/act/releases/latest/download/act_Windows_x86_64.zip' -OutFile 'act.zip'"
        powershell -Command "Expand-Archive -Path 'act.zip' -DestinationPath '.' -Force"
        mv act.exe ./act.exe
        export PATH="$PATH:$(pwd)"
    else
        curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | bash
    fi
fi

# Step 2: Run existing GitHub Actions workflow locally
echo -e "\n🔍 Step 2: Running GitHub Actions workflow locally..."

# List available jobs
echo "Available jobs:"
act --container=false --list

# Run Rust core analysis (most important)
echo -e "\n🦀 Running Rust core analysis..."
act --container=false -j rust-core

# Step 3: Quick code check
echo -e "\n📊 Step 3: Quick code health check..."
if [[ -f "Cargo.toml" ]]; then
    echo "🔨 Checking Rust compilation..."
    cargo check --quiet && echo "✅ Compilation successful" || echo "❌ Compilation failed"

    echo "⚠️  Checking for warnings..."
    cargo clippy --quiet -- -D warnings 2>/dev/null && echo "✅ No warnings" || echo "⚠️  Warnings found"

    echo "🧪 Checking tests..."
    cargo test --quiet 2>/dev/null && echo "✅ Tests pass" || echo "⚠️  Tests failed"
else
    echo "❌ No Cargo.toml found"
fi

echo -e "\n🎯 Analysis complete!"
echo "==================="
echo "Use 'act --container=false -j <job-name>' to run specific jobs"
echo "Use 'act --container=false --list' to see all available jobs"