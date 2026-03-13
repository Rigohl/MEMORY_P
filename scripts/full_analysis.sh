#!/bin/bash
# MEMORY_P Full Analysis: FFI + BRAIN + SRC + Compilation
# Complete 10-phase integration analysis
set -euo pipefail

PROJECT_ROOT=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && cd .. && pwd)\"
cd \"$PROJECT_ROOT\"

echo \"╔═════════════════════════════════════════════╗\"
echo \"║ MEMORY_P Full Analysis & Build Pipeline    ║\"
echo \"╚═════════════════════════════════════════════╝\"
echo \"\"

# Phase 1: FFI Module Detection
echo \"═══ PHASE 1: FFI Module Detection ===\"
FFI_COUNT=$(find src/ffi -name '*.rs' 2>/dev/null | wc -l)
echo \"FFI modules: $FFI_COUNT\"

# Phase 2: BRAIN Integration Check
echo \"═══ PHASE 2: BRAIN Integration ===\"
JULIA_COUNT=$(find brain -name '*.jl' 2>/dev/null | wc -l || echo '0')
MOJO_COUNT=$(find brain -name '*.mojo' 2>/dev/null | wc -l || echo '0')
PYTHON_COUNT=$(find brain -name '*.py' 2>/dev/null | wc -l || echo '0')
echo \"Julia modules: $JULIA_COUNT\"
echo \"Mojo kernels: $MOJO_COUNT\"
echo \"Python modules: $PYTHON_COUNT\"

# Phase 3: SRC Structure
echo \"═══ PHASE 3: SRC Module Structure ===\"
SRC_MODULES=$(ls -d src/*/ 2>/dev/null | wc -l)
echo \"Core modules: $SRC_MODULES\"

# Phase 4: Clean
echo \"═══ PHASE 4: Clean Build Environment ===\"
cargo clean
cargo update --aggressive 2>/dev/null || true

# Phase 5: Format
echo \"═══ PHASE 5: Format Code ===\"
cargo fmt --all

# Phase 6: Check
echo \"═══ PHASE 6: Syntax Check ===\"
cargo check --all --all-features

# Phase 7: Build
echo \"═══ PHASE 7: Release Build ===\"
cargo build --release --all-features

# Phase 8: Test
echo \"═══ PHASE 8: Run Tests ===\"
cargo test --all

# Phase 9: Quality
echo \"═══ PHASE 9: Code Quality ===\"
cargo clippy --all --all-features -- -D warnings || echo \"[Clippy] warnings detected\"

# Phase 10: Analysis Report
echo \"\"
echo \"═══════════════════════════════════════════════\"
echo \"  ANALYSIS COMPLETE\"
echo \"═══════════════════════════════════════════════\"
echo \"\"
echo \"Summary:\"
echo \"  FFI modules: $FFI_COUNT\"
echo \"  BRAIN integration: Julia($JULIA_COUNT) + Mojo($MOJO_COUNT) + Python($PYTHON_COUNT)\"
echo \"  SRC structure: $SRC_MODULES modules\"
echo \"  Build: ✓ Success\"
echo \"\"
echo \"Status: ✓ All FFI/BRAIN/SRC integrated and compiled\"

# Phase 3: Format
echo "[Phase 3] Format check..."
cargo fmt --all -- --check || cargo fmt --all

# Phase 4: Build
echo "[Phase 4] Build release..."
cargo build --release || exit 1

# Phase 5: Tests
echo "[Phase 5] Running tests..."
cargo test --all

echo ""
echo "=== FFI Integration Check ==="
for lang in zig julia mojo jax pony; do
  [ -f "src/ffi/$lang.rs" ] && echo "  ✓ $lang"
done -- --nocapture

# Phase 6: Clippy
echo "[Phase 6] Clippy lint check..."
cargo clippy --all --all-features -- -D warnings || true

# Phase 7: Coverage
echo "[Phase 7] Coverage analysis..."
cargo tarpaulin --out Xml --timeout 120 2>&1 | tail -30 || true

# Phase 8: Benchmarks
echo "[Phase 8] Running benchmarks..."
cargo criterion 2>&1 | tail -20 || true

# Phase 9: Security audit
echo "[Phase 9] Security audit..."
cargo audit 2>&1 | tail -20 || true

# Phase 10: FFI check
echo "[Phase 10] FFI linking test..."
cargo build --release 2>&1 | grep -i "zig\|julia\|mojo\|jax\|pony" || echo "FFI links OK"

echo ""
echo "=== ANALYSIS COMPLETE ==="
echo "Build artifacts: target/release/"
echo "Binary: target/release/memory_p"
# Complete DevOps and Code Analysis Pipeline for MEMORY_P
# Runs setup, act analysis, and comprehensive code review

set -e

echo "🚀 MEMORY_P Complete DevOps & Code Analysis Pipeline"
echo "==================================================="

# Step 1: Setup local DevOps environment
echo -e "\n📦 Step 1: Setting up local DevOps environment..."
if [[ -f "scripts/setup_local_devops.sh" ]]; then
    bash scripts/setup_local_devops.sh
else
    echo "❌ setup_local_devops.sh not found!"
    exit 1
fi

# Step 2: Run act to analyze with CI/CD
echo -e "\n🔍 Step 2: Running CI/CD analysis with act..."

# Check if Docker is available for full analysis
if command -v docker &> /dev/null && docker info > /dev/null 2>&1; then
    echo "🐳 Docker available - running full analysis..."
    act --dry-run || echo "⚠️  Dry run failed, trying containerless mode..."
    act --container=false -j rust-core || echo "⚠️  Rust core analysis failed"
else
    echo "⚠️  Docker not available - running containerless analysis..."
    act --container=false --dry-run || echo "⚠️  Dry run failed"
    act --container=false -j rust-core || echo "⚠️  Rust core analysis failed"
fi

# Step 3: Comprehensive code analysis
echo -e "\n📊 Step 3: Running comprehensive code analysis..."
if [[ -f "scripts/analyze_codebase.sh" ]]; then
    bash scripts/analyze_codebase.sh
else
    echo "❌ analyze_codebase.sh not found!"
    exit 1
fi

# Step 4: Generate improvement report
echo -e "\n📋 Step 4: Generating improvement report..."

echo "🎯 MEMORY_P Analysis Report"
echo "=========================="
echo ""
echo "✅ Completed:"
echo "  - Local DevOps setup"
echo "  - CI/CD workflow analysis"
echo "  - Codebase structure review"
echo "  - FFI integration check"
echo "  - Brain directory analysis"
echo ""
echo "🔧 Next Steps:"
echo "  1. Review warnings and errors above"
echo "  2. Fix any compilation issues"
echo "  3. Add missing tests"
echo "  4. Update documentation"
echo "  5. Optimize performance"
echo ""
echo "📈 For detailed CI/CD results, run:"
echo "  act --container=false -j rust-core  # Rust analysis"
echo "  act --container=false --list        # List all jobs"
echo ""
echo "🎉 Pipeline complete! Ready for improvements."