.PHONY: help build test check clean fmt clippy doc bench run release profile security audit install-deps all

# Default target
help:
	@echo "╔════════════════════════════════════════════════════════════════╗"
	@echo "║         MEMORY_P v2.0 - Development Commands (Makefile)       ║"
	@echo "╠════════════════════════════════════════════════════════════════╣"
	@echo "║ Build & Compile:                                              ║"
	@echo "║   make build          - Build release binary                  ║"
	@echo "║   make release        - Optimized release build               ║"
	@echo "║ Testing & Validation:                                         ║"
	@echo "║   make test           - Run all tests                         ║"
	@echo "║   make check          - Run cargo check (fast validation)     ║"
	@echo "║ Code Quality:                                                 ║"
	@echo "║   make fmt            - Format code with rustfmt             ║"
	@echo "║   make clippy         - Run Clippy linter                    ║"
	@echo "║   make clippy-all     - Clippy with all features enabled     ║"
	@echo "║ Documentation & Analysis:                                     ║"
	@echo "║   make doc            - Generate rustdoc documentation       ║"
	@echo "║   make doc-open       - Open docs in browser                 ║"
	@echo "║ Performance:                                                  ║"
	@echo "║   make bench          - Run benchmarks (Criterion)            ║"
	@echo "║   make profile        - CPU profiling with perf              ║"
	@echo "║ Maintenance:                                                  ║"
	@echo "║   make audit          - Security audit (cargo-audit)         ║"
	@echo "║   make outdated       - Check outdated dependencies          ║"
	@echo "║   make clean          - Clean all build artifacts            ║"
	@echo "║ Installation:                                                 ║"
	@echo "║   make install-deps   - Install development dependencies     ║"
	@echo "║   make install        - Install binary locally               ║"
	@echo "║ Running:                                                      ║"
	@echo "║   make run            - Run the server locally               ║"
	@echo "║ Comprehensive:                                                ║"
	@echo "║   make all            - Full: check, test, doc, bench        ║"
	@echo "║   make ci             - CI pipeline: check, test, clippy     ║"
	@echo "╚════════════════════════════════════════════════════════════════╝"

# ============================================================================
# BUILD & COMPILE
# ============================================================================

build:
	@echo "🔨 Building debug binary..."
	cargo build

release:
	@echo "🚀 Building optimized release binary..."
	cargo build --release
	@echo "✅ Release binary at: target/release/memory_mcp_server"

# ============================================================================
# TESTING & VALIDATION
# ============================================================================

test:
	@echo "🧪 Running all tests..."
	cargo test --all-features

test-quick:
	@echo "⚡ Running quick tests..."
	cargo test --lib

test-verbose:
	@echo "🔎 Running tests with output..."
	cargo test -- --nocapture

check:
	@echo "⚡ Running cargo check (fast validation)..."
	cargo check --all-features

# ============================================================================
# CODE QUALITY
# ============================================================================

fmt:
	@echo "✨ Formatting code..."
	cargo fmt -- --check

fmt-fix:
	@echo "✨ Auto-fixing code formatting..."
	cargo fmt

clippy:
	@echo "🔍 Running Clippy (linter)..."
	cargo clippy --all-targets --all-features -- -D warnings

clippy-all:
	@echo "🔍 Running Clippy with all features..."
	cargo clippy --all-targets --all-features --tests -- -D warnings

clippy-fix:
	@echo "🔧 Attempting auto-fix with Clippy..."
	cargo clippy --fix --all-targets --all-features --allow-dirty

# ============================================================================
# DOCUMENTATION
# ============================================================================

doc:
	@echo "📚 Generating rustdoc documentation..."
	cargo doc --all-features --no-deps

doc-open:
	@echo "📖 Opening documentation in browser..."
	cargo doc --all-features --no-deps --open

# ============================================================================
# PERFORMANCE
# ============================================================================

bench:
	@echo "⏱️  Running Criterion benchmarks..."
	cargo bench

bench-quiet:
	@echo "⏱️  Running benchmarks (quiet mode)..."
	cargo bench -- --quiet

profile:
	@echo "📊 Running CPU profiling..."
	@which perf > /dev/null || (echo "⚠️  perf not found. Install with: sudo apt-get install linux-tools"; exit 1)
	perf record -g cargo build --release
	perf report

# ============================================================================
# MAINTENANCE & SECURITY
# ============================================================================

audit:
	@echo "🔒 Running security audit..."
	cargo audit

audit-deny:
	@echo "⛔ Running cargo-deny checks..."
	-cargo deny check

outdated:
	@echo "📦 Checking for outdated dependencies..."
	cargo outdated

update-deps:
	@echo "🔄 Updating dependencies..."
	cargo update
	@echo "Run 'make test' to verify updates don't break anything"

# ============================================================================
# INSTALLATION
# ============================================================================

install-deps:
	@echo "📦 Installing development dependencies..."
	@command -v cargo-clippy >/dev/null 2>&1 || echo "Clippy comes with rustup (rustup component add clippy)"
	@command -v cargo-audit >/dev/null 2>&1 || (echo "Installing cargo-audit..." && cargo install cargo-audit)
	@command -v cargo-outdated >/dev/null 2>&1 || (echo "Installing cargo-outdated..." && cargo install cargo-outdated)
	@echo "✅ Development dependencies installed"

install:
	@echo "📦 Installing memory_mcp_server locally..."
	cargo install --path .

# ============================================================================
# RUNNING
# ============================================================================

run:
	@echo "🚀 Running MEMORY_P server..."
	cargo run --release

run-debug:
	@echo "🔧 Running MEMORY_P server (debug)..."
	cargo run

# ============================================================================
# COMPREHENSIVE TARGETS
# ============================================================================

all: check fmt clippy test doc bench
	@echo "✅ All checks passed!"

ci: check fmt clippy test
	@echo "✅ CI pipeline completed successfully!"

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf target_new/
	rm -rf analysis_results/
	find . -name "*.profdata" -delete
	find . -name "*.profraw" -delete
	@echo "✅ Cleanup complete"

# ============================================================================
# DEVELOPMENT WORKFLOW
# ============================================================================

dev-setup: install-deps build
	@echo "✅ Development environment ready!"

dev-check: fmt clippy test
	@echo "✅ Development checks completed!"

# ============================================================================
# GIT WORKFLOW
# ============================================================================

git-check: fmt clippy test audit
	@echo "✅ Ready to commit! (All checks passed)"

git-status:
	@echo "📊 Git status:"
	@git status --short
	@echo ""
	@echo "📊 Recent commits:"
	@git log --oneline -10

# ============================================================================
# DOCKER
# ============================================================================

docker-build:
	@echo "🐳 Building Docker image..."
	docker build -t memory-p:latest .

docker-run:
	@echo "🚀 Running Docker container..."
	docker-compose up

# ============================================================================
# DEBUGGING
# ============================================================================

debug:
	@echo "🐛 Building debug binary..."
	cargo build

debug-run:
	@echo "🐛 Running with debug output..."
	RUST_LOG=debug cargo run

# ============================================================================
# VERSION & INFO
# ============================================================================

version:
	@echo "MEMORY_P Version Info:"
	@cargo --version
	@rustc --version
	@echo ""
	@grep '^version' Cargo.toml | head -1

info:
	@echo "System Information:"
	@uname -a
	@echo ""
	@echo "Rust Toolchain:"
	@rustup show
